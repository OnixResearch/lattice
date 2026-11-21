# Publish worker inventory v3 for Kiln #60

Tracking: [Lattice #25](https://github.com/OnixResearch/lattice/issues/25) section 6 (inventory publication of the revision lifecycle) and [Lattice #24](https://github.com/OnixResearch/lattice/issues/24) (qualified capability). Consumer: [Kiln #60](https://github.com/OnixResearch/kiln/issues/60), whose reader is being moved to v2 and must move to v3. Owner-approved on 2026-11-20.

## Why

`lattice.worker-inventory.v2` (`publish-worker-inventory-v2`) publishes the five #25 revision slots as unknowns owned by `lattice#25` and a profile-level `qualification_ref` that nothing supplies. `manage-agent-revisions` now keeps the lifecycle, `adopt-animus-qualification-attachments` records Animus #18 attachments with `fixture`/`live` provenance, and the implementation-attempt seam records each attempt's admission mode and pin. Kiln still cannot see any of it, and cannot tell a definition Lattice's lifecycle does not govern from a managed one whose fact is missing.

The profile revision also changed meaning: v2 documents carried a per-attempt revision (base snapshot, task prompt and broker expiry were framed); it now frames profile-level configuration and the reviewed harness template only. Kiln's v2 DTOs deny unknown fields, so new fields and changed slot shapes need a clean schema cutover, as v1 to v2 did.

## What Changes

- `implementation-attempt inspect` emits `lattice.worker-inventory.v3` only, with `lattice.worker-profile-facts.v3` and `lattice.worker-instance-observation.v3`; `RuntimeOptions::worker_profile_facts` emits profile facts v3. No v1 or v2 output or decoder remains, and the top-level `qualification_ref` is removed. r[lattice.worker_inventory.schema]
- `inspect` takes an optional `--revision-store`. Profiles carry four revision slots (`desired`, `prepared`, `qualified`, `selected_for_new_sessions`) and instances a `pinned` slot. Each slot is `unmanaged` (no store or no lifecycle), `unknown` with its owner (`lattice#25`, `kiln#49` for preparation, `animus#18` for qualification) or `recorded`. The pin follows the admission mode recorded in the attempt's intent. r[lattice.worker_inventory.revisions]
- Profiles carry `capability_qualifications` from current lifecycle qualifications of their own revision, with the attachment identity, evidence class and provenance. `task_families` becomes `qualified` only when every declared family has one. r[lattice.worker_inventory.qualification]
- Wait reasons replace `revision_selection_unpublished` with `revision_not_admitted`, and `qualification_unbound` names `capability_qualifications`. r[lattice.worker_inventory.wait_reasons]
- Instances gain the `revision_blocked_before_dispatch` lifecycle state with `not_started` liveness. r[lattice.worker_inventory.liveness]
- v3 captures of the v2 scenarios plus a managed lifecycle are produced for Kiln's fixtures, with the exact commands. r[lattice.worker_inventory.captures]

## Impact

- **Pure core:** `crates/lattice-core/src/execution_binding/projection/{inventory,lineage,retained,wait}.rs` and the new private `projection/attempt.rs` (instance lifecycle, cancellation and liveness); `agent_revision/projection.rs` (provenance, `RecordedPin`, `SessionPin`, `session_pin`).
- **Shell:** `src/implementation_attempt/cli/inspect/mod.rs`, `src/implementation_attempt/cli/session/intent.rs` and `src/implementation_attempt/cli/session/revision.rs` (recorded admission), `src/agent_node/runtime/profile.rs`.
- **Tests:** `tests/test_worker_inventory.rs`, `tests/test_execution_binding_inventory.rs`, `tests/test_agent_profile_bindings.rs`, `src/implementation_attempt/cli/inspect/tests.rs`, `src/implementation_attempt/cli/binding/tests/serving.rs`, `src/implementation_attempt/cli/tests/revisions.rs`.
- **Docs:** `docs/worker-inventory.md`, `docs/agent-revisions.md` and the changelog.
- **Compatibility:** breaking for inventory readers; Kiln's `worker_inventory` reader must move to v3 in a separate Kiln change. Every profile revision value differs from v2 because of the revision framing change, which is not part of this change.

## Dependencies

- Local: `publish-worker-inventory-v2` (whose schema, revisions, liveness, wait-reason and capture requirements this change modifies), `manage-agent-revisions` (lifecycle, slots projection, seam wiring) and `adopt-animus-qualification-attachments` (provenance).
- Consumer: Kiln #60 reader update, separate.
- Deferred owners: Kiln #49 (live preparation), Animus #18 (live qualification).

## Out of Scope

- The profile revision framing and the seam wiring themselves (`manage-agent-revisions`, `execute-admitted-implementation-attempt`).
- Revision slots for configured Pi, Clankers and Animus routes; they stay `unmanaged`.
- Task-to-profile matching (Lattice #26, Kiln #83) and Kiln's reader update.
- Live preparation or qualification of any worker.
