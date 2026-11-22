# Worker inventory

`lattice implementation-attempt inspect --request REQUEST --state STATE
[--revision-store DB]` prints one `lattice.worker-inventory.v3` document. It
separates what a worker profile declares from what one retained attempt shows,
and with a revision store it publishes the definition's
[agent revision lifecycle](agent-revisions.md).
[Kiln #60](https://github.com/OnixResearch/kiln/issues/60) reads it as
producer observations for agent definitions and worker instances. The change
package is `.cairn/changes/publish-worker-inventory-v3/`, which supersedes
`publish-worker-inventory-v2`.

Inspection is read-only. It takes no claim, schedules nothing, never calls the
compute port, and reads only the latest snapshots of an injected serving-owner
port. With `--revision-store` it reads the definition's lifecycle and writes
nothing; a named store that does not exist is an error. Listing a profile or
an instance grants no execution, credential or compute authority. A wait
reason is an explanation, never a reservation or a dispatch decision.

## Document

| Field | Schema | Meaning |
|---|---|---|
| `schema` | `lattice.worker-inventory.v3` | The envelope; the only inventory Lattice emits |
| `profile` | `lattice.worker-profile-facts.v3` | The worker profile the request admits |
| `instance` | `lattice.worker-instance-observation.v3` or `null` | The retained attempt; `null` when no execution is retained |
| `model_requests` | see [worker serving binding](worker-serving-binding.md#usage-for-13) | Per-request usage coverage, or `null` before any binding |

`RuntimeOptions::worker_profile_facts` returns the same v3 profile facts for
configured Pi, Clankers and Animus routes. Configured routes have no revision
lifecycle, so every revision slot is `unmanaged`.

### Cutover

There is no v1 or v2 output and no v1 or v2 decoder, the same cutover rule as
v1 to v2. Every v2 reader must move to v3. Kiln's v2 DTOs deny unknown fields,
so they refuse v3 documents as an unsupported schema, which is intended. The
v2 top-level `qualification_ref`, the five owned-unknown revision slots and
the `revision_selection_unpublished` wait reason no longer exist.

## Profile facts

The v1 facts remain: adapter, protocol revision, executable identity, inputs,
outputs, sessions, checkpoint, workspace, network, isolation, deadline and
output bounds, model routes, compute and serving readiness (see
[worker compute binding](worker-compute-binding.md#profile-and-instance-facts)).
Each fact is `{ support, value }`, where `support` is `declared`, `qualified`,
`unavailable` or `unknown` and `value` is present only for declared or
qualified support. Only `task_families` can be `qualified`
([capability qualifications](#capability-qualifications)); every other fact
is `declared`, `unavailable` or `unknown`.

| Field | Shape | Source |
|---|---|---|
| `definition` | `{ identity, name, basis }` | Declared name, or the owner's default |
| `task_families` | fact of a sorted name list | `worker_capability.task_families`, `qualified` per [capability qualifications](#capability-qualifications) |
| `tools` | fact of a sorted name list | `worker_capability.tools` |
| `capability_qualifications` | list | Current lifecycle qualifications of this profile's revision |
| `revisions` | four [revision slots](#revision-slots) | The definition's agent revision lifecycle |
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
A new executable, reviewed harness flag, tool list or route keeps the
definition and changes the revision. The basis is framed, so a declared name
that equals a default name is still a distinct definition. Every instance
carries the `definition_identity` of its profile.

### Revision

`profile.revision` and `instance.profile_revision` are the
`lattice-worker-profile-revision-v2` identity, spelled as bare hex. It frames
profile-level configuration and the reviewed harness template: the OMP launch
prefix `launch --cwd /workspace` and the print options the shell passes to the
worker, in argv order, and the request schema. Changing any reviewed flag
(tools, mode, `--no-session`, approval mode) is a new revision. Per-attempt
task prompts, base snapshots, the workspace root and broker expiry do not
change it; the exact request bytes bind them. See
[what the revision frames](agent-revisions.md#what-the-revision-frames).

v2 inventory documents carried the earlier framing, which also hashed the base
snapshot, task prompt and broker expiry, so each attempt had its own
revision. Never compare a v2 revision with a v3 one.

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
| `declared` | `{ "support": "declared", "value": [sorted names] }`, or `qualified` for `task_families` per [capability qualifications](#capability-qualifications) |
| `unavailable` | `{ "support": "unavailable", "value": null }` |

An undeclared set is never published as `[]`. A declared set is complete for
its profile, but it is a statement of support, not a qualification or a grant.

The profile revision frames the canonical declaration (names sorted) only
when one is present. Name order never changes a revision, and adding a tool
does.

### Capability qualifications

```json
"capability_qualifications": [
  { "task_family": "rust-implementation", "qualification_ref": "blake3:<64 hex>",
    "evidence_class": "task_family_evaluation", "provenance": "fixture" }
]
```

`capability_qualifications` lists, sorted by task family, every qualification
the definition's lifecycle records for this profile's own `revision` whose
attachment is not revoked. It is empty when the revision itself is revoked,
when no revision store is supplied, and for a definition without a lifecycle.
`qualification_ref` is the [Animus #18](https://github.com/OnixResearch/animus/issues/18)
attachment identity, verbatim. `evidence_class` is the class the promotion
policy required. `provenance` is `fixture` or `live`, as the producer labelled
the attachment; it is published so a consumer can tell fixture evidence
apart, and it never gates anything.

`task_families` becomes `qualified`, with the same sorted names as value, only
when the profile declares a `task_families` set and every declared family has
an entry in `capability_qualifications`. A fixture-provenance qualification
counts, so a consumer that needs live evidence must read `provenance`. One
missing family leaves the whole set `declared`. `tools` is never qualified.

### Revision slots

Every revision slot, in the profile and in the instance, is one of three
shapes:

| Shape | Meaning |
|---|---|
| `{ "support": "unmanaged" }` | Lattice's revision lifecycle does not govern this: `inspect` ran without `--revision-store`, the store holds no lifecycle for the definition, or (for `pinned` only) the attempt was admitted unmanaged. No fact is missing; there is none to publish |
| `{ "support": "unknown", "owner": OWNER }` | The definition is lifecycle-managed, but the lifecycle holds no fact for this slot yet. `owner` names who can supply it |
| `{ "support": "recorded", "value": VALUE }` | The lifecycle's fact |

`unmanaged` is distinct from `unknown`, and neither is an empty value or the
current revision. A definition becomes managed when an operator first runs
`agent-revision desire` for it.

The four profile slots describe the definition, so they may name revisions
other than `profile.revision`:

| Slot | Recorded value | Owner when `unknown` |
|---|---|---|
| `desired` | `{ revision, generation }`: the revision to prepare next and its change generation | `lattice#25` |
| `prepared` | Sorted bare revisions prepared for the desired generation they served; never empty | `kiln#49` |
| `qualified` | `[{ revision, task_family, qualification_ref, evidence_class, provenance, is_revoked }]`, sorted by revision and task family, revoked attachments included with `is_revoked: true`; never empty | `animus#18` (no attachment recorded) |
| `selected_for_new_sessions` | `{ revision, rollout_generation, kind, qualification_ref }`, `kind` `promotion` or `rollback` | `lattice#25` |

The instance carries the fifth slot, `pinned`, for its own session (the
attempt's envelope identity):

| Recorded `status` | Meaning |
|---|---|
| `live` | The lifecycle holds the live pin. `is_blocked` means a further effect would be refused: the session is closed or revoked, or its pinned revision or qualification is revoked |
| `closed` | The session closed and a held checkpoint keeps its pin. Single-shot attempts hold no checkpoints, so implementation attempts do not reach it |
| `released` | The session closed and the lifecycle forgot its pin; the value is the pin the attempt's `intent.json` recorded at admission |

For `closed` and `released`, `is_blocked` means the attempt's retained output
may no longer be exported: its session, pinned revision or qualification has
been revoked since. The recorded value is `{ revision, qualification_ref,
rollout_generation, pin_identity, status, is_blocked }`.

`pinned` follows the admission mode recorded in `intent.json`, never whether
the definition is managed now:

| Recorded admission | `pinned` |
|---|---|
| `unmanaged`, or an intent without an admission field | `unmanaged`, even after the definition becomes managed |
| `managed` | `recorded`; `unknown` (`lattice#25`) only when the store holds no lifecycle |
| no intent yet, without a store or lifecycle | `unmanaged` |
| no intent yet, with a lifecycle | `recorded` when the lifecycle pins the session, else `unknown` (`lattice#25`) |

Inspecting an attempt whose intent records `managed` without
`--revision-store` is an error that names the flag, never a guessed state.

## Instance observation

The v1 facts remain: profile revision, `work_id`, `attempt_id`, incarnation,
`fence`, binding identity, compute provenance, model routes, lifecycle,
cancellation, host reachability, model readiness, allocation liveness,
observation time and state digest. `work_id`, `attempt_id` and `fence` are the
admitted binding's values, unchanged. v2 added `definition_identity`,
`liveness` and `wait_reasons`; v3 adds `pinned` and the
`revision_blocked_before_dispatch` lifecycle state.

| Lifecycle `state` | Retained state |
|---|---|
| `admitted_not_dispatched` | Binding only, no intent |
| `binding_rejected_before_dispatch` | Intent and binding refusal |
| `revision_blocked_before_dispatch` | Intent and `revision-block.json`: the managed session's effect check (`seam::effect`) was blocked right before spawn, so the worker never started |
| `dispatch_recorded_outcome_unknown` | Intent without a terminal result: no published native result, or, for an attempt with a repair policy, no retained repair disposition whose complete records prove every started worker stopped |
| `terminal_observed` | Published terminal result, with its `disposition`; for an attempt with a repair policy, a retained repair disposition whose complete records prove every started worker stopped |

`disposition` is a token. A single-shot attempt publishes its native
candidate disposition (`produced`, `failed`, `blocked`, `cancelled` or
`unknown`). An attempt with a repair policy has no native result. Its
`disposition` is the `kind` of its retained v2 `repair/disposition.json` only
when the complete record set of the exact request (the typed disposition
under this envelope, execution binding, policy, lineage and dispatch plan,
and the typed intent and record of every step it counts) agrees with that
kind and shows every started worker `exited` or `cancelled`. Then the kind is
one of `correct_candidate`, `repaired_candidate`, `budget_exhausted`,
`cancelled`, `uncertain` (`check_outcome` or `clock`), `not_repairable`,
`checks_rejected`, `lineage_blocked`, `kiln_owned`, `fence_superseded` or
`authority_withdrawn`. `uncertain` with cause `worker_outcome` (an unknown
worker outcome, including an unknown teardown) never is: it stays
`dispatch_recorded_outcome_unknown`, as does any step intent without a record
other than the one `budget_exhausted` (`elapsed`) or `uncertain` (`clock`)
leaves before its worker is called, and a first-schema (v1) disposition whose
request identities, envelope, policy, execution binding and step count name
this attempt. Only the kind is published; the revision, repair count, bound,
cause and defect stay in the retained record, which `recover` prints. A repair
`cancelled` is observed termination like a single-shot one. A malformed,
crossed or contradicted record set is an error, never a guessed state; for a
first-schema record `recover` refuses it with the same reason. This is the
existing v3 shape: Kiln's v3 reader at `4899d149` decodes `disposition` as an
open string, so no schema or enumeration changes
([candidate worker](candidate-worker.md#exporting-repaired-candidates)).

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
| `admitted_not_dispatched`, `binding_rejected_before_dispatch`, `revision_blocked_before_dispatch` | `not_started` | `null` |
| `terminal_observed` | `exited` | `null` |
| `dispatch_recorded_outcome_unknown`, observed within the request's `expires_at_ms` | `unknown` | `unobserved` |
| `dispatch_recorded_outcome_unknown`, observed after it | `unknown` | `stale` |

Implementation-attempt workers are single-shot. Lattice owns no reusable
runtime, idle pool or heartbeat, so `idle` and `live` are not in the liveness
vocabulary, and missing or stale data is never reported as either. (`live` as
a `pinned` status describes the session's pin, not the worker process.)
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
| `qualification_unbound` (`task_families` is not `qualified`) | profile | `capability_qualifications` | `animus#18` |
| `revision_not_admitted` | profile | `revisions` | `lattice#25` |
| `serving_port_absent` | both | `serving_readiness` | `kiln#49` |
| `serving_snapshot_unavailable`, `serving_port_issue` | both | route | `animus#17` |
| `serving_snapshot_not_current` (freshness other than `current`) | both | route | `animus#17` |
| `serving_not_ready` (current snapshot, readiness other than `ready`) | both | route | `animus#17` |
| `awaiting_dispatch`, `admission_expired` | instance | `lifecycle` | `kiln#45` |
| `dispatch_outcome_unknown` | instance | `lifecycle` | `lattice#6` |
| `cancellation_unobserved` | instance | `cancellation` | `lattice#6` |
| `liveness_unobserved`, `liveness_stale` | instance | `liveness` | `kiln#49` |
| `host_reachability_unknown`, `allocation_liveness_unknown` | instance | that fact | `kiln#49` |

`revision_not_admitted` appears only for a lifecycle-managed definition that
would refuse a new session of this profile's revision: nothing or another
revision is selected, or this revision or the selection's qualification is
revoked. An unmanaged profile never carries it.

A Kiln budget lineage blocker, serving backpressure or exhaustion and a
missing repair port are not wait reasons. Lattice refuses such a request
before its fence record and intent, writing only the attempt-fence's closure
marker (`lineage_blocked`, `lineage_exhausted`,
`repair_ports_unavailable`), and Kiln owns waiting for its lineage. A repair
loop's exhausted bound is a terminal disposition, not a wait. An attempt run
with a repair policy stays `dispatch_recorded_outcome_unknown` with the
`dispatch_outcome_unknown` reason until its complete records prove its
workers stopped; from then on it is `terminal_observed` with no wait reason.

- Reasons are sorted by code in the order of the table above (not
  alphabetically), then by subject. They are deduplicated and bounded to 32,
  and a route gives at most one reason.
- Terminal, rejected and revision-blocked instances carry none.
- The owner is fixed per code. It names who can resolve the reason, not who
  authorizes anything.

Disclosure: a subject is a fact field from a closed set or a route by the
name already printed in the same document's `model_routes`. Service ids stay
in readiness. Endpoints, credential and grant references, backend handles,
paths, prompts and commands never appear in a reason, and reasons carry no
free text.

## Examples

Excerpts from the [captures](#captures-for-consumers), with fields and values
copied verbatim. Facts that v3 leaves unchanged (adapter, routes, compute,
serving readiness, binding and similar) are omitted.

Managed: `managed-5-terminal-pin-released.json`. The declared
`rust-implementer` definition is desired, prepared through the fixture
preparation port and promoted with an Animus #18 attachment of `fixture`
provenance for `rust-implementation`. Its attempt reached a terminal result,
which closed its session, so the pin comes from the intent and is `released`.
The inspection has no serving-owner port, hence `serving_port_absent`:

```json
{
  "schema": "lattice.worker-inventory.v3",
  "profile": {
    "schema": "lattice.worker-profile-facts.v3",
    "owner": "implementation_attempt",
    "definition": {
      "basis": "declared",
      "identity": "blake3:367783379a41fd2e834f26af4e7504906c1f3967bd9f21bba979441a1f0789c1",
      "name": "rust-implementer"
    },
    "revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46",
    "task_families": { "support": "qualified", "value": ["rust-implementation"] },
    "tools": { "support": "declared", "value": ["edit", "read", "write"] },
    "capability_qualifications": [
      {
        "evidence_class": "task_family_evaluation",
        "provenance": "fixture",
        "qualification_ref": "blake3:f87cdbc0556abfd2bcf7bdd9d26e89b722a57bf5b6b1a81d0ea73fc5c1c41fcc",
        "task_family": "rust-implementation"
      }
    ],
    "revisions": {
      "desired": {
        "support": "recorded",
        "value": { "generation": 1, "revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46" }
      },
      "prepared": {
        "support": "recorded",
        "value": ["96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46"]
      },
      "qualified": {
        "support": "recorded",
        "value": [
          {
            "evidence_class": "task_family_evaluation",
            "is_revoked": false,
            "provenance": "fixture",
            "qualification_ref": "blake3:f87cdbc0556abfd2bcf7bdd9d26e89b722a57bf5b6b1a81d0ea73fc5c1c41fcc",
            "revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46",
            "task_family": "rust-implementation"
          }
        ]
      },
      "selected_for_new_sessions": {
        "support": "recorded",
        "value": {
          "kind": "promotion",
          "qualification_ref": "blake3:f87cdbc0556abfd2bcf7bdd9d26e89b722a57bf5b6b1a81d0ea73fc5c1c41fcc",
          "revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46",
          "rollout_generation": 1
        }
      }
    },
    "wait_reasons": [
      { "code": "serving_port_absent", "owner": "kiln#49", "subject": { "field": "serving_readiness", "kind": "fact" } }
    ]
  },
  "instance": {
    "schema": "lattice.worker-instance-observation.v3",
    "definition_identity": "blake3:367783379a41fd2e834f26af4e7504906c1f3967bd9f21bba979441a1f0789c1",
    "profile_revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46",
    "pinned": {
      "support": "recorded",
      "value": {
        "is_blocked": false,
        "pin_identity": "blake3:32be75ce1baa21850b62b00d3cfa4d9f082403a3e7cbba8b8c4cf55be8e7c0b6",
        "qualification_ref": "blake3:f87cdbc0556abfd2bcf7bdd9d26e89b722a57bf5b6b1a81d0ea73fc5c1c41fcc",
        "revision": "96737fddb3dd1ef86fb1916f1731aa65ca70455506f4a08c984ce04380349f46",
        "rollout_generation": 1,
        "status": "released"
      }
    },
    "lifecycle": { "disposition": "blocked", "state": "terminal_observed" },
    "liveness": {
      "observed_at_epoch_ms": 1795237881949,
      "retained_generation": 5,
      "runtime": "single_shot",
      "state": "exited",
      "unknown_cause": null
    },
    "wait_reasons": []
  }
}
```

In `managed-4-admitted-session-live.json`, a session admitted and bound before
its intent, the instance is `admitted_not_dispatched` and its `pinned` value
has `"status": "live"`. After `agent-revision revoke --revision` of that
revision (`managed-6-revoked-terminal.json`), the same attempt has
`"capability_qualifications": []`, `task_families` back to `declared`, the
`qualified` entry still listed with `"is_revoked": false` (the attachment
itself was not revoked), profile reasons `qualification_unbound`,
`revision_not_admitted` and `serving_port_absent`, and
`"is_blocked": true` on the released pin.

Unmanaged: `agent-a-1-terminal-failed.json`, inspected without
`--revision-store`, with no capability declaration and a provider allocation
whose terminal result is `failed`:

```json
{
  "schema": "lattice.worker-inventory.v3",
  "profile": {
    "schema": "lattice.worker-profile-facts.v3",
    "owner": "implementation_attempt",
    "definition": {
      "basis": "owner_default",
      "identity": "blake3:c2edbc9f0aa03386b1e8d4eb900e2b10db1083ed568ed934c53e923a2e23a8b6",
      "name": "omp-attempt-worker"
    },
    "revision": "7a2e02cbb387e5e5744d4a63084238c2b6cb58d8572ad048fadcd38887ec1258",
    "task_families": { "support": "unknown", "value": null },
    "tools": { "support": "unknown", "value": null },
    "capability_qualifications": [],
    "revisions": {
      "desired": { "support": "unmanaged" },
      "prepared": { "support": "unmanaged" },
      "qualified": { "support": "unmanaged" },
      "selected_for_new_sessions": { "support": "unmanaged" }
    },
    "wait_reasons": [
      { "code": "capability_undeclared", "owner": "configuration", "subject": { "field": "task_families", "kind": "fact" } },
      { "code": "capability_undeclared", "owner": "configuration", "subject": { "field": "tools", "kind": "fact" } },
      { "code": "qualification_unbound", "owner": "animus#18", "subject": { "field": "capability_qualifications", "kind": "fact" } },
      { "code": "serving_port_absent", "owner": "kiln#49", "subject": { "field": "serving_readiness", "kind": "fact" } }
    ]
  },
  "instance": {
    "schema": "lattice.worker-instance-observation.v3",
    "definition_identity": "blake3:c2edbc9f0aa03386b1e8d4eb900e2b10db1083ed568ed934c53e923a2e23a8b6",
    "profile_revision": "7a2e02cbb387e5e5744d4a63084238c2b6cb58d8572ad048fadcd38887ec1258",
    "pinned": { "support": "unmanaged" },
    "lifecycle": { "disposition": "failed", "state": "terminal_observed" },
    "liveness": {
      "observed_at_epoch_ms": 1795237881828,
      "retained_generation": 3,
      "runtime": "single_shot",
      "state": "exited",
      "unknown_cause": null
    },
    "wait_reasons": []
  }
}
```

## Captures for consumers

The capture tests write v3 documents only when `LATTICE_INVENTORY_CAPTURE_DIR`
names an existing directory:

```console
LATTICE_INVENTORY_CAPTURE_DIR=/path/to/dir cargo test --locked -p lattice --lib -- \
  capture_ --test-threads 1
```

The filter matches `capture_worker_inventory_v3`,
`capture_served_worker_inventory_v3` and `capture_managed_worker_inventory_v3`
(and two unrelated tests whose names contain `capture_`).
Each run reproduces the v2 scenarios (terminal failed, admitted not
dispatched, dispatched outcome unknown, a profile without an instance, a
second provider allocation (`agent-a-2`), a declared capability, and a served
route with a ready port and with a silent port), all unmanaged, plus a managed
lifecycle: desired, prepared through the fixture preparation port, promoted
with an Animus #18 fixture attachment, a live admitted session, a terminal
attempt whose pin was released, both after revocation of the revision, and the
profile after an attempt refused before its intent, whose closed attempt has
no instance (`managed-1` to `managed-7`). Every field is deterministic except `observed_at_epoch_ms` and
`liveness.observed_at_epoch_ms`, and, in the managed scenarios, the instance's
`incarnation` and `state_digest`, whose requests name a per-run temporary
workspace. The capture directory's `README.md` lists the exact command, every
scenario and the BLAKE3 of each file. Captures are synthetic fixture outputs:
no worker, provider, host, preparation owner, qualification producer or
serving owner is observed.

## Non-claims

- A configured or declared profile is not a running worker and is not
  capacity.
- Declared task families and tools are operator statements. A `qualified`
  task family rests on recorded attachments whose provenance may be
  `fixture`; it is not live efficacy, and task matching belongs to Lattice #26
  and Kiln #83.
- `unknown` liveness is not failure, and `not_started` is not a reservation.
- A recorded revision slot reports the lifecycle; it is not promotion,
  admission or export authority. `unmanaged` does not mean unqualified or
  unsafe, only that Lattice's lifecycle does not govern the definition.
