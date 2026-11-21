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
| `candidates` | Every desired revision with its bundle, preparation, occupied artifacts, cleanup status and holds |
| `qualifications` | The attachment admitted per revision and task-family scope |
| `last_usable` | The latest revision prepared for its then-current generation and not revoked |
| `selected`, `rollout_generation` | The revision for new sessions; the generation increments on each selection |
| `sessions` | Each session's pin: revision, qualification reference, rollout generation, `live` or `closed` |
| `receipts` | Applied promotions and rollbacks by request identifier |
| `revoked` | Revoked revisions, qualification references and sessions |

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
revision is never prepared again. Desiring a failed or superseded revision
again starts a new generation.

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
acknowledgement is reconciled by repeating it. Reusing an identifier for
another command is `request_conflict`. Of several requests that expect the
same rollout generation, one is selected and the rest are
`generation_conflict`. A favorable answer is evidence, never authority.

### Sessions and effects

`admit-session --request REQUEST --session ID` pins the session to the
selected revision when the request's revision equals it. `effect_permitted`
refuses further effects when the session is closed or revoked, or its pinned
revision or qualification is revoked. Pinning preserves configuration, not
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

The standalone CLI installs `UnavailablePreparation` and
`UnavailableQualification`: preparation records `unknown`, and promotion and
rollback of a prepared revision are refused as `qualification_unavailable`
with owner `animus#18`. State checks come before evidence, so a revision whose
preparation is still `unknown` is refused first as `preparation_uncertain`.
An embedder passes its own ports to `agent_revision::run_command_with_ports`.
`FixturePreparation` and `FixtureQualification` are deterministic test ports
only; the fixture qualification port answers only the exact subject it was
scripted for.

## Persistence

Lifecycles live in the existing store database, table
`agent_revision_lifecycles_v1`, one row per definition with a version.
A commit names the version it read; any other version is a conflict, and the
shell reloads and reapplies up to four times. Only `desire` creates a row.
`resolve` and `inspect` write nothing, and `inspect` does not create a missing
store.

## Integration

The session-admission seam, `agent_revision::seam`, is not wired into
`implementation-attempt` yet:

- `seam::admit(store, request, session)` answers `unmanaged` without writing
  when the request's definition has no lifecycle, so unmanaged definitions
  behave exactly as before. Otherwise it answers `admitted` with the pin or
  `refused`.
- `seam::effect(store, definition, session)` answers `unmanaged`, `permitted`
  or `blocked` before a protected effect.
- `seam::close(store, definition, session, checkpoints)` closes a session and
  holds its checkpoints.

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
