# Design: Export repaired candidates and report repair dispositions in inventory

## Context

Baseline: Lattice `0862c017`.

- A repair run writes `repair/step-N.intent` before each worker effect, `repair/step-N.json` (the step, its report, its candidate `Revision`, its check observations and verdict) after each step, and `repair/disposition.json` (`lattice.implementation-attempt-repair-observation.v1`: envelope, request and worker-request identities, execution binding, disposition, step count, recorded revisions with their receipts) last, all write-once. It publishes no native attempt result; it persists only the `Scope::Attempt` usage observation in `candidate.redb`.
- Single-shot `export-candidate` admits the request, the intent of the exact request bytes, the output parent and the revision seam (`session::revision::exportable`, `export_revoked`), then requires a published native result and binds the workspace to its receipt.
- `inspect` derives `terminal_observed` from the native result only (`inspect::terminal_disposition`).
- Kiln `4899d149` (`kiln-adapter-lattice/src/worker_inventory.rs`) decodes `LifecycleDto::TerminalObserved { disposition: String }` without validating the string, maps it to `WorkerLifecycle::Stopped`, and requires `exited` liveness and no instance wait reasons for it. Its refusal classifier (`worker_settlement.rs`) treats any reason outside four no-start reasons, or any `dispatched:true`, as `Retain`.

## Goals and non-goals

Goal: export exactly the revision a passing disposition names, through the same admission as a single-shot export, and report retained dispositions in inventory v3 without a schema change.

Non-goals: a new inventory schema, Kiln changes, live qualification.

## Decision 1: the revision check is a pure core

`repair::exportable::candidate(binding, Retained { disposition, steps, revisions })` decides from the retained facts only. A disposition other than `correct_candidate` or `repaired_candidate` is `NotCandidate`. A passing one must name the last recorded revision at iteration 1 or `repairs + 1` (with `repairs > 0`), equal to the recorded step count; the loop issues `correct_candidate` only when one step ran and `repaired_candidate` with `steps - 1` repairs, after the checks of the last revision. Every recorded revision must have a strictly larger iteration than the one before, name it as parent (the first none) and re-derive through `revision::identity` from `StepBinding::of(envelope)`, so a record of another attempt or fence never passes. At most `MAX_REPAIR_ITERATIONS + 1` revisions are read. Anything else is `Contradicted`. The recorded revision type moves from the shell (`cli/repair/steps.rs`) to `repair::revision::Record` with `Deserialize`; its serialized form is unchanged. Rejected: comparing only the named revision string, which would accept a forged chain or a record copied from another fence.

## Decision 2: the shell binds the record to this request, its step record and the workspace

`cli/export/repaired.rs` runs after `admission::load`, which is unchanged except that it keeps the request identity. A binding refusal or a retained revision block means the attempt never dispatched and has no candidate. `cli::repair::retained` reads the disposition with a typed `Unretained` (`Missing`, `Malformed`, `Crossed`: other worker-request or request identity); recovery's existing `disposition` maps it to the same diagnostics as before, except that an unreadable record now reads "malformed". The shell then requires the record's `envelope` and `execution_binding` to equal this request's and its persisted binding, the final receipt to be a produced receipt bound to this work, attempt, fence, workspace and base (the single-shot `receipt_matches`), and `repair/step-N.json` of the final iteration to carry the same revision with a `passed` verdict. The workspace must capture to the receipt's candidate identity: that is the snapshot each check's read-only copy was written from. Output layout and replay are the single-shot `publication::write`, with the receipt as `host-receipt.json` and the disposition as `native-observation.json`. Rejected: re-materializing the candidate from records, which retain no file bytes.

## Decision 3: typed refusals

`Missing` is `repair_outcome_unknown`; `Malformed`, `Crossed` and `Contradicted` are `repair_disposition_invalid`; `NotCandidate` is `repair_not_candidate`. All three are `dispatched:true` because the attempt's intent exists and its steps may have run, so Kiln `4899d149` retains them. A missing disposition is never inferred from step records, so a crash between a step record and the disposition stays unknown. `cli/refusal.rs` would exceed 300 lines; its unit tests move unchanged to `cli/refusal/tests.rs`. A drifted workspace keeps the single-shot export's untyped diagnostic.

## Decision 4: inventory v3 carries the disposition kind

For a request with a repair policy, `inspect` reads the same typed disposition: `Missing` keeps `dispatch_recorded_outcome_unknown` and its `dispatch_outcome_unknown` reason; `Malformed` or `Crossed`, or a `disposition` that does not decode as `repair::state::Disposition`, is an error, as for any contradictory artifact; otherwise the instance is `terminal_observed` with the disposition's `kind`. The core derives `exited` liveness, no wait reasons, and `observed_terminated` for `cancelled`, as for a single-shot result. Kiln's v3 reader decodes `disposition` as a free string, so no enumeration it validates changes. The revision, repair count, bound, cause and defect are not in v3; they stay in the retained record. A v4 would only be needed to publish those details or to distinguish repair from single-shot dispositions structurally: a typed `terminal_observed.repair` object, which Kiln's `deny_unknown_fields` DTO would reject in v3. Rejected: keeping the conservative unknown, which misreports a retained terminal outcome as possibly running.

## Verification

`cargo test -p lattice --lib -- implementation_attempt`, the worker-inventory tests, `tests/test_implementation_attempt_refusals.rs`, the `LATTICE_INVENTORY_CAPTURE_DIR` capture tests, clippy with `-D warnings`, `cargo fmt -p lattice --check`, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks on the changed paths; results in `evidence/verification.json`.
