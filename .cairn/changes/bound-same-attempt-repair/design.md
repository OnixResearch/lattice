# Design: Bound same-attempt repair and consume Kiln's budget lineage

## Context

Baseline: Lattice `db64cd654e52cf6579643e6e96ad92bda2b3d61a`.

- `implementation-attempt run` admits one v3 request, records the fence, binding and intent, then runs exactly one bounded OMP step (`cli/session/launch.rs`). Recovery reconciles only retained results.
- The request schema is versioned. Optional coordinator fields (`worker_capability`, `network`, `dispatch_plan`) decode as absent and are not serialized when absent, so earlier requests keep their bytes.
- Usage is the `Scope::Attempt` `CommandWorkerObservation` (`cli/binding/usage.rs`): host duration when observed, model and device usage unknown, persisted before the result.
- Kiln `339116f6` keeps one recursive lineage per root (`fair_admission/lineage.rs`) and gates requests on serving snapshots (`fair_admission/serving.rs`). Its explanation carries `LineageExplanation { root, account, depth, blocker }` and `ServingExplanation { gate, .. }`; the account's `available` `Amounts` is its remainder.

## Goals and non-goals

Goal: the issue #6 repair, bypass, disposition and usage criteria as a pure core plus a shell over injected ports, and the Kiln #61 lineage as an input that only ever draws.

Non-goals: an OMP worker port, repaired-candidate export and inventory projection, Kiln-side emission, live qualification.

## Decision 1: two optional request fields, one framed

`repair` and `lineage` follow the existing optional-field pattern. The policy describes how the profile behaves, so it is framed into the profile revision: `repair::profile_revision(single_shot, policy)`, a domain-separated BLAKE3 over the unchanged single-shot revision and the policy identity. Without a policy the revision is exactly the single-shot revision. The lineage is a per-attempt snapshot and is bound only by the exact request bytes. Rejected: framing the policy through `lattice-core`'s `WorkerProfileRevisionInput`, which would move every revision.

## Decision 2: policy bounds tie to the frozen envelope

Every worker step is one envelope turn, so `max_iterations + 1 <= budget.max_turns`; the elapsed bound must fit `budget.max_seconds`; the aggregate model and tool budgets must cover one step and its checks. A step reserves the broker's request ceiling in model requests and `step_tool_invocations` plus one per declared check in tool invocations. Iterations are bounded by 8, checks and failure classes by 16. Lists must be strictly sorted so one policy has one identity.

## Decision 3: conservative charges

A worker step is charged `max(reservation, reported)` per dimension, and the reservation when the count is unknown, which is the orchestration ledger's rule. An under-report cannot lower the charge, an overrun is charged, and an unknown count stays `None` with `unknown` provenance. Checks are host executions: known count, zero model requests, `host_measured`. Preparation is a host step with zero model and tool use. Sums saturate, which only exhausts bounds sooner.

## Decision 4: the loop is a pure state machine

`state::Loop` answers `next(elapsed, cancel)` before every spawn, in the order cancellation, clock, elapsed bound, iteration bound, aggregate budgets and lineage. `start` refuses a step whose binding differs from the frozen one (`changed_work` for work, version or task; `new_semantic_attempt` for attempt, fence or envelope) before recording it. `after_step` stops first on a cancelled or unknown worker outcome, then on a Kiln-owned proposal, then on an unreadable clock, a reached elapsed bound or charges already over a policy budget or the lineage remainder, and maps no new candidate to the reserved class `no_candidate`. `after_checks` maps the verdict; a pass is re-checked against the clock, the elapsed bound and the charged totals first. A failure inside the admitted classes returns `Repair`; the shell then asks `next` again, so every bound applies to every repair.

## Decision 5: candidates are host-derived

After an exited step the shell snapshots the workspace (`candidate_capture::workspace`). Equal to the parent or the base is no candidate. Otherwise `candidate::produced_receipt_from_snapshots` against the admitted base gives a receipt bound to work, attempt, fence and workspace, and `repair::revision::identity` binds envelope, attempt, fence, iteration, parent revision and the receipt's candidate and diff identities. Each check invocation names that revision. A cancelled or uncertain step's workspace is retained, never captured. The base is re-observed before the first step and must equal the admitted `base_identity`.

## Decision 6: check evidence precedence

`checks::classify` rejects defects first (undeclared, duplicate, self-approval by the worker profile, other executor, crossed revision, changed configuration, skipped, missing), then reports unknown outcomes, then failures, else passes. The worker's exit status and files are never inputs. Self-approval compares the executor with the worker profile revision; the policy names the only admitted executor.

## Decision 7: shell, records and recovery

`cli::repair::Ports` (worker, checks, clock) is a new optional member of `cli::Ports`; the standalone CLI passes `None`. `repair::admit` runs before the session and any write: a lineage that cannot cover the first step is a typed `lineage_exhausted` or `lineage_blocked` refusal, and a policy without ports is `repair_ports_unavailable`; all three print `dispatched:false`. After the existing fence, binding, intent, revalidation and revision permit, the shell drives the loop with at most `3 * (8 + 1) + 2` transitions. Under `<attempt>/repair/` it writes `step-N.intent` before each worker effect, `step-N.json` after each step's checks and `disposition.json` last, all write-once. The `Scope::Attempt` usage observation is persisted before the disposition. Recovery and duplicate runs print the retained disposition after checking its schema, worker-request and envelope identity; a missing one is an unknown outcome and never reruns.

