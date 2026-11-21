# Design: Manage immutable agent revisions

## Context

Baseline: Lattice `9a74e7ef1da4e0ca7610dce0db039b3825a7154d`.

- `Request::worker_profile_identity()` (`src/implementation_attempt/cli.rs:107-128`) is BLAKE3 over a JSON tuple of program, sandbox program, OMP profile, read and write paths, output bound, `allow_unchanged` and `worker_compute`, plus the canonical capability declaration when one is present. It is published bare (64 hex) as the inventory `revision` and the envelope's `worker_profile`.
- The inventory (`publish-worker-inventory-v2`) already publishes a definition identity that is stable across revisions and five revision slots as unknowns owned by `lattice#25`.
- The store is one redb database with private per-domain modules and version-checked transitions (`commit_orchestration_transition(expected_version, ...)`).
- Injected ports follow `bind-model-serving-admission`: a trait in the shell, an `Unavailable` implementation installed by the standalone CLI, a `Fixture` implementation for tests, and `run_command_with_ports` for embedders.

## Goals and non-goals

Goal: issue #25 sections 1 to 5 at fixture level, without changing implementation attempts, the inventory or any existing identity.

Non-goals: inventory publication, live preparation or qualification, canaries, checkpoint and resume, and the OCI and Cairn follow-ups on the issue.

## Decision 1: the profile revision is the revision identity

The lifecycle uses `worker_profile_identity()` verbatim and treats it as opaque, validated only as 64 lowercase hex. The definition is the inventory's `lattice-worker-definition-v1` identity (`blake3:<hex>`), so a new prompt, executable, tool list or route keeps the definition and changes the revision.

What the revision frames today, by section 1 component:

| Component | Framed | Where |
|---|---|---|
| Harness and executable | yes: program and sandbox paths, OMP source revision and executable pin, broker relay, shell and sandbox pins | `cli.rs:109-111`; `candidate_worker/omp.rs:16-17`; `candidate_worker/broker.rs:38-40` |
| Harness argv | no: print options are Lattice code constants | `candidate_worker/omp/profile.rs:38-55` |
| Tools and protocol | declared tool names only, through the capability (`cli.rs:120`); the exposed tool list `--tools read,edit,write` and `--mode json` are code constants (`omp/profile.rs:43,51-52`) | |
| Model route | yes: OMP model selector and the broker's schema, endpoint, model, credential reference and request bound | `omp.rs:19,21`; `broker.rs:34-37,42` |
| Model requirements | no: serving route requirements live in the per-attempt `execution_binding` (`cli.rs:91`), outside the tuple | `execution_binding/serving/mod.rs:109` |
| Memory policy | no: single-shot `--no-session` is a code constant | `omp/profile.rs:40` |
| Checkpoint semantics | no: inventory reports `unavailable` | `cli/inspect/mod.rs:98` |
| Workspace and isolation | read and write paths framed (`cli.rs:112-113`); isolation and private network are code invariants (`omp/profile.rs:5,11`) | |
| Compute | yes | `cli.rs:116` |
| Capability | yes, when declared | `cli.rs:120` |
| Producer/consumer compatibility | no: the request schema is checked (`cli.rs:137`), not framed; the worker compatibility schema is a run observation (`candidate_worker.rs:171`) | |
| Per-attempt values | framed today: workspace snapshot `base_identity`, task `prompt`, broker `expires_at_ms` | `omp.rs:18,20`; `broker.rs:41` |

The last row means the current revision changes per attempt. Stabilizing it belongs to the implementation-attempt slice; this change depends only on the value being bare hex and compares it exactly.

## Decision 2: an optional components declaration for the unframed components

`lattice.agent-revision-components.v1` (`framing.rs`) declares memory policy (`single_shot_no_session` or `session_scoped` with a schema name), checkpoint semantics (`unsupported` or `supported` with a schema name), a tool protocol with tool names, per-route model requirement references and producer/consumer compatibility names. `framing::digest(None)` is `None`; a valid declaration gives a `lattice-agent-revision-components-v1` digest with sorted tool names and requirements. A revision that frames the digest only when present keeps every existing value, as capability does. The call-site change is in Integration below.

Rejected: framing the code constants unconditionally. That would change every existing revision.

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

`projection::slots(&Lifecycle) -> Slots` (`lattice.agent-revision-slots.v1`) projects desired, prepared, qualified, last usable, selected and pinned facts, with whether each pinned session is blocked. Bundles, artifacts, holds and receipts are not projected.

## Integration

1. Components digest: add to `Request` `#[serde(default = "lattice_core::agent_revision::framing::absent", skip_serializing_if = "Option::is_none")] pub revision_components: Option<lattice_core::agent_revision::framing::Components>,` and frame `lattice_core::agent_revision::framing::digest(self.revision_components.as_ref())` only when it is `Some`.
2. Seam: before writing `intent.json`, call `seam::admit` with the attempt's session identifier and refuse on `Refused`; record `decision.admission()` with the intent; before spawn and each protected effect, call `seam::effect` with the recorded mode; after the terminal result, call `seam::close` with the recorded mode. A standalone run without a store stays unmanaged.
3. Inventory v3: fill the revision slots from `projection::slots`.

## Tests

`tests/test_agent_revision_core.rs` covers each issue criterion on the pure core, every refusal, each bound filled past its size, reissue of abandoned attempts and the committed fixture identities. `tests/test_agent_revision_shell.rs` covers resolution, persistence with an interleaved writer, the standalone CLI, the `lattice` binary's exit status, fixture ports and the seam's admission modes.

## Risks and claim boundaries

- Fixture ports are not live preparation or qualification. The evidence states `live_qualification: false`.
- Until the profile revision is stabilized, live session admission matches only requests that reproduce the exact revision.
- `effect_permitted` covers lifecycle revocation only; cancellation, deadlines, grants and budgets keep their existing owners.
