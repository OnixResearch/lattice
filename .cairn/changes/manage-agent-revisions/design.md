# Design: Manage immutable agent revisions

## Context

Baseline: Lattice `9a74e7ef1da4e0ca7610dce0db039b3825a7154d`.

- `Request::worker_profile_identity()` (`src/implementation_attempt/cli.rs`) is the `lattice-worker-profile-revision-v2` identity (`content_identity/attempt.rs`) over profile-level configuration, plus the canonical capability declaration when one is present; the `#6` slice removed the base snapshot, task prompt, broker expiry and workspace root from it. It is published bare (64 hex) as the inventory `revision` and the envelope's `worker_profile`.
- The inventory (`publish-worker-inventory-v2`) publishes a definition identity that is stable across revisions and five revision slots as unknowns owned by `lattice#25`.
- The store is one redb database with private per-domain modules and version-checked transitions (`commit_orchestration_transition(expected_version, ...)`).
- Injected ports follow `bind-model-serving-admission`: a trait in the shell, an `Unavailable` implementation installed by the standalone CLI, a `Fixture` implementation for tests, and `run_command_with_ports` for embedders.

## Goals and non-goals

Goal: issue #25 sections 1 to 5 at fixture level, the reviewed harness template in the revision, and the seam wired into implementation attempts so that an attempt without a revision store behaves exactly as before.

Non-goals: inventory publication (`publish-worker-inventory-v3`), live preparation or qualification, canaries, checkpoint and resume, and the OCI and Cairn follow-ups on the issue.

## Decision 1: the profile revision is the revision identity

The lifecycle uses `worker_profile_identity()` verbatim and treats it as opaque, validated only as 64 lowercase hex. The definition is the inventory's `lattice-worker-definition-v1` identity (`blake3:<hex>`), so a new executable, reviewed harness flag, tool list or route keeps the definition and changes the revision. A new task prompt changes neither.

What the revision frames, by section 1 component:

| Component | Framed | Where |
|---|---|---|
| Harness and executable | yes: program and sandbox paths, OMP source revision and executable pin, broker relay, shell and sandbox pins | `content_identity/attempt.rs` |
| Harness argv | yes: the launch prefix `launch --cwd /workspace` and the print options, in argv order | `candidate_worker::omp::profile::{LAUNCH_PREFIX, PRINT_OPTIONS}` through `implementation_attempt::cli::session::profile::EFFECTIVE_TEMPLATE` |
| Tools and protocol | yes: the exposed tool list `--tools read,edit,write`, `-p --mode json` and `--approval-mode write` are print options; declared tool names through the capability | `omp/profile.rs` |
| Model route | yes: OMP model selector and the broker's schema, endpoint, model, credential reference and request bound | `content_identity/attempt.rs` |
| Model requirements | no: serving route requirements live in the per-attempt `execution_binding` | `execution_binding/serving/mod.rs` |
| Memory policy | yes: single-shot `--no-session` is a print option | `omp/profile.rs` |
| Checkpoint semantics | no: inventory reports `unavailable` | `cli/inspect/mod.rs` |
| Workspace and isolation | read and write paths framed; isolation and private network are code invariants | |
| Compute | yes | `content_identity/attempt.rs` |
| Capability | yes, when declared | `content_identity/attempt.rs` |
| Producer/consumer compatibility | yes: the request schema | `implementation_attempt::cli::REQUEST_SCHEMA` through `implementation_attempt::cli::session::profile::EFFECTIVE_TEMPLATE` |
| Per-attempt values | no: workspace snapshot `base_identity`, task `prompt`, broker `expires_at_ms`, workspace root; the exact request bytes bind them | |

## Decision 2: frame the effective harness template, not a declaration

`WorkerProfileRevisionInput` takes the request schema, the launch prefix and the print options as ordered strings. `Request::framed_revision(HarnessTemplate)` (`implementation_attempt::cli::session::profile`) builds the revision for any template, and `worker_profile_identity()` passes `EFFECTIVE_TEMPLATE`, which borrows the constants the shell actually passes to the worker (`candidate_worker::omp::profile::{LAUNCH_PREFIX, PRINT_OPTIONS}`, both `pub(crate)` and never restated). Changing a reviewed flag, its order, the launch prefix or the request schema is therefore a new revision. The domain stays `lattice-worker-profile-revision-v2` because the v2 framing was never published.

Rejected: the operator-declared `lattice.agent-revision-components.v1` extension (`framing.rs`), which an earlier revision of this change added. Nothing wired it, and a declaration could disagree with the constants that actually run. It and the bundle's `components` digest are deleted.