## Decision 8: the lineage consumer type

`lineage::Binding` maps Kiln's types field by field (recorded in `docs/candidate-worker.md`): `root`, `account` and `depth` from `LineageExplanation`, `parent` from the Kiln account record, `remaining` from the account's `available` `Amounts`, `blocker` from `LineageExplanation.blocker` (only `denied`, `expired`, `cancelled`, `budget_exhausted`), and `backpressure` from `ServingExplanation.gate`, or `none` without a serving owner. Identities must be lowercase 64-hex, which Kiln's derived identities are. `gate` refuses a blocker, any gate other than `none` or `eligible`, a zero remainder in any dimension, and a remainder below the next reservation in `model_requests` or `tool_invocations`; other dimensions are never drawn and are reported as unaccounted. `admit_child` requires the same root, this account as parent, depth plus one, and a ceiling within the remainder in every dimension, with none added or omitted. A single-shot attempt with a lineage reports a conservative draw of one broker ceiling and lists `tool_invocations` as unaccounted.

## Decision 9: resolutions of the independent review at `647bc59d`

The review found the loop could pass a candidate after its authority, time or budget ran out, or over bytes its checks did not test. Each resolution keeps the decisions pure and the effects in the shell.

- **Authority before every step.** Under the step's fence lock, every step after the first calls `binding::recheck::current` and `session::revision::recheck`, the non-writing halves that `binding::revalidate` and `session::revision::permit` now share, so the per-step check is the pre-spawn check with the same seam subject. A lost binding, a blocked session or a seam error ends the loop as `authority_withdrawn` with its cause, retained with the evidence in the observation. It writes neither pre-dispatch record, because inventory reads those as `binding_rejected_before_dispatch` and `revision_blocked_before_dispatch`. The first step is covered by the pre-spawn checks that immediately precede the loop.
- **Checks against the captured candidate.** `cli::repair::materialized` writes each check its own copy of the host snapshot, with write bits removed, and captures it again afterwards. A changed copy is passed to `checks::classify` as the drifted check, which outranks every other defect as `candidate_drift`, and no further check runs. Copies are removed after their check.
- **Elapsed bound through the terminal decision.** `Loop::remaining_ms` gives the worker and each check the time left; the ports take it as a deadline. The clock is read again after the worker and after the checks, and a reached bound or an unreadable clock ends the loop.
- **Budgets before a pass.** `Loop::settled` compares the charged totals with every policy budget and `lineage::Binding::overdrawn` before checks run and before any passing disposition.
- **Unknown outcomes outrank proposals.** `after_step` handles cancelled and unknown outcomes before a proposal.
- **Lineage before the state root.** `Context::load` only reads; the state root and attempt directory are created after the lineage gate, the repair ports and the session admission, before the fence lock is taken, so the `e3ef038c` lock ordering is unchanged.
- **Kiln's `Alias` names.** Dimension names follow `Alias::new` at Kiln `339116f6`: non-empty, at most `DEFAULT_MAX_ALIAS_BYTES` (256). Names Lattice cannot charge stay unaccounted.

## Tests

- `src/implementation_attempt/lineage/tests.rs`: validation, Kiln alias names, identity, gate, overdraw, child admission and wire shape.
- `src/implementation_attempt/repair/tests.rs`: policy admission, check classification and drift, every disposition, step binding, conservative charges, unknown outcomes over proposals, budget and lineage overruns before a pass, and the elapsed bound and clock through the terminal decision.
- `src/implementation_attempt/cli/tests/repair.rs`: the real `run`/`recover` shell with a fixture worker, check port and clock for every disposition, the bypass cases, Kiln-owned proposals, typed refusals that leave the state root unchanged, a lost controller, base drift, absent-field byte identity, the single-shot draw, a tool overrun whose check would pass, per-check read-only copies and drift, slow and clockless workers and checks, and a withdrawn execution binding and a revoked or unreadable managed revision before the second step.

## Risks and claim boundaries

- The shell cannot interrupt a port: a worker or check that ignores its deadline is detected when it returns, and its step or check then ends the loop as `budget_exhausted` (`elapsed`), never as a pass.
- Drift is detected by capturing a check's copy after it returns. A check that changes its copy and restores the same bytes before returning is not detected; it still never touches the worker workspace or another check's copy.

- The fixture ports prove the shell's decisions over supplied observations, not any real worker, check executor or Kiln lineage. No OMP process or model endpoint runs.
- The lineage is a frozen snapshot; Lattice does not observe Kiln's current ledger, so concurrent siblings are bounded only by Kiln's own transfers.
- Worker-reported counts are retained but never lower a charge; check outcomes are trusted only from the admitted executor identity, which the embedder must bind to a real host check runner.
