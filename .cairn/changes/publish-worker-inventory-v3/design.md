# Design: Worker inventory v3

## Context

Baseline: the integration of `publish-worker-inventory-v2`, `manage-agent-revisions`, `adopt-animus-qualification-attachments` and `execute-admitted-implementation-attempt` on top of Lattice `9a74e7ef`.

- `projection/inventory.rs` projects profile facts v2 with a `qualification_ref` that nothing supplies; `projection/lineage.rs` publishes five `OwnedUnknown { unknown, lattice#25 }` slots; `projection/wait.rs` has `revision_selection_unpublished`.
- `lattice_core::agent_revision::projection::slots` projects desired, prepared, qualified (with class and revocation), last usable, selected and pinned facts of one lifecycle. The admitted Animus #18 attachment records `fixture` or `live` provenance.
- With `--revision-store`, an implementation attempt admits its session (the envelope identity) through `seam::admit` and records the admission mode and pin in `intent.json` (`lattice.implementation-attempt-intent.v1`). A managed attempt closes its session after the terminal result; a session closed without a held checkpoint is forgotten by the lifecycle. A managed session blocked by `seam::effect` before spawn retains `revision-block.json`.

## Goals and non-goals

Goal: publish the lifecycle facts, the session pin and lifecycle-backed capability qualification as one clean v3 schema, without turning "not governed" into "unknown" or a missing fact into an empty one.

Non-goals: the lifecycle itself, the seam wiring, the revision framing, the Kiln reader, slots for configured routes.

## Decision 1: clean v3 cutover at every level

The envelope, profile facts and instance observation all move to `.v3`, as v1 to v2 did. Lattice keeps no v2 constant, emitter, decoder or alias. Kiln's v2 DTOs deny unknown fields and refuse v3 as an unsupported schema.

Rejected: an additive v2. The slot shape changes from `OwnedUnknown` to a tagged slot, the profile loses `qualification_ref`, and a wait-reason code is renamed; a v2 reader would fail on all of them.

## Decision 2: a three-shape slot

```text
Slot<T> = { support: "unmanaged" } | { support: "unknown", owner } | { support: "recorded", value: T }
```

`unmanaged` means Lattice's lifecycle does not govern the definition: no `--revision-store`, no lifecycle row, or a configured route. `unknown` means the definition is managed but the lifecycle holds no such fact; the owner is the one that can supply it: `lattice#25` for desired and selection (operator commands), `kiln#49` for preparation, `animus#18` for qualification. A list slot with no entries is `unknown`, never `[]`.

Rejected: reusing `OwnedUnknown` for unmanaged definitions. A consumer would wait on `lattice#25` for a definition no operator manages.

## Decision 3: four profile slots and an instance pin

`revisions = { desired, prepared, qualified, selected_for_new_sessions }` describe the definition, so they may name revisions other than the profile's. `last_usable` is not published; it is an internal fallback, not a consumer fact. `pinned` moves to the instance, since a pin belongs to one session:

```text
SessionPin = { revision, qualification_ref, rollout_generation, pin_identity, status: live | closed | released, is_blocked }
```

| Recorded admission (`intent.json`) | Lifecycle | `pinned` |
|---|---|---|
| `unmanaged`, or no admission field | any | `unmanaged` |
| none yet | none | `unmanaged` |
| none yet | present | the lifecycle's pin (`live`/`closed`), else `unknown` |
| `managed` | present | the lifecycle's pin, else the recorded pin as `released` |
| `managed` | none | `unknown` |

`released` exists because closing a single-shot session forgets its pin; the intent's recorded pin is then the only source. `is_blocked` uses `effect_permitted` for a live pin and `retained_permitted` (session, revision or qualification revoked) for a closed or released one, the same check that refuses export. A managed attempt inspected without its store is an error, never a guessed `unmanaged`.

## Decision 4: capability qualification from the lifecycle

```text
capability_qualifications = [{ task_family, qualification_ref, evidence_class, provenance }]   // sorted by task_family
```

Entries are the lifecycle's qualifications of the profile's own revision whose attachment is not revoked; none when the revision is revoked. `task_families` is `qualified` exactly when a declared set exists and every family has an entry. Provenance is published and never gates: fixture evidence qualifies, and the consumer decides whether it needs `live`. No other fact is ever `qualified`; the v2 rule that one reference qualified every fact is gone. A lifecycle of another definition is a typed defect (`FactsDefect::Lifecycle`).

Rejected: per-family support levels inside `task_families`. The fact shape stays the v2 one; `capability_qualifications` carries the per-family evidence.

## Decision 5: wait reasons and lifecycle state

`revision_not_admitted` (subject `revisions`, owner `lattice#25`) replaces `revision_selection_unpublished`. It is emitted only for a managed definition that would refuse a new session of the profile's revision: nothing or another revision selected, or the revision or the selection's qualification revoked. `qualification_unbound` names `capability_qualifications`. `revision_blocked_before_dispatch` requires an intent and a revision block and no binding refusal or terminal result; its liveness is `not_started` and it carries no reasons.

## Pure core and shell

| Pure (`lattice-core`) | Shell (`lattice`) |
|---|---|
| slot projection, session pin and status, capability qualifications, reasons, the new lifecycle state | opening the store read-only, loading the definition's lifecycle once, reading the recorded admission from `intent.json`, the revision block, printing |

## Captures for Kiln

The v2 capture tests become `capture_worker_inventory_v3` (`cli/inspect/tests.rs`) and `capture_served_worker_inventory_v3` (`cli/binding/tests/serving.rs`), all unmanaged. `capture_managed_worker_inventory_v3` (`cli/tests/revisions.rs`) writes `managed-1` to `managed-7`: desired, prepared through `FixturePreparation`, promoted with an `encode_attachment` fixture attachment, a live admitted session, a terminal attempt whose pin was released, the live session and the terminal attempt after revocation, and the profile after a refused attempt. `LATTICE_INVENTORY_CAPTURE_DIR` gates all of them.

## Tests

- Core (`tests/test_worker_inventory.rs`, `tests/test_execution_binding_inventory.rs`): unmanaged slots and no `revision_not_admitted` without a lifecycle; `unknown` owners after desire; recorded slots and `qualified` task families after promotion; revoked attachment, revoked revision and another revision qualify nothing; a foreign lifecycle is refused; the pin for every recorded admission, including `released` and blocked after revocation; liveness and reasons for every lifecycle.
- Shell (`cli/tests/revisions.rs`, `cli/inspect/tests.rs`, `cli/binding/tests/serving.rs`): the full managed scenario through the CLI with the fixture worker; managed inspection without its store refused; a blocked attempt shows `revision_blocked_before_dispatch`; v3 schema names only.
- Configured routes (`tests/test_agent_profile_bindings.rs`): profile facts v3 with unmanaged slots.

## Risks and claim boundaries

- Breaking for Kiln's reader until its separate update lands; it fails closed on the schema.
- A `qualified` task family may rest on fixture evidence. Provenance is visible; it is not live efficacy.
- Captures are synthetic fixture outputs. No live worker, preparation owner, qualification producer, host or serving owner is observed.