## Decision 3: separate facts in one bounded lifecycle per definition

`state::Lifecycle` keeps `desired` and `change_generation`, `candidates` (bundle, preparation, occupied artifacts, cleanup status, holds), `qualifications` per (revision, task family), `last_usable`, `selected` and `rollout_generation`, `previously_selected`, `sessions` (pins, with the checkpoints a closed session holds), `receipts` by request identifier and monotonic `revoked` sets. Nothing is derived from anything else.

Bounds count only what is still referenced, so none is a lifetime limit: 64 revisions, 1024 live sessions, the 256 most recent receipts, 256 qualification scopes, 1024 revocations per kind of unreferenced targets, and 128 artifacts or holds per revision. When revisions or qualification scopes are full, `ops::forget_unreferenced` drops every revision that is unprepared, failed or superseded, occupies no artifact, has no hold and is not desired, selected, last usable or pinned, with its qualifications and `previously_selected` entry. A closed session is forgotten once none of its checkpoints is held (`sessions::forget_unheld`, at close and at hold release). The oldest receipt (lowest rollout generation) is forgotten when the window is full; because the receipt binds the expected rollout generation, its request can never match the current generation again and is refused rather than applied twice. Revocations are never forgotten, and revoking a referenced target is never refused, so the revocation bound cannot fail open.

Rejected: keeping closed sessions in a recency window for idempotent close. The pin carries nothing once its checkpoints are released, and session identifiers are unique per attempt.

## Decision 4: state machine

```text
desire(bundle)        -> change_generation+1, desired; new candidate Unprepared (Failed or Superseded reset to Unprepared)
begin(revision)       -> Unprepared, not revoked -> Pending{g}: Started(ticket{definition, revision, g}); revoked: revision_revoked
                         Pending{g}/Unknown{g} at current g: Reissued(same ticket), even when revoked; older g: preparation_in_flight
                         Prepared: AlreadyPrepared (no re-preparation); Failed / Superseded: refused
reissue(revision)     -> Pending{g}/Unknown{g} at any g, desired or not: that attempt's own ticket; otherwise stale_ticket
                         (the shell's `prepare` falls back to it when begin refuses, so old attempts settle)
record(ticket, obs)   -> only an open attempt (Pending/Unknown) with ticket.g settles:
                         Prepared & ticket is current desired -> Prepared{g}, last_usable
                         Prepared & not current               -> Superseded{g} (late; retained for cleanup only)
                         Failed -> Failed{g}; Unknown -> Unknown{g}; artifacts merged
                         a settled attempt: same result unchanged (prepared matches Prepared{g} or Superseded{g}),
                         different result_conflict; other g stale_ticket
select(cmd, verdict)  -> receipt for request_id? same kind, revision, ref, policy and expected generation: AlreadyApplied;
                         else request_conflict
                         eligible (prepared, not revoked, ref not bound elsewhere, not already selected,
                         rollback: previously selected with the same recorded ref)
                         accept verdict (qualified, canonical, required class, same ref)
                         expected rollout generation == current, else generation_conflict
                         -> rollout_generation+1, selected, qualification recorded, receipt (oldest forgotten past 256)
admit(session, rev)   -> existing pin: other rev session_conflict; closed session_closed; revoked revision or ref refused;
                         else AlreadyPinned
                         new: rev == selected, not revoked, < 1024 live -> pin {rev, ref, rollout generation}
effect_permitted      -> live, session/revision/qualification not revoked
revoke(target)        -> monotonic; never refused for a referenced target; a revoked last usable revision stops being last usable
close(session, ckpts) -> Closed; checkpoints held on the pinned revision; the pin is forgotten unless a checkpoint is held
retain / plan / record cleanup -> released only on observed release; failed or unknown -> uncertain
release(hold)         -> hold removed; closed pins with no held checkpoint forgotten
```

A favorable verdict is an input to `select`, never a trigger for it.

## Decision 5: idempotency and concurrency through the existing store pattern

`src/store/revisions.rs` adds table `agent_revision_lifecycles_v1`: definition identity to `{version, lifecycle}`. `commit_lifecycle(expected_version, lifecycle)` refuses any other current version. The shell (`agent_revision::transact`) loads, applies the pure transition, skips the write when nothing changed, commits with the version it read and reapplies up to four times after a conflict. Duplicate requests return their receipt; concurrent requests that expect the same rollout generation select once; a lost acknowledgement is reconciled by repeating the same request identifier, which neither prepares nor selects again. The receipt binds the expected rollout generation, so a reused identifier with another expectation is `request_conflict`.

## Decision 6: ports and the standalone CLI

