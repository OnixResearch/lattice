# Agent revisions

Lattice keeps a lifecycle of immutable agent revisions per agent definition.
An operator records a desired revision, prepares it, promotes it explicitly
for new sessions, and can roll back to an earlier one. Existing sessions keep
the revision they were admitted under, but current revocation still applies
to them. This implements sections 1 to 5 of
[Lattice #25](https://github.com/OnixResearch/lattice/issues/25). The change
package is `.cairn/changes/manage-agent-revisions/`.

## Identities

| Identity | Spelling | Source |
|---|---|---|
| Definition | `blake3:<64 hex>` | The inventory definition (`lattice-worker-definition-v1`), stable across revisions |
| Revision | bare 64 lowercase hex | `Request::worker_profile_identity()`, used verbatim and compared exactly |
| Bundle | `blake3:<hex>` | `lattice-agent-bundle-v1` over the canonical `lattice.agent-bundle.v1` bytes |
| Preparation | `blake3:<hex>` | `lattice-agent-revision-preparation-v1` over definition, revision and desired generation |
| Policy | `blake3:<hex>` | `lattice-agent-revision-promotion-policy-v1` over task family and required class |
| Promotion | `blake3:<hex>` | `lattice-agent-revision-promotion-v1` over the applied selection |
| Session pin | `blake3:<hex>` | `lattice-agent-revision-session-pin-v1` over definition, session, revision, qualification and rollout generation |
| `qualification_ref` | `blake3:<64 hex>` | The attachment identity from Animus #18, used verbatim |

A new executable, prompt, tool list or model route keeps the definition and
changes the revision.

### What the revision frames

The revision frames the program and sandbox paths, the OMP source revision and
executable pin, the broker's relay, shell and sandbox pins, the model route
(schema, endpoint, model, credential reference, request bound), read and write
paths, the output bound, compute and, when declared, the capability. It does
not frame the memory policy (single-shot `--no-session`), checkpoint
semantics, the harness tool list and protocol (code constants), serving model
requirements or producer/consumer compatibility.

An optional `lattice.agent-revision-components.v1` declaration covers those:

```json
{
  "schema": "lattice.agent-revision-components.v1",
  "memory": { "kind": "single_shot_no_session" },
  "checkpoint": { "kind": "unsupported" },
  "tools": { "protocol": "omp-print-json", "names": ["edit", "read", "write"] },
  "model_requirements": [ { "route": "primary", "requirement_ref": "blake3:<64 hex>" } ],
  "compatibility": { "producer": "lattice.implementation-attempt-run.v3", "consumer": "lattice.candidate-worker-compatibility.v1" }
}
```

`lattice_core::agent_revision::framing::digest` returns `None` without a
declaration, so a revision that frames the digest only when present keeps
every existing value. The implementation-attempt request does not carry the
declaration yet; see [integration](#integration).

The current revision also frames per-attempt values (workspace snapshot, task
prompt, broker expiry). Until it is stabilized, a session is admitted only
when its request reproduces the selected revision exactly.

### Bundle

`lattice agent-revision resolve --request REQUEST` prints the
`lattice.agent-bundle.v1` bundle without writing anything: definition and
name, revision, adapter (`omp`), protocol revision, executable pin, compute,
declared task families and tools, the hosted route and the components digest.
The hosted route carries its route identity and the provider-exposed model;
`weight_identity` and `model_version` are always `unknown`, and a bundle that
claims anything else does not decode. Paths, prompts, endpoints and credential
values never appear; a credential reference is bound only through the route
identity.

## Lifecycle

`lattice.agent-revision-lifecycle.v1` keeps these facts separately:

| Fact | Meaning |
|---|---|
| `desired`, `change_generation` | The revision to prepare next; the generation increments on each change |
| `candidates` | Each desired revision not yet forgotten, with its bundle, preparation, occupied artifacts, cleanup status and holds |
| `qualifications` | The attachment admitted per revision and task-family scope |
| `last_usable` | The latest revision prepared for its then-current generation and not revoked |
| `selected`, `rollout_generation` | The revision for new sessions; the generation increments on each selection |
| `sessions` | Each live session's pin, and each closed session's pin while one of its checkpoints is held: revision, qualification reference, rollout generation, `live` or `closed`, held checkpoints |
| `receipts` | The 256 most recent applied promotions and rollbacks by request identifier |
| `revoked` | Revoked revisions, qualification references and sessions; never forgotten |

### Bounds

Every bound counts only what the lifecycle still references, so none is a
lifetime limit:

| Bound | Frees up when |
|---|---|
| 64 revisions | A new desired revision first forgets every revision that is unprepared, failed or superseded, occupies no artifact, has no hold, and is not desired, selected, last usable or pinned, with its qualifications and selection history. A revision that still occupies artifacts frees up after an observed cleanup. |
| 1024 live sessions | A session closes. A closed session is forgotten at once unless one of its checkpoints is held, and when its last held checkpoint is released. |
| 256 receipts | The oldest receipt is forgotten. Its request expected an older rollout generation, so repeating it is refused (`generation_conflict` or another state refusal), never applied again. |
| 256 qualification scopes | As for revisions. |
| 1024 revocations per kind | Never: the bound applies only to targets the lifecycle does not reference. Revoking a retained revision, a recorded or pinned qualification reference or a pinned session is never refused. |
| 128 artifacts or holds per revision | Cleanup or release. |

A forgotten closed session or revision is unknown afterwards: repeating its
close answers `unknown_session`, and session identifiers must not be reused.

### Preparation

A preparation ticket binds the definition, the revision and the desired
generation. `prepare` starts or reissues the ticket for the current desired
revision and records the preparation port's observation:

| Observation | Current generation | Older generation |
|---|---|---|
| `prepared` | `prepared`; becomes last usable | `superseded`: kept for cleanup, never promotable |
| `failed` | `failed`; last usable unchanged | `failed` |
| `unknown` | `unknown`; resources treated as occupied | `unknown` |

A pending or unknown attempt is reissued with the same ticket. A prepared
revision is never prepared again, and a revoked one never starts; a pending
or unknown attempt of a revoked revision is still reissued so it can be
settled and cleaned up. Desiring a failed or superseded revision again starts
a new generation.

An attempt bound to an older generation (its revision desired again, or no
longer desired) blocks a new start as `preparation_in_flight` or
`not_desired`. `prepare` then reissues that attempt's own ticket
(`prepare::reissue`) and records its result, which settles it as
`superseded`, `failed` or `unknown`; desiring the revision again then starts a
fresh attempt. The same result delivered twice is recorded once, including a
`prepared` result for a generation that is no longer desired.

### Promotion and rollback

```console
lattice agent-revision promote --store DB --definition DEF --revision REV \
  --qualification-ref blake3:... --task-family rust-implementation \
  --required-class task_family_evaluation --expected-generation 0 --request-id promote-1
```

Promotion requires a prepared, unrevoked revision that is not already
selected, a reference that is not revoked or bound to another revision or task
family, and the qualification port's current answer for the exact definition,
revision, task family and required class. The answer must be `qualified` with
canonical identities, the required evidence class and the same reference. The
expected rollout generation must be current. Rollback also requires a revision
that was selected before and still has the same qualification recorded for
the task family. Neither rewrites pins or history, and neither undoes external
work.

The request identifier makes both idempotent. Repeating a request returns its
receipt (`already_applied`) without selecting or preparing again, so a lost
acknowledgement is reconciled by repeating it. The receipt binds the kind,
revision, reference, policy and expected rollout generation: reusing an
identifier for any other command, including the same command with another
expected generation, is `request_conflict`. Of several requests that expect the
same rollout generation, one is selected and the rest are
`generation_conflict`. A favorable answer is evidence, never authority.

### Sessions and effects

`admit-session --request REQUEST --session ID` pins the session to the
selected revision when the request's revision equals it, and exits non-zero
when admission is refused. Re-admitting a pinned session returns its pin only
while the session is live and neither its revision nor its qualification is
revoked; a closed session is `session_closed`. `effect_permitted` refuses
further effects when the session is closed or revoked, or its pinned revision
or qualification is revoked. Pinning preserves configuration, not
privileges. Cancellation, deadlines, grants and budgets keep their existing
owners.

### Retention and cleanup

A revision's artifacts stay retained while it is desired (unless its
preparation failed), selected or last usable, while a live session pins it,
while a checkpoint or recovery evidence holds it, or while its preparation is
pending or unknown. `cleanup` asks the preparation port to release only
unretained artifacts. Only an observed `released` frees them; `failed` or
`unknown` leaves them occupied as `uncertain`, and they stay planned for a
later cleanup.

### Refusals

Every refusal leaves the lifecycle unchanged: `malformed`,
`definition_mismatch`, `unknown_revision`, `not_desired`, `not_prepared`,
`preparation_superseded`, `preparation_failed`, `preparation_uncertain`,
`preparation_in_flight`, `stale_ticket`, `result_conflict`, `revision_revoked`,
`qualification_revoked`, `session_revoked`, `qualification_unavailable`
(with `owner`), `not_qualified` (with the producer's `reason`),
`insufficient_class`, `qualification_mismatch`, `already_selected`,
`not_previously_selected`, `generation_conflict`, `request_conflict`,
`no_selection`, `revision_not_selected`, `session_conflict`,
`unknown_session`, `session_closed`, `retained`, `capacity_exhausted` and
`generation_exhausted`.

## Ports

```text
trait PreparationPort: Send {
  fn prepare(&mut self, ticket: &Ticket, bundle: &Bundle) -> Prepared;   // Prepared { artifacts } | Failed { artifacts } | Unknown
  fn cleanup(&mut self, sweep: &Sweep) -> Cleanup;                       // Released | Failed | Unknown
}
trait QualificationPort: Send {
  fn answer(&mut self, query: &Query) -> Verdict;                        // Qualified(Attachment) | NotQualified { reason } | Unavailable
}
```

The query carries the definition identity, the bare revision, the task
family, the required class (`declared_support`, `contract_conformance`,
`task_family_evaluation` or `deployment_admission`, which are independent, not
a ladder) and the shell's `now_ms` for the producer's freshness check.
`NotQualified` reasons are `subject_mismatch`, `scope_mismatch`, `stale`,
`unusable`, `insufficient_class`, `conflicting` and `malformed`.

A `Qualified` attachment carries its `subject`: the definition identity, bare
revision and task family it was produced for. The core does not trust the
port to have answered the question it was asked. It refuses an answer whose
subject names another definition or revision as `not_qualified` /
`subject_mismatch`, and another task family as `not_qualified` /
`scope_mismatch`, even when the answer names the requested reference. A
subject that is not spelled canonically is `not_qualified` / `malformed`.

The standalone CLI installs `UnavailablePreparation` and
`UnavailableQualification`: preparation records `unknown`, and promotion and
rollback of a prepared revision without attachment files are refused as
`qualification_unavailable` with owner `animus#18`. State checks come before
evidence, so a revision whose preparation is still `unknown` is refused first
as `preparation_uncertain`. An embedder passes its own ports to
`agent_revision::run_command_with_ports`. `FixturePreparation` and
`FixtureQualification` are deterministic test ports only; the fixture
qualification port answers only the exact subject it was scripted for.

### Qualification attachments

`AttachmentQualification` answers the qualification port from operator-supplied
[Animus #18 attachments](https://github.com/OnixResearch/animus/issues/18)
(`animus.qualification-attachment.v1`). The validator is
`animus_runtime::qualification::qualify` from the vendored Animus snapshot
(revision `8d6f67d38509899402df4d62e2d54321ff047f06`, recorded in
`vendor-cargo-deps/animus/source.ncl`). Lattice does not parse attachments
itself.

Operator procedure:

1. Resolve the definition identity and bare revision with
   `lattice agent-revision resolve --request REQUEST`, and have the producer
   write an attachment for exactly that subject and the task family.
2. Compute the attachment identity, `blake3:` plus the BLAKE3 hex of the
   canonical bytes (without a trailing newline). This is the
   `qualification_ref`.
3. Promote or roll back with that reference and one or more
   `--qualification-attachment PATH` arguments:

```console
lattice agent-revision promote --store DB --definition DEF --revision REV \
  --qualification-ref blake3:... --task-family rust-implementation \
  --required-class task_family_evaluation --expected-generation 0 \
  --request-id promote-1 --qualification-attachment attachment.json
```

Each file must be a regular file, not a symlink, of at most 16,384 bytes. At
most 16 files may be named. A missing, non-regular or oversized file fails the
command before any query, and nothing is written. One trailing newline is
dropped, so the producer's checked files are accepted as written. Any other
byte difference from the canonical spelling is `malformed`.

The port builds the producer's `ExpectedSubject` from the query: the
definition identity as published, the bare profile revision, the task family,
the promotion policy's required class and the shell clock's `now_ms`. It
passes no required or current components. The answer maps as follows:

| Animus answer | Lattice verdict | Command outcome |
|---|---|---|
| `Qualified` | `qualified` with `qualification_ref`, the subject and provenance label (`fixture` or `live`) Animus decoded from that attachment's bytes, class, protocol, verifier and `observed_at_ms` | Selected only when the named `--qualification-ref` equals the answer's reference (else `qualification_mismatch`) and the subject is the queried one. The provenance label is recorded with the admitted qualification and never gates promotion |
| `NotQualified(subject_mismatch)` | `not_qualified` / `subject_mismatch` | Refused |
| `NotQualified(scope_mismatch)` | `not_qualified` / `scope_mismatch` | Refused |
| `NotQualified(insufficient_class)` | `not_qualified` / `insufficient_class` | Refused |
| `NotQualified(unusable)` | `not_qualified` / `unusable` | Refused |
| `NotQualified(stale)` | `not_qualified` / `stale` | Refused |
| `NotQualified(conflicting)` | `not_qualified` / `conflicting` | Refused |
| `NotQualified(malformed)` | `not_qualified` / `malformed` | Refused |
| `Unavailable` (no attachment) | `unavailable` | `qualification_unavailable`, owner `animus#18` |

With several files, the producer collapses exact duplicates, refuses distinct
attachments that claim one source as `conflicting`, and otherwise qualifies
the most recent valid attachment. Without valid attachments it reports the
furthest-stage reason. Without any `--qualification-attachment`, the injected
port answers, so the standalone CLI still reports `qualification_unavailable`.

Non-claims:

- The only exercised producer lane is fixture evidence: Animus's scripted
  campaign-manager lane and attachments the Lattice tests encode with
  `encode_attachment`. No live producer has qualified a Lattice revision.
- A favorable attachment is evidence, not authority. Promotion and rollback
  remain separate explicit operator commands, and revocation of the reference,
  the revision or a session still applies afterwards.
- A qualified answer does not claim live efficacy, deployment readiness or
  correctness beyond the attachment's own scope and limitations.

## Persistence

Lifecycles live in the existing store database, table
`agent_revision_lifecycles_v1`, one row per definition with a version.
A commit names the version it read; any other version is a conflict, and the
shell reloads and reapplies up to four times. Embedders apply their own pure
transition the same way with `agent_revision::transact`. Only `desire`
creates a row. `resolve` and `inspect` write nothing, and `inspect` does not
create a missing store.

## Integration

The session-admission seam, `agent_revision::seam`, is not wired into
`implementation-attempt` yet. Its contract for the integrator:

1. Before writing `intent.json`, call
   `seam::admit(store, request, session) -> Result<Decision, String>` once
   with the attempt's session identifier, which must be unique and never
   reused.
   - `Decision::Unmanaged`: the definition has no lifecycle. Nothing was
     written; proceed exactly as today.
   - `Decision::Admitted { admitted }`: the session is pinned to the selected
     revision (`pinned`, or `already_pinned` for a retried admission).
   - `Decision::Refused { refusal }`: start nothing: no binding, intent or
     worker effect. A managed definition refuses every new session until a
     revision is selected (`no_selection`), and refuses a request whose
     revision is not the selection (`revision_not_selected`).
   - `Err(String)`: the store or request failed; start nothing.
2. Record `decision.admission()` durably with the session, in the same write
   as the intent: `Admission::Managed` (`"managed"`) for `admitted`,
   `Admission::Unmanaged` (`"unmanaged"`) for `unmanaged`. It is `None` for
   `refused`. Reconciliation after a restart reads it back; never derive it
   from whether a lifecycle exists now.
3. Before each protected effect, call
   `seam::effect(store, request, session, admission) -> Result<Effect, String>`
   with the recorded mode and proceed only on `Effect::Unmanaged` or
   `Effect::Permitted { permit }`.
   - `Unmanaged` sessions answer `unmanaged` without reading the store, even
     after their definition becomes managed. They were admitted outside the
     lifecycle, so lifecycle revocation does not reach them; they finish
     exactly as they would today.
   - `Managed` sessions fail closed: `Effect::Blocked { refusal }` for a
     closed or revoked session, a revoked pinned revision or qualification,
     and `unknown_session` when the lifecycle or the pin is missing.
   - `Err(String)` is not a permit.
4. After the terminal result, call
   `seam::close(store, request, session, admission, checkpoints) -> Result<Closure, String>`.
   `Unmanaged` answers `Closure::Unmanaged` and writes nothing. `Managed`
   answers `Closure::Closed { closed }` and holds the checkpoints on the
   pinned revision, or `Closure::Refused { refusal }`; a session closed
   without a held checkpoint is forgotten, so repeating its close answers
   `unknown_session`.

`lattice_core::agent_revision::projection::slots` projects
`lattice.agent-revision-slots.v1` for a later inventory schema. The inventory
still publishes the five revision slots as unknowns owned by `lattice#25`.

## Owners and non-claims

| Capability | Owner |
|---|---|
| Qualification attachments | [Animus #18](https://github.com/OnixResearch/animus/issues/18) |
| Runtime preparation, cleanup and live qualification | [Kiln #49](https://github.com/OnixResearch/kiln/issues/49) |
| Operator presentation of rollout observations | [Kiln #60](https://github.com/OnixResearch/kiln/issues/60) |
| Checkpoint and resume | [Lattice #12](https://github.com/OnixResearch/lattice/issues/12) |

- No live preparation, qualification, canary or rollout has been exercised.
  The retained evidence is fixture-only and states `live_qualification: false`.
- A qualification answer is not promotion authority, and startup success is
  not an efficacy verdict.
- A pin is not a grant. This lifecycle does not make sessions migratable or
  resumable into another revision.
- Hosted model routes report what the provider exposes; an endpoint or model
  name is not a verified weight identity.
