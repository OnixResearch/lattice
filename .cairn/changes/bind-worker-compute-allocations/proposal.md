# Bind worker execution to provider-owned compute allocations

Tracking: [Lattice #24](https://github.com/OnixResearch/lattice/issues/24) (compute half) and [Lattice #13](https://github.com/OnixResearch/lattice/issues/13) (compute provenance of usage observations). Producer: [Aspen/Molten #42](https://github.com/brittonr/aspen/issues/42), closed at `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4`.

## Why

`project-configured-agent-bindings` publishes material-free configured-route facts and keeps compute reservations `unsupported` because no producer schema existed. Aspen #42 now publishes versioned compute-capacity records: `compute-capacity-allocation-v2`, `compute-capacity-readback-v1` and `compute-capacity-offer-v2`. These carry consumer, operation and attempt identities, owner generation, backing-pool claims, reservation status, freshness, enforcement strength and qualification scope. Lattice can now bind an admitted worker attempt to an exact provider-owned allocation. It does this without depending on Aspen crates and without becoming the device-allocation authority.

The owner's decision of 2026-11-18 selects the compute half only. This change adds a Lattice-owned consumer DTO and an injected provider port. Serving readiness, inference request admission and model-service usage remain blocked on [Animus #17](https://github.com/OnixResearch/animus/issues/17). Live host qualification remains with [Kiln #49](https://github.com/OnixResearch/kiln/issues/49).

## What Changes

- Project versioned worker-profile facts that keep declared, qualified, unavailable and unknown support distinct. The facts reuse the existing configured-route and implementation-attempt profile owners. r[lattice.execution_binding.profile_facts]
- Add `lattice.compute-allocation-observation.v1`, a bounded consumer DTO that mirrors the published producer record labels, field order and enum spellings, with canonical `blake3:` producer record references. Validate it in the pure core. r[lattice.execution_binding.allocation_observation]
- Add a read-only injected `ComputeAllocationPort`. Lattice never reserves, activates, releases or probes. The standalone CLI's default port is explicitly unavailable, so a request that needs a provider allocation fails closed before effects. r[lattice.execution_binding.port_boundary]
- Admit an exact execution binding in a pure core. The binding covers the profile revision, the attempt and fence, a host-local or provider allocation, the placement generation, an opaque shared-serving reference and the declared model routes. A CPU-only allocation cannot satisfy a direct-accelerator worker. Hosted-API and no-model workers bind without fabricated leases. r[lattice.execution_binding.admission] r[lattice.execution_binding.identity]
- Persist the admitted binding before the implementation-attempt intent. Revalidate it after the intent and immediately before worker spawn. Stale, expired, crossed, replaced or wrong-generation allocations are rejected or blocked without dispatch. r[lattice.execution_binding.persistence]
- Keep recovery, cancellation and lost-acknowledgement handling on the existing one-shot reconciliation path. These paths never redispatch, rebind or release compute. r[lattice.execution_binding.recovery]
- Publish read-only observations of owned worker instances. Missing or stale lifecycle stays unknown, not idle or healthy. r[lattice.execution_binding.instances]
- Bind #13 command-worker usage observations to the exact binding provenance. Reserved claims, host-measured duration and unknown model or device usage stay distinct, and conservative settlement is unchanged. r[lattice.execution_binding.usage_provenance]
- Record requirement-level non-claims and the blocking owner for every deferred criterion. r[lattice.execution_binding.deferred]

## Impact

- **Pure core:** new `crates/lattice-core/src/execution_binding/` (DTO model, validation, admission, revalidation, identity and projections). The crate's admitted dependency set is unchanged.
- **Shell:** new `src/execution_binding/` (port trait, unavailable production port, deterministic fixture port, safe JSON projection). The implementation-attempt CLI gains a request-schema v3 cutover, binding persistence and revalidation steps, an `inspect` subcommand and a `run_command_with_compute_port` library entry.
- **Usage:** `CommandWorkerObservation` gains an optional frozen `execution_binding` provenance and a `Scope::Attempt` receipt, persisted through the existing immutable store path. Orchestration settlement arithmetic is untouched.
- **Configuration:** new Nickel contract `contracts/worker-execution-binding.ncl` with positive and negative fixtures for operator-authored compute requirements and binding declarations.
- **Docs:** `docs/candidate-worker.md` (v3 request and binding lifecycle) and a new `docs/worker-compute-binding.md` (producer field mapping, port contract and non-claims). The parent owns `docs/source-authoring.md`, Cargo and Nix pins. This change edits none of them.
- **Compatibility:** `lattice.implementation-attempt-run.v2` requests require explicit readmission as v3, matching the v1→v2 precedent. Existing Pi, Clankers and Animus configured routes keep reporting compute reservations as unavailable. No orchestration Agent-node compute binding is added.

## Dependencies

- Local: `project-configured-agent-bindings` (configured-route projection and `ConfiguredBinding` provenance reused here). Its open producer-adoption task is carried for compute by this change.
- Producer: Aspen `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4`, which contains `docs/compute-capacity.md`, `config/compute-capacity/{contracts,profile}.ncl` with its fixtures, `crates/molten-core/src/compute_capacity/mod.rs` and `src/compute_capacity/canonical/{allocations,readbacks,offers,enums}.rs`. Lattice reads these as a specification only and adds no Cargo, Nix or sibling-path dependency.
- Deferred owners: Animus #17 (serving readiness, request admission, per-request usage), Kiln #49 (live host/device and cross-host qualification), [Animus #18](https://github.com/OnixResearch/animus/issues/18) (qualification evidence that can promote `declared` to `qualified`), [Kiln #45](https://github.com/OnixResearch/kiln/issues/45) (aggregate budgets), [Kiln #60](https://github.com/OnixResearch/kiln/issues/60) (joined inventory presentation), [Lattice #25](https://github.com/OnixResearch/lattice/issues/25) (revision promotion/pinning) and [Lattice #12](https://github.com/OnixResearch/lattice/issues/12) (checkpoint/restore authority).

## Out of Scope

- Reserving, activating, draining or releasing compute; device discovery; host probes; Preserves decoding of producer records inside Lattice.
- Serving readiness, model-service request admission, replica routing observations, per-request token/cost provenance, pricing or currency identity.
- Compute binding for orchestration Agent nodes (Pi, Clankers, Animus, direct OpenAI-compatible routes).
- A global claim service, task queue, scheduler or another accounting ledger.
- Worker authority-lifecycle composition (Animus seal, output-release profile, checkpoint/restore authority) beyond the rule that a binding never grows authority.
