# Publish worker inventory v2 for Kiln #60

Tracking: [Lattice #24](https://github.com/OnixResearch/lattice/issues/24) (inventory facts and the unchecked "publish redacted bounded capability and wait reasons" criterion) and [Lattice #25](https://github.com/OnixResearch/lattice/issues/25) (revision lifecycle slots only). Consumer: [Kiln #60](https://github.com/OnixResearch/kiln/issues/60), whose producer reader at Kiln `4371dec083d446106bcc03ef01d6fe2d41850164` (`docs/inventory-producers.md`, `crates/kiln-adapter-lattice/src/worker_inventory.rs`) reads `lattice.worker-inventory.v1` and leaves every agent fact Lattice does not publish unknown. Owner-approved on 2026-11-20.

## Why

`implementation-attempt inspect` publishes `lattice.worker-inventory.v1` at Lattice `1bdacb35`. Kiln #60 can project only profile revisions, adapters, outputs and the retained lifecycle from it. Its consumer contract asks Lattice for four things it cannot derive:

- a definition identity separate from the profile revision, so a new revision of the same agent is recognisable;
- declared task families and tools, with "not declared" distinguishable from "declares none";
- a liveness or idle signal per instance, with an observed generation or time, that never turns missing or stale data into idle or live;
- bounded wait reasons that a Cairn workspace (Kiln #84) can show without leaking material.

Lattice #25's desired, prepared, qualified, selected and pinned revisions are not implemented. Kiln must still be able to tell "not published" from "empty".

Kiln's v1 DTOs deny unknown fields. Adding fields to v1 would make every v1 reader refuse every document, so the change is a clean schema cutover, not an additive v1 edit.

## What Changes

- `implementation-attempt inspect` emits `lattice.worker-inventory.v2` only, with `lattice.worker-profile-facts.v2` and `lattice.worker-instance-observation.v2`. `RuntimeOptions::worker_profile_facts` emits the same profile-facts v2 for configured Pi, Clankers and Animus routes. No v1 output or decoder remains. r[lattice.worker_inventory.schema]
- Every profile carries a `definition`: a BLAKE3 identity over the profile owner, the definition basis and a bounded name. The name is operator-declared, or the owner's default (`omp-attempt-worker`, or the configured target label). The definition identity is independent of the profile revision, and every instance carries its definition identity. r[lattice.worker_inventory.definition]
- An optional typed `lattice.worker-capability.v1` declaration in the existing worker configuration (`contracts/worker-execution-binding.ncl` and the implementation-attempt request) names the definition and declares `task_families` and `tools`. Each set is `declared` (1 to 32 distinct lowercase names of at most 64 bytes) or `unavailable`. An absent set stays `unknown`, never empty. A present declaration is part of the profile revision, and an absent one leaves every existing revision unchanged. Malformed declarations are refused in Nickel, in the strict decoder and before any effect. r[lattice.worker_inventory.capability]
- Every instance carries a `liveness` signal for its runtime class (`single_shot` today) with the retained-state generation and observation time. Lattice owns no reusable runtime and no heartbeat, so `idle` and `live` are never published. A dispatched attempt without a terminal observation is `unknown`, and becomes `unknown`/`stale` once its admitted window has passed. r[lattice.worker_inventory.liveness]
- Profiles and non-terminal instances carry sorted, deduplicated, bounded wait reasons from a closed vocabulary. Each reason names its code, a closed subject (a fact field or an already-published route name) and the owner that can resolve it. Reasons never carry free text, paths, endpoints, prompts, commands, credentials, grant references or backend handles. r[lattice.worker_inventory.wait_reasons]
- Profiles carry Lattice #25's desired, prepared, qualified, selected-for-new-sessions and pinned revision slots as explicit owned unknowns (`unknown`, owner `lattice#25`). r[lattice.worker_inventory.revisions]
- Deterministic v2 captures of the scenarios Kiln's v1 capture patch used are produced for Kiln's fixtures, with the exact commands. r[lattice.worker_inventory.captures]

## Impact

- **Pure core:** `crates/lattice-core/src/execution_binding/capability.rs` (new), `projection/inventory.rs`, `projection/retained.rs`, `projection/wait.rs` (new), `projection/mod.rs` and a `lattice-worker-definition-v1` role in `content_identity`.
- **Shell:** `src/implementation_attempt/cli.rs` (`worker_capability`), `cli/inspect/mod.rs`, `src/execution_binding/format.rs`, `src/agent_node/runtime/profile.rs`.
- **Contract:** `contracts/worker-execution-binding.ncl` gains an optional `worker_capability`; positive and negative Nickel fixtures.
- **Tests:** `tests/test_execution_binding_inventory.rs`, `tests/test_execution_binding_contracts.rs`, `tests/test_execution_binding_identity.rs`, `tests/test_agent_profile_bindings.rs`, `src/implementation_attempt/cli/inspect/tests.rs`, `src/implementation_attempt/cli/tests.rs`.
- **Docs:** new `docs/worker-inventory.md`; `docs/worker-compute-binding.md`, `docs/worker-serving-binding.md` and the changelog.
- **Compatibility:** breaking for inventory readers. Kiln's `kiln_adapter_lattice::worker_inventory` must move to v2; that update belongs to a separate Kiln change. Existing requests and worker configurations without `worker_capability` stay valid, keep their profile revision and report capability `unknown`.

## Dependencies

- Local: `bind-worker-compute-allocations` (profile and instance projections) and `bind-model-serving-admission` (serving readiness in inventory).
- Consumer: Kiln `4371dec0`, read-only, for the contract and the capture method.
- Deferred owners: Lattice #25 (revision lifecycle), Animus #18 (qualification evidence), Kiln #49 (live host, allocation and serving qualification, any heartbeat owner), Kiln #45 (dispatch).

## Out of Scope

- Lattice #25's revision preparation, qualification, promotion, rollback and pinning.
- A heartbeat, reusable runtime, idle pool or health probe.
- Task-to-profile matching (Lattice #26, Kiln #83), qualification attachment (Animus #18) and Kiln's reader update.
- Capability declarations for configured Pi, Clankers and Animus routes; they report `unknown`.
- Live qualification of any worker, host, provider or serving owner.
