# Design: Export repaired candidates and report repair dispositions in inventory

## Context

Baseline: Lattice `0862c017`; first version `2dff32d1`, revised after the independent review `RepairExportReview`.

- A repair run writes `repair/step-N.intent` before each worker effect, `repair/step-N.json` after each step and `repair/disposition.json` last, all write-once. It publishes no native attempt result; it persists only the `Scope::Attempt` usage observation in `candidate.redb`.
- Single-shot `export-candidate` admits the request, the intent of the exact request bytes, the output parent and the revision seam (`session::revision::exportable`, `export_revoked`), then requires a published native result and binds the workspace to its receipt.
- `inspect` derives `terminal_observed` from the native result only (`inspect::terminal_disposition`).
- Kiln `4899d149` (`kiln-adapter-lattice/src/worker_inventory.rs`) decodes `LifecycleDto::TerminalObserved { disposition: String }` without validating the string, maps it to `WorkerLifecycle::Stopped`, and requires `exited` liveness and no instance wait reasons for it. Its refusal classifier (`worker_settlement.rs`) treats any reason outside four no-start reasons, or any `dispatched:true`, as `Retain`.

## Goals and non-goals

Goal: export exactly the revision a passing disposition names, and report retained dispositions in inventory v3 without a schema change, deriving both from the complete, request-bound retained evidence, re-derived and cross-checked, never from a single recorded field.

Non-goals: a new inventory schema, Kiln changes, live qualification, and protection against an actor who can rewrite the private state directory and forge a fully consistent record set.

## Decision 1: typed, complete records

`repair::retained` holds `Observation` (v2), `StepRecord` (v2), `Withdrawal` and `CopyIntegrity`. The shell writes both records from these types and reads them back into them; `deny_unknown_fields` refuses extra fields and a `required` deserializer refuses a left-out nullable field. `dispatch_plan` and `authority_withdrawn` are present exactly when the request has a plan and the disposition is `authority_withdrawn`, which the reader checks. `revision::Record` (moved from the shell) adds `executables` (the `mode & 0o111` of every executable file, the only mode state a check's copy is sealed with besides read bits; capture admits only regular files) and `base_files` (the base length and BLAKE3 of every changed path, `None` where the base had no file). The revision identity moves to the `lattice.implementation-attempt-candidate-revision.v2` domain and also binds `executables`. A step record adds `copies`: per invocation, the declared check and whether its copy was intact. The record bound rises from 1 MiB to 8 MiB for the base digests. Recovery's read still prints a v1 disposition; export and inventory treat v1 as lacking evidence (`Unretained::Legacy`). Rejected: keeping the v1 schema names with new required fields, which could not tell an older record from a malformed one.

## Decision 2: one evidence reader for export and inventory

`cli::repair::evidence::read` returns the typed disposition and the typed intent and record of every step it counts, or `Missing`, `Legacy`, `Malformed` or `Crossed`. It requires the disposition to name this request's bytes and admitted envelope, this envelope, the persisted execution binding, the policy and lineage identities, the lineage's unaccounted dimensions and the dispatch plan, with `accepted:false` and `integrated:false`; at most the admitted policy's step count; each step's intent equal to its record's step at its iteration with this attempt's binding; the recorded revisions equal to the step records' candidates; and nothing beyond the next step intent. A step intent after the counted steps without a record is reported to the caller. Rejected: reading only the schema and the two identity strings, which let a partial or copied record through.

## Decision 3: the export decision is a replay in the pure core

`repair::exportable::candidate(frame, retained)` takes what the request admits (`Frame`: step binding, receipt bindings, base identity, policy, worker profile) and the evidence. A non-passing kind is `NotCandidate`. A passing one must have `repairs` within the policy's `max_iterations` and the global 8, and `repairs + 1` step records. Every revision must have an increasing iteration within those bounds, name the revision before it as parent, carry a produced receipt bound to this work, attempt, fence, workspace and base whose changed paths are exactly its `base_files` keys, and re-derive. Every step must be this attempt's exited step with the loop's phase, parent and failure classes and no proposal; its verdict is re-derived with `checks::classify` from its observations and the copy integrity aligned with the policy's invocation order (only the last copy may have drifted) and must equal the recorded one. Each step before the last must fail only with admitted classes, or produce no candidate with `no_candidate` admitted; the last must pass with the named revision. Anything else is `Contradicted`. Rejected: trusting the recorded `verdict`, which a relabelled failed or drifted step defeated.

## Decision 4: the workspace and the whole receipt

The shell captures the workspace and its modes once. The execute bits must equal the record's; the receipt is re-derived by `candidate::digests::produced_receipt_from_changes` from the captured candidate's digests and the retained `base_files`: the base is the candidate with those paths replaced or removed, a listed path equal to the candidate is refused, and the base, candidate and diff identities and the changed paths follow from the same formulas as `produced_receipt_from_snapshots` (shared through `snapshots::{tree,diff}_identity_of`). The re-derived receipt must equal the recorded one and its base identity the admitted one. Publication uses the same captured bytes and modes. Rejected: the byte-only candidate identity, which an after-the-fact `chmod +x` and a rewritten `changed_paths` both passed.

## Decision 5: typed refusals

`Missing` is `repair_outcome_unknown`; `Legacy`, `Malformed`, `Crossed` and `Contradicted` are `repair_disposition_invalid`; `NotCandidate` is `repair_not_candidate`. All three are `dispatched:true` because the attempt's intent exists and its steps may have run, so Kiln `4899d149` retains them. A missing disposition is never inferred from step records. A drifted workspace or changed mode keeps an untyped diagnostic, like the single-shot export's. `cli/refusal.rs` would exceed 300 lines; its unit tests move unchanged to `cli/refusal/tests.rs`.

## Decision 6: inventory reports only proven stops

For a request with a repair policy, `inspect` reads the same evidence. `Missing` and `Legacy` keep `dispatch_recorded_outcome_unknown`; `Malformed` and `Crossed` are errors. The kind must agree with the records (`exportable::is_consistent`): every recorded verdict is its re-derived one, a passing kind replays as for export, `not_repairable` is the last step's re-derived failure outside the admitted classes (or `no_candidate`), `checks_rejected` its re-derived defect, and `uncertain` (`check_outcome`) its re-derived unknown; otherwise it is an error. Kinds decided by the clock, cancellation, the fence, authority, the lineage or Kiln cannot be re-derived from step records. Then `retained::is_stopped` decides: every recorded step must report `exited` or `cancelled`, `uncertain` (`worker_outcome`) is never stopped, and a step intent without a record is a never-called worker only for `budget_exhausted` (`elapsed`) and `uncertain` (`clock`), which the loop decides after the intent and before the worker call; for any other kind, such as `kiln_owned` from a refused step binding, that worker may have run. Only then is the instance `terminal_observed`, which the core reports with `exited` liveness and no wait reasons. A v4 would only be needed to publish the revision, repair count, bound, cause or defect, or a structural repair/single-shot distinction (a typed `terminal_observed.repair` object, which Kiln's `deny_unknown_fields` DTO would reject in v3).

## Verification

`cargo test -p lattice --lib -- implementation_attempt` and `candidate::`, the worker-inventory tests, `tests/test_implementation_attempt_refusals.rs`, the `LATTICE_INVENTORY_CAPTURE_DIR` capture tests against a `0862c017` reference capture, clippy with `-D warnings`, `cargo fmt -p lattice --check`, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks on the changed paths; results in `evidence/verification.json`.
