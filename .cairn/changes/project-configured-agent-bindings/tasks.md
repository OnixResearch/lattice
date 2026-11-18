## Implementation

- [x] [serial] Reuse existing route/configuration identity and pure planners for a bounded authority-free profile projection. r[lattice.agent_runtime.configured_bindings]
- [x] [serial] Persist host-selected runtime provenance with existing conservative usage, rejecting malformed/crossed identities and preserving replay/restart immutability. r[lattice.agent_runtime.configured_bindings.scenario.usage]
- [x] [serial] Author positive/negative deterministic configuration, production metadata, normalization and durable conflict fixtures. r[lattice.agent_runtime.configured_bindings.scenario.compatibility] r[lattice.agent_runtime.configured_bindings.scenario.usage]
- [x] [serial] Document exact producer pins and unavailable capabilities without invented producer schemas or live claims. r[lattice.agent_runtime.configured_bindings.scenario.boundary]

## Integration and qualification

- [x] [serial] Parent runs focused fixtures and repository lifecycle/quality gates after shared integration and retains actual results. r[lattice.agent_runtime.configured_bindings]
- [ ] [serial] Adopt actual producer-owned compute reservation and serving readiness/request-admission schemas when implemented, preserving exact attempt/fence/generation and unknown-effect semantics. r[lattice.agent_runtime.configured_bindings.scenario.boundary]
- [ ] [serial] Separately authorize and qualify the exact worker/compute/service path, owned lifecycle and supported per-request telemetry. r[lattice.agent_runtime.configured_bindings.scenario.boundary]

The parent passed all 32 x86_64-linux Nix checks, including 1,445 passing Nextest cases with 10 optional cases skipped, and the six native proposal/design/tasks gates for this change and preserve-candidate-worker-observations. The selected OMP normal/failure/cancellation cases were separately exercised and are retained in that change's source-qualification record. These are local integration observations, not compute-reservation or inference-service qualification; issues #24/#13 remain open for actual producer contracts and joined live usage.

The compute half of the first open task (Aspen `aac7b5ba` compute-capacity records through a read-only injected port, with exact attempt/fence/owner-generation revalidation) is carried by `bind-worker-compute-allocations`. That task stays open here for the serving readiness and request-admission schemas still blocked on Animus #17.
