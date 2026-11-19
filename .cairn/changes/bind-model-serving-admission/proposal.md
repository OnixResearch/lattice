# Bind brokered model requests to serving-owner admission

Tracking: [Lattice #24](https://github.com/OnixResearch/lattice/issues/24) (serving half) and [Lattice #13](https://github.com/OnixResearch/lattice/issues/13) (per-request usage bindings). Producer: [Animus #17](https://github.com/OnixResearch/animus/issues/17), candidate head `1c370d5029a2667051c46bdfa1919e78e65e2f4b` (local, unpublished at authoring time).

## Why

`bind-worker-compute-allocations` binds an implementation attempt to a host-local or provider compute allocation and its named model routes. It left serving readiness, inference request admission and per-request model usage `unknown` with owner `animus#17`. Animus #17 now publishes a pure `no_std` serving core in `animus_adapters::serving`: availability snapshots, a capacity-aware profile, route screening, a request-scoped serving-owner gate and a metadata-only invocation receipt. Lattice already vendors `animus-adapters`, so it can run that gate itself, per request, inside its attempt-local model broker. It does this without becoming the serving owner and without treating a capacity snapshot as a reservation.

## What Changes

- Re-vendor Animus at `1c370d50` through `scripts/sync_animus.rs`, adding `animus-frozen-policy` to the shared vendored crate set. Lattice consumes only the pure `animus_adapters::serving` core; the `animus-std` `serving-admission` host feature is not vendored or enabled. r[lattice.serving_binding.producer]
- Add an optional `lattice.serving-binding.v1` declaration to the per-attempt execution binding: the serving profile, privacy band, attribution policy and, for each served named route, the logical service, endpoint identity, exact model and configuration and request requirements. It is validated before effects and frozen into the admitted binding and its identity. Without it, every existing binding keeps its identity and behaviour. r[lattice.serving_binding.declaration]
- Add a read-only injected `ServingOwnerPort` (snapshot, request admission, current recheck). The standalone CLI installs `UnavailableServingPort`, so a served route blocks before any upstream effect. Lattice never reserves, loads, drains or releases serving capacity. r[lattice.serving_binding.port_boundary]
- Run Animus's pure screen and dispatch gate for every brokered request of a served route. Stale, out-of-order, too-old, draining and not-ready snapshots, model or configuration replaced between selection and dispatch, replaced deployment generations or endpoints, revoked or expired grants, owner rejections and exhausted request budgets never dispatch. A capacity snapshot is never a reservation. r[lattice.serving_binding.request_admission]
- Persist a dispatch intent before upstream contact and one `lattice.model-request-observation.v1` per request afterwards, carrying the Animus receipt identity and exact worker, binding, service, deployment and request provenance. r[lattice.serving_binding.request_observation]
- Extend `lattice.execution-binding-provenance.v1` model-request coverage with `no_model` and per-service `served` usage that reaches #13's existing `Scope::Attempt` observation path. Known and unknown usage stay separate, unknown is never zero, shared deployments are never charged in full per worker, recovery deduplicates exact attempt/service/request observations and settled usage cannot be reset. r[lattice.serving_binding.usage]
- Keep #13's overrun visible and enforceable: the usage summary carries reserved per-request token envelopes, known provider-reported tokens, per-request overruns, unknown usage, refused requests and `is_exact_ceiling` (never true while usage is unknown). An optional `stop_on_overrun` declaration field refuses later served requests before any serving-port call, and the broker's existing request-budget refusal is recorded once as `budget-exhausted`. r[lattice.serving_binding.overrun]
- Report inventory serving readiness as its actual cause (`no_serving_port`) or as the injected port's latest snapshot readiness and freshness, replacing the `unknown`/`animus#17` placeholder. r[lattice.serving_binding.readiness]
- Keep compute admission independent: a direct-accelerator worker still needs its own device allocation, and a ready serving route never satisfies it. Lost acknowledgements, timeouts and cancellation never relaunch, retry or reroute. r[lattice.serving_binding.independence]
- Document reservation-only versus qualified measured usage, and record that no live serving owner or provider was exercised. r[lattice.serving_binding.non_claims]

## Impact

- **Vendored:** `vendor-cargo-deps/animus` at `1c370d50` with `animus-frozen-policy`; `Cargo.toml` gains its path dependency (shared contract with the #23 change).
- **Pure core:** `crates/lattice-core/src/execution_binding/serving.rs` (declaration, record and usage summary) and the binding identity gains an optional `serving-declaration` part. The core's dependency set is unchanged; it does not depend on Animus.
- **Shell:** `src/execution_binding/serving/` (port, unavailable and fixture ports, the gate session over `animus_adapters::serving`). The candidate-worker broker consults an injected `ModelRequestGate` per request. The implementation-attempt CLI gains `run_command_with_ports`.
- **Usage:** attempt usage provenance carries per-service request usage. Orchestration settlement arithmetic is unchanged.
- **Docs:** new `docs/worker-serving-binding.md`; `docs/worker-compute-binding.md` and `docs/orchestration-budget-usage.md` gain the serving and compatibility sections.
- **Compatibility:** the declaration is optional; persisted bindings and usage rows without it decode unchanged. `model_requests: "unknown"` remains the coverage of a route without a serving declaration.

## Dependencies

- Local: `bind-worker-compute-allocations` (binding, port pattern, usage provenance).
- Producer: Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b` (`crates/animus-adapters/src/serving*`, `docs/serving-admission.md`, `config/serving-capacity-profile.ncl`).
- Deferred owners: Kiln #49 (live serving owner, provider and cross-host qualification), Aspen #42 (device utilization), Kiln #45 (aggregate budgets), Kiln #60 (joined presentation), Animus #18 (qualification evidence).

## Out of Scope

- A serving owner, scheduler, replica router, model loader or health probe.
- Pricing, currency or cost apportionment arithmetic; only an explicit versioned attribution policy reference is retained.
- Serving admission for orchestration Agent nodes (Pi, Clankers, Animus, direct OpenAI-compatible routes).
- Live qualification of any endpoint, serving owner or device.
