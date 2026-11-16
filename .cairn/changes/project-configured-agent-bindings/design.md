## Design

`RuntimeOptions::configured_profiles` projects at most the three supported target variants. Existing route selection rejects missing/ambiguous routes; existing command/direct planning checks configuration without effects. The projection omits executable paths/argv, endpoints, prompts and credentials. Generic commands do not acquire qualified session/image/checkpoint semantics from a configuration entry. Direct OpenAI-compatible and Animus leaf routes retain their actual text/JSON, no-image/no-session limitations.

`ConfiguredBinding` uses `lattice.configured-agent-runtime.v1`, the existing target, and the exact existing `Adapter::execution_identity` digest. No admission identity algorithm changes. This identifies host configuration, not executable authenticity, worker ownership, service readiness or allocation. Compute reservations, named serving bindings and qualified worker telemetry remain unsupported.

Production response metadata carries the binding independently of model content. Orchestration accepts the bounded schema/hash shape and exact workflow target, then freezes the optional binding in `ProviderReport` through the existing command-worker usage receipt. Absence preserves historical compatibility and unknown provenance. Malformed or crossed provenance does not establish usage; conservative reservation fallback remains. Existing immutable-content comparison handles replay/conflict, while owned outcome refinement remains the only permitted mutation.

## Producer boundary

Inspected Aspen `1e9c36edc437fdf4118120776d94f71570190511`, Animus `1785984ec15bf8e799da13c895abe80e58788b2d`, and Lattice's unchanged Animus snapshot `51d2683776ab73d4fd36d3d9e6ced2beadf9793d`. Existing membership placement and Mesh catalog/dispatch DTOs are not the proposed #42 compute device/backing-pool reservation or #17 serving readiness/generation/request admission APIs. No producer DTO, clock, resource reservation, provider invocation or sibling-relative dependency is added.

## Coverage

Authored deterministic tests cover material-free projections; missing/ambiguous/invalid configuration; truthful direct limits; metadata identity equal to admission identity; dynamic-input independence and configuration drift; optional/malformed/crossed report provenance; durable duplicate/restart/conflicting binding behavior. Parent runs all verification after integration. No fixture establishes live availability, resource enforcement, actual worker lifecycle or provider billing.
