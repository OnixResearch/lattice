# Worker inventory

`lattice implementation-attempt inspect --request REQUEST --state STATE` prints
one `lattice.worker-inventory.v2` document. It separates what a worker profile
declares from what one retained attempt shows. [Kiln #60](https://github.com/OnixResearch/kiln/issues/60)
reads it as producer observations for agent definitions and worker instances.
The change package is `.cairn/changes/publish-worker-inventory-v2/`.

Inspection is read-only. It takes no claim, schedules nothing, never calls the
compute port, and reads only the latest snapshots of an injected serving-owner
port. Listing a profile or an instance grants no execution, credential or
compute authority. A wait reason is an explanation, never a reservation or a
dispatch decision.

## Document

| Field | Schema | Meaning |
|---|---|---|
| `schema` | `lattice.worker-inventory.v2` | The envelope; the only inventory Lattice emits |
| `profile` | `lattice.worker-profile-facts.v2` | The worker profile the request admits |
| `instance` | `lattice.worker-instance-observation.v2` or `null` | The retained attempt; `null` when no execution is retained |
| `model_requests` | see [worker serving binding](worker-serving-binding.md#usage-for-13) | Per-request usage coverage, or `null` before any binding |

`RuntimeOptions::worker_profile_facts` returns the same profile facts for
configured Pi, Clankers and Animus routes. There is no v1 output or decoder.
Every v1 reader must move to v2. Kiln's v1 DTOs deny unknown fields, so they
refuse v2 documents with `lattice_inventory_unsupported_schema`, which is
intended.

## Profile facts

The v1 facts remain: adapter, protocol revision, executable identity, inputs,
outputs, sessions, checkpoint, workspace, network, isolation, deadline and
output bounds, model routes, compute and serving readiness (see
[worker compute binding](worker-compute-binding.md#profile-and-instance-facts)).
Each fact is `{ support, value }`, where `support` is `declared`, `qualified`,
`unavailable` or `unknown` and `value` is present only for declared or
qualified support. `qualified` needs a canonical `qualification_ref`, and
nothing supplies one yet ([Animus #18](https://github.com/OnixResearch/animus/issues/18)).

v2 adds:

| Field | Shape | Source |
|---|---|---|
| `definition` | `{ identity, name, basis }` | Declared name, or the owner's default |
| `task_families` | fact of a sorted name list | `worker_capability.task_families` |
| `tools` | fact of a sorted name list | `worker_capability.tools` |
| `revisions` | five owned unknowns | [Lattice #25](https://github.com/OnixResearch/lattice/issues/25), not implemented |
| `wait_reasons` | sorted list | Derived from the facts above |

### Definition

A definition is the agent a profile revision realizes. Its identity stays the
same across revisions:

```text
identity = "blake3:" + hex(content_identity::worker_definition { owner, basis, name })   // domain lattice-worker-definition-v1
```

| Owner | `basis: declared` | `basis: owner_default` name |
|---|---|---|
| `implementation_attempt` | `worker_capability.definition` | `omp-attempt-worker` |
| `configured_runtime` | — | the target label: `pi`, `clankers` or `animus` |

The revision, adapter, executable, capability sets and routes are not inputs.
A new executable, prompt, tool list or route keeps the definition and changes
the revision. The basis is framed, so a declared name that equals a default
name is still a distinct definition. Every instance carries the
`definition_identity` of its profile.

### Capability declaration

The optional `worker_capability` lives in the worker configuration authored
against [`contracts/worker-execution-binding.ncl`](../contracts/worker-execution-binding.ncl)
and in the exported implementation-attempt request:

```nickel
worker_capability = {
  schema = "lattice.worker-capability.v1",
  definition = "rust-implementer",
  task_families = { kind = "declared", names = ["rust-implementation", "cairn-task"] },
  tools = { kind = "unavailable" },
},
```

- `definition` is required whenever a declaration is present. `task_families`
  and `tools` are optional.
- A set is `{ kind = "declared", names }` with 1 to 32 distinct names, or
  `{ kind = "unavailable" }`.
- A name is 1 to 64 bytes. It starts with `[a-z0-9]` and continues with
  `[a-z0-9._:-]`, so it cannot carry a path, whitespace, command line, prompt
  or URL.
- The Nickel contract, the strict decoder (`format::load_configuration`) and
  request validation refuse anything else before any binding, intent or
  worker effect. Error messages name the defect, not the value.

| Declaration | Published fact |
|---|---|
| absent, or set absent | `{ "support": "unknown", "value": null }` |
| `declared` | `{ "support": "declared", "value": [sorted names] }` (`qualified` only with bound evidence) |
| `unavailable` | `{ "support": "unavailable", "value": null }` |

An undeclared set is never published as `[]`. A declared set is complete for
its profile, but it is a statement of support, not a qualification or a grant.

The request's profile revision frames the canonical declaration (names
sorted) only when one is present. Requests and configurations without one
keep their revision bytes and stay valid. Name order never changes a revision,
and adding a tool does.

### Revision slots

```json
"revisions": {
  "desired":                   { "support": "unknown", "owner": "lattice#25" },
  "prepared":                  { "support": "unknown", "owner": "lattice#25" },
  "qualified":                 { "support": "unknown", "owner": "lattice#25" },
  "selected_for_new_sessions": { "support": "unknown", "owner": "lattice#25" },
  "pinned":                    { "support": "unknown", "owner": "lattice#25" }
}
```

These slots are not published until Lattice #25 implements preparation,
qualification, promotion and pinning. A consumer must treat them as unknown,
never as empty or as the current revision.

## Instance observation

The v1 facts remain: profile revision, `work_id`, `attempt_id`, incarnation,
`fence`, binding identity, compute provenance, model routes, lifecycle,
cancellation, host reachability, model readiness, allocation liveness,
observation time and state digest. `work_id`, `attempt_id` and `fence` are the
admitted binding's values, unchanged. v2 adds `definition_identity`,
`liveness` and `wait_reasons`.

### Liveness

```json
"liveness": {
  "runtime": "single_shot",
  "state": "unknown",
  "unknown_cause": "unobserved",
  "retained_generation": 2,
  "observed_at_epoch_ms": 1795200625000
}
```

| Lifecycle | `state` | `unknown_cause` |
|---|---|---|
| `admitted_not_dispatched`, `binding_rejected_before_dispatch` | `not_started` | `null` |
| `terminal_observed` | `exited` | `null` |
| `dispatch_recorded_outcome_unknown`, observed within the request's `expires_at_ms` | `unknown` | `unobserved` |
| `dispatch_recorded_outcome_unknown`, observed after it | `unknown` | `stale` |

Implementation-attempt workers are single-shot. Lattice owns no reusable
runtime, idle pool or heartbeat, so `idle` and `live` are not in the
vocabulary, and missing or stale data is never reported as either.
`retained_generation` counts the retained write-once artifacts plus the
terminal result, so it never decreases for one attempt.
`observed_at_epoch_ms` is the inspecting host's clock.

## Wait reasons

```json
{ "code": "serving_snapshot_unavailable", "subject": { "kind": "route", "name": "primary" }, "owner": "animus#17" }
```

| Code | Section | Subject | Owner |
|---|---|---|---|
| `capability_undeclared` | profile | `task_families` or `tools` | `configuration` |
| `qualification_unbound` | profile | `qualification_ref` | `animus#18` |
| `revision_selection_unpublished` | profile | `revisions` | `lattice#25` |
| `serving_port_absent` | both | `serving_readiness` | `kiln#49` |
| `serving_snapshot_unavailable`, `serving_port_issue` | both | route | `animus#17` |
| `serving_snapshot_not_current` (freshness other than `current`) | both | route | `animus#17` |
| `serving_not_ready` (current snapshot, readiness other than `ready`) | both | route | `animus#17` |
| `awaiting_dispatch`, `admission_expired` | instance | `lifecycle` | `kiln#45` |
| `dispatch_outcome_unknown` | instance | `lifecycle` | `lattice#6` |
| `cancellation_unobserved` | instance | `cancellation` | `lattice#6` |
| `liveness_unobserved`, `liveness_stale` | instance | `liveness` | `kiln#49` |
| `host_reachability_unknown`, `allocation_liveness_unknown` | instance | that fact | `kiln#49` |

- Reasons are sorted by code in the order of the table above (not
  alphabetically), then by subject. They are deduplicated and bounded to 32,
  and a route gives at most one reason.
- Terminal and rejected instances carry none.
- The owner is fixed per code. It names who can resolve the reason, not who
  authorizes anything.

Disclosure: a subject is a fact field from a closed set or a route by the
name already printed in the same document's `model_routes`. Service ids stay
in readiness. Endpoints, credential and grant references, backend handles,
paths, prompts and commands never appear in a reason, and reasons carry no
free text.

## Captures for consumers

The capture tests write v2 documents only when `LATTICE_INVENTORY_CAPTURE_DIR`
is set:

```console
LATTICE_INVENTORY_CAPTURE_DIR=/path/to/dir cargo test -p lattice --lib -- \
  capture_worker_inventory_v2 capture_served_worker_inventory_v2 --exact --test-threads 1
```

Each run reproduces these scenarios: terminal failed, admitted not
dispatched, dispatched outcome unknown, a profile without an instance, a
second provider allocation (`agent-a-2`), a declared capability, and a served
route with a ready port and with a silent port. Every field is deterministic
except `observed_at_epoch_ms` and `liveness.observed_at_epoch_ms`. Captures
are synthetic fixture outputs: no worker, provider, host or serving owner is
observed.

## Non-claims

- A configured or declared profile is not a running worker and is not
  capacity.
- Declared task families and tools are operator statements. Qualification
  needs bound evidence (Animus #18), and task matching belongs to Lattice #26
  and Kiln #83.
- `unknown` liveness is not failure, and `not_started` is not a reservation.
- Revision slots publish nothing until Lattice #25.