`PreparationPort::{prepare, cleanup}` and `QualificationPort::answer` live in `src/agent_revision/mod.rs`. The query carries the exact definition identity, bare revision, task family, required class and the shell's `now_ms` for the producer's freshness check; the core never reads it. The attachment records `observed_at_ms` verbatim. `run_command` installs `UnavailablePreparation` and `UnavailableQualification`; `run_command_with_ports` takes embedder ports.

## Decision 7: the session-admission seam

`seam::admit(store, request, session)` returns `Unmanaged` without writing when no lifecycle exists, `Admitted` with the pin, or `Refused`. `Decision::admission()` gives the mode the caller records durably with the session: `Admission::Managed` for `Admitted`, `Admission::Unmanaged` for `Unmanaged`, none for `Refused`. `seam::effect(store, request, session, admission)` and `seam::close(store, request, session, admission, checkpoints)` take that recorded mode. An `Unmanaged` session answers `Unmanaged` without touching the store for its whole life, even after the first `desire` makes its definition managed, so in-flight sessions behave exactly as before. A `Managed` session fails closed: `Blocked`/`Refused` with `unknown_session` when its lifecycle or pin is missing. New sessions of a managed definition still require a selection. Implementation attempts are not wired in this change.

Rejected: deriving the mode from whether a lifecycle exists at effect time. The first `desire` would then block every in-flight session admitted before management as `unknown_session`.

## Decision 8: slots for a later inventory

`projection::slots(&Lifecycle) -> Slots` (`lattice.agent-revision-slots.v1`) projects desired, prepared, qualified (with class, provenance and revocation), last usable, selected and pinned facts, with whether each pinned session is blocked. Bundles, artifacts, holds and receipts are not projected. `projection::session_pin` resolves one session's pin from the lifecycle or, once closing forgot it, from the pin the embedder recorded (`RecordedPin`, status `released`).

## Decision 9: the seam in implementation attempts

`implementation-attempt run`, `recover`, `export-candidate` and `inspect` take an optional `--revision-store`; `implementation_attempt::cli::session::revision` opens the store only around each seam call, so `revoke` is never locked out during a run. The session is the attempt's envelope identity.

1. `run` calls `seam::admit` before the fence record, binding, candidate store or intent. `Refused` prints a typed `revision_not_admitted` refusal and writes nothing. A session this run pinned is closed if the fence or binding step fails before the intent.
2. With a store, `intent.json` is the canonical `lattice.implementation-attempt-intent.v1` record of request identity, session, admission and pin (`implementation_attempt::cli::session::intent`). Without one it stays the bare request identity. A legacy intent without an admission field reads as `unmanaged`, since it was admitted unmanaged by construction; the mode is never re-derived.
3. `seam::effect` runs after binding revalidation, immediately before spawn. `Blocked` retains `revision-block.json`, closes the session and refuses as `revision_blocked`; recovery reports `not_dispatched_revision_blocked`.
4. `seam::close` runs without checkpoints after the terminal result, in `run` and in `recover`; `unknown_session` (already closed and forgotten) is settled.
5. Recovery, inspection and export of a managed attempt require the store. Export also requires `sessions::retained_permitted` for the recorded pin, so revocation of the session, its revision or its qualification refuses export after the session closed.

Rejected: a revision store implied by the state directory. An operator who never manages a definition would get a lifecycle database per run, and an unmanaged run would no longer be byte-identical.

## Tests

`tests/test_agent_revision_core.rs` covers each issue criterion on the pure core, every refusal, each bound filled past its size, reissue of abandoned attempts and the committed fixture identities. `tests/test_agent_revision_shell.rs` covers resolution, persistence with an interleaved writer, the standalone CLI, the `lattice` binary's exit status, fixture ports and the seam's admission modes. `src/implementation_attempt/cli/tests.rs` covers every reviewed harness flag and the request schema. `src/implementation_attempt/cli/tests/revisions.rs` runs the CLI with a fixture worker: desire, fixture preparation, promotion with an `encode_attachment` attachment, a managed run, revocation, a refused later run and a refused export; refusal without a selection; revocation between admission and spawn; an unmanaged run with byte-identical intent; and recovery that keeps the recorded mode.

## Risks and claim boundaries

- Fixture ports are not live preparation or qualification. The evidence states `live_qualification: false`.
- A managed attempt's session is admitted only when its request reproduces the selected revision exactly; any profile-level change, including a reviewed harness flag, needs a new desire, preparation and promotion.
- `effect_permitted` covers lifecycle revocation only; cancellation, deadlines, grants and budgets keep their existing owners.
