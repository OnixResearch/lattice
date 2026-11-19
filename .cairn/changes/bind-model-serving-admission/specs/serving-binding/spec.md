# Serving Binding Specification

## ADDED Requirements

### Requirement: Exact Animus serving producer
r[lattice.serving_binding.producer] Lattice MUST consume Animus serving readiness, capacity snapshots, serving-owner request admission and invocation receipts only through the pure `animus_adapters::serving` core vendored by `scripts/sync_animus.rs` at an exact reviewed revision. The vendored crate set MUST be the existing required set plus `animus-frozen-policy`. Lattice MUST NOT vendor or enable `animus-std`'s `serving-admission` host path, its `/v1/models` discovery or its Mesh transport, and MUST NOT add a sibling-relative dependency.

#### Scenario: Vendored snapshot is exact
r[lattice.serving_binding.producer.scenario.exact]
- GIVEN the repository-owned sync script pinned to Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b` and its archive BLAKE3
- WHEN the snapshot is regenerated
- THEN `vendor-cargo-deps/animus/source.ncl` MUST record that revision, archive identity and crate set, and the serving core MUST compile into Lattice without a non-default Animus feature

### Requirement: Optional serving declaration bound into the execution binding
r[lattice.serving_binding.declaration] A per-attempt execution binding MAY carry one `lattice.serving-binding.v1` declaration. It MUST name a serving profile (profile id, identity rule, capacity rule, replica rule and a maximum snapshot age of 1 to 3600 logical ticks), a privacy band, an attribution policy and at most 8 served routes. Each served route MUST name an already declared model route and carry its logical service id, endpoint identity, exact model id, canonical `blake3:` configuration reference and request requirements (tools, streaming, structured output, developer role, positive output tokens and input tokens). Tokens MUST satisfy the producer's opaque-token rule. Validation MUST be pure and MUST refuse a declaration before any binding, intent or worker effect. An admitted binding MUST freeze the declaration, and the `lattice-execution-binding-v1` identity MUST include it as a tagged part only when present, so bindings without a declaration keep their identity.

#### Scenario: Served route admitted with the binding
r[lattice.serving_binding.declaration.scenario.admitted]
- GIVEN a host-local CPU worker with one `primary` route and a declaration serving it from `serving-b` with an observed-required, known-required profile
- WHEN the binding is admitted
- THEN the admitted binding MUST carry the declaration verbatim and its identity MUST differ from the same binding without it, while the binding without it MUST keep its prior identity vector

#### Scenario: Unsupported declarations fail before effects
r[lattice.serving_binding.declaration.scenario.unsupported]
- GIVEN a declaration with a wrong schema, a route name not declared as a model route, a repeated route, more than 8 routes, an unsafe token, a noncanonical configuration reference, zero output tokens, output tokens exceeding input-plus-output bounds, a zero or oversized snapshot age, a malformed attribution policy, or any declaration on a no-model binding
- WHEN admission evaluates it
- THEN admission MUST reject it with a typed defect and persist nothing

### Requirement: Read-only injected serving-owner port
r[lattice.serving_binding.port_boundary] Lattice MUST obtain serving facts only through an injected `ServingOwnerPort` with three operations: observe an availability snapshot with a port-supplied logical time, ask the serving owner to admit one exact request, and observe the current serving state for that request. Lattice MUST NOT reserve, load, drain, release or probe serving capacity. The standalone CLI MUST install `UnavailableServingPort`, which reports every operation unavailable. Deterministic fixtures MUST use a recording fixture port whose results are fixture evidence only.

#### Scenario: Standalone CLI blocks served routes
r[lattice.serving_binding.port_boundary.scenario.unavailable]
- GIVEN the default unavailable port and a binding with a served route
- WHEN the worker issues a model request
- THEN the request MUST be refused before any upstream contact, the observation MUST be `not-dispatched` with a blocked reason, and no dispatch intent MUST be written

#### Scenario: Unserved routes never call the port
r[lattice.serving_binding.port_boundary.scenario.unserved]
- GIVEN a binding without a serving declaration
- WHEN the broker forwards requests
- THEN no serving port call MUST occur, no request observation MUST be written and model-request coverage MUST stay `unknown`

### Requirement: Per-request serving gate in the model broker
r[lattice.serving_binding.request_admission] For every request on a served route, the attempt-local model broker MUST screen the declared route against the current snapshot with Animus `screen_routes`, query the serving owner for a request-scoped grant, recheck current serving state and run Animus `gate_dispatch`, with the broker's existing exact-model, deadline and request-budget checks as the existing dispatch admission. Only an `Admitted` verdict MAY reach the upstream endpoint. The broker MUST track the newest accepted snapshot position for the attempt and pass it as freshness. Request identities MUST be deterministic from the binding identity, route and a per-route ordinal so recovery can deduplicate them.

#### Scenario: Capacity snapshot is not a reservation
r[lattice.serving_binding.request_admission.scenario.not_reservation]
- GIVEN two attempts sharing one service whose snapshot reports one free request slot
- WHEN both screen the route as eligible and the owner grants only the first
- THEN only the first MUST dispatch, the second MUST record `not-dispatched` with the owner rejection, and an unavailable owner MUST block dispatch even when the snapshot reports capacity

#### Scenario: Stale or changed serving facts never dispatch
r[lattice.serving_binding.request_admission.scenario.refused]
- GIVEN an expired, too-old, out-of-order or freshness-unknown snapshot; a draining, loading or absent service; a recheck reporting another model, configuration, endpoint or deployment generation; a grant for another request, an expired grant or a changed admission epoch; an owner rejection; or an exhausted broker request budget
- WHEN the gate runs
- THEN no upstream contact MUST occur and the request observation MUST record the typed exclusion, rejection or block

### Requirement: Durable per-request observations
r[lattice.serving_binding.request_observation] Before upstream contact the broker MUST durably write one `lattice.model-request-dispatch.v1` intent for an admitted request. After the gate refuses or the upstream call ends, it MUST durably write one `lattice.model-request-observation.v1` record. The record MUST carry the binding identity, profile revision, worker incarnation, attempt and fence, route name and route identity, request id, service, endpoint, deployment and generation, exact model and configuration with attestation, replica and whether it changed, the serving resource-consumer reference, the snapshot identity and position, the admission record, the outcome class and label, provider-reported or unknown token counters, the evidence label and the Animus receipt identity. Both files are write-once; a different second write MUST be refused.

#### Scenario: Lost acknowledgement or timeout cannot relaunch or reroute
r[lattice.serving_binding.request_observation.scenario.uncertain]
- GIVEN an admitted request whose upstream call times out, fails in transport, is cancelled, or whose response is withheld
- WHEN the broker records it
- THEN the outcome MUST be `uncertain` with unknown usage, the request MUST NOT be retried or replayed, the next request MUST receive a new identity on the same declared route, and no other service or model MUST be selected

#### Scenario: Cancellation requested versus observed
r[lattice.serving_binding.request_observation.scenario.cancellation]
- GIVEN cancellation requested while an admitted request is in flight
- WHEN the broker stops waiting
- THEN the request MUST be recorded `uncertain-cancelled` with unknown usage, the attempt MUST still report `cancellation_requested` separately from observed termination, and no grant or capacity MUST be described as released

### Requirement: Per-service usage reaches #13
r[lattice.serving_binding.usage] The attempt usage observation MUST summarize request observations into per-service `served` model-request coverage inside `lattice.execution-binding-provenance.v1`, persisted through the existing immutable `Scope::Attempt` path before the attempt result. Each service entry MUST keep its exact route, service, model, configuration, resource-consumer reference, attribution label and every request with its own deployment generation, so multi-service attempts keep per-service provenance. Known token counts MUST be summed only from provider-reported counters; requests with unknown usage MUST be counted separately and coverage MUST be incomplete whenever any dispatched request lacks known usage or a recorded outcome. A dispatch intent without an observation MUST appear as `dispatch-unconfirmed` with unknown usage. Shared deployments MUST be labelled `not-apportioned` unless the declaration names an explicit versioned attribution policy, which MUST be labelled an estimate. No-model workers MUST report `no_model` coverage with no counters. Hosted APIs MUST bind host-local compute without claims.

#### Scenario: Duplicate receipts and recovery
r[lattice.serving_binding.usage.scenario.dedup]
- GIVEN identical observations for one attempt, service and request, conflicting observations for the same request, an observation crossed to another binding, or a dispatched observation without its intent
- WHEN usage is summarized
- THEN identical observations MUST count once and every other case MUST be refused

#### Scenario: Partial telemetry after interruption
r[lattice.serving_binding.usage.scenario.partial]
- GIVEN one completed request with provider-reported usage and one admitted request whose observation was never written
- WHEN usage is summarized
- THEN the known tokens MUST be retained, the second request MUST be unknown and coverage MUST be incomplete, never zero

#### Scenario: Settled usage cannot be reset
r[lattice.serving_binding.usage.scenario.settled]
- GIVEN a persisted attempt usage observation with served coverage
- WHEN a reconnect, relocation or recovery saves a different summary for the same attempt
- THEN the store MUST refuse it and the persisted observation MUST remain unchanged

### Requirement: Overrun stays visible and may stop further requests
r[lattice.serving_binding.overrun] Each served route's declared token envelope (input plus output tokens) MUST be the reservation of one dispatched request. The served usage summary MUST carry an overrun block with the declaration's `stop_on_overrun`, the reserved envelope tokens of dispatched requests, the known provider-reported tokens, the known-usage requests that exceeded their own envelope and by how many tokens, the unknown-usage request count, the recorded stop refusals and `is_exact_ceiling`. Each known request MUST be compared with its own envelope, so neither an under-used request nor an unknown request's envelope offsets an overrun. `is_exact_ceiling` MUST be false whenever any dispatched request lacks known usage or any overrun was observed. `stop_on_overrun` MUST default to false, MUST be omitted from the canonical declaration when false and MUST change the binding identity when true. With it, the broker MUST compare known served usage with the envelopes before the gate and refuse the next request once an overrun is known. The broker's existing request-budget refusal MUST be recorded as a not-dispatched `budget-exhausted` record on a served route. Each stop MUST be recorded once per route and reused for later refusals; an identical replay MUST be idempotent and count once. Neither refusal MUST call the serving port or create a dispatch intent, and no envelope MUST be presented as a billing ceiling.

#### Scenario: Overrun is visible and stops further requests when declared
r[lattice.serving_binding.overrun.scenario.stop]
- GIVEN a completed request whose provider-reported tokens exceed its route's declared envelope
- WHEN the summary is built, and when the next request arrives under `stop_on_overrun`
- THEN the summary MUST show reserved and known tokens separately with the overrun, a later under-used request MUST NOT offset it, and under `stop_on_overrun` the next request MUST be recorded `overrun-stopped` with zero snapshot, admission or recheck calls while a binding without the policy keeps dispatching

#### Scenario: Unknown usage is never headroom or an exact ceiling
r[lattice.serving_binding.overrun.scenario.inexact]
- GIVEN a dispatched request whose usage is unknown, alone or beside a known request that exceeds its envelope
- WHEN the summary is built or the next request arrives under `stop_on_overrun`
- THEN `is_exact_ceiling` MUST be false, the unknown request MUST NOT trigger a stop by itself, and its envelope MUST NOT offset the known overrun

#### Scenario: Budget exhaustion is recorded once
r[lattice.serving_binding.overrun.scenario.budget_exhausted]
- GIVEN a served route whose broker request budget is exhausted
- WHEN the worker sends further requests and the retained record is replayed
- THEN exactly one not-dispatched `budget-exhausted` record MUST exist with no intent and no serving-port call, an identical replay MUST be idempotent, and the attempt summary and `implementation-attempt inspect` MUST show one refused request

### Requirement: Inventory names the actual serving readiness cause
r[lattice.serving_binding.readiness] Worker profile facts and instance observations MUST report serving readiness as `no_serving_port` when no serving-owner port is injected, `no_served_route` when an injected port has no declared served route to read, or the injected port's latest snapshot per served route with its identity, position, the service's readiness (or `not-listed`) and the producer's freshness label. A port read that is unavailable or fails MUST be reported as such. Inspection MUST call only the snapshot observation, never admission or recheck, and MUST NOT report a not-ready or stale snapshot as ready. This replaces the `unknown` placeholder with owner `animus#17` of `lattice.execution_binding.deferred.scenario.serving` for readiness only; model-request coverage without a serving declaration keeps that owner.

#### Scenario: No serving port injected
r[lattice.serving_binding.readiness.scenario.no_port]
- GIVEN the standalone CLI or a configured runtime profile without a serving-owner port
- WHEN profile facts and the instance observation are projected
- THEN both MUST report `no_serving_port` and no owner placeholder

#### Scenario: Injected port snapshot
r[lattice.serving_binding.readiness.scenario.injected]
- GIVEN an injected port whose latest snapshot lists the served service as ready and current, or a port that answers unavailable
- WHEN `implementation-attempt inspect` runs
- THEN both projections MUST carry that route's snapshot identity, position, readiness and freshness, or `unavailable`, after exactly one snapshot call per served route and no admission or recheck call

### Requirement: Compute and serving stay independent
r[lattice.serving_binding.independence] Serving admission MUST NOT satisfy, replace or weaken compute admission. A direct-accelerator worker MUST still require its own provider allocation with sufficient chips, and a healthy or ready serving route MUST NOT admit it from a CPU-only offer. Serving facts MUST NOT add tools, endpoints, credentials or disclosure destinations.

#### Scenario: GPU-bound worker with a healthy remote service
r[lattice.serving_binding.independence.scenario.gpu]
- GIVEN a direct-accelerator worker, a CPU-only execution allocation and a served route that is ready with capacity
- WHEN the binding is admitted
- THEN compute admission MUST reject it and no serving port call MUST occur

### Requirement: Compatibility documentation and non-claims
r[lattice.serving_binding.non_claims] Documentation MUST distinguish reservation-only operation (configured charges, host duration, unknown model usage) from serving-bound measured usage (provider-reported per-request counters with provenance), MUST state that provider counters are reports rather than measurements, and MUST state that no live serving owner, endpoint, provider or device was exercised; live qualification belongs to Kiln #49.

#### Scenario: Fixture evidence is labelled
r[lattice.serving_binding.non_claims.scenario.fixture]
- GIVEN only the fixture serving port
- WHEN evidence is recorded
- THEN every observation MUST carry the `fixture` evidence label and the qualification record MUST state `live_qualification: false`

#### Scenario: Evidence label follows port provenance
r[lattice.serving_binding.non_claims.scenario.provenance]
- GIVEN a fixture port, the unavailable standalone port or an embedder-supplied port
- WHEN a request is dispatched or refused
- THEN the record MUST be `fixture` for the fixture port, `live_endpoint` only for a request an embedder-supplied port admitted to upstream, and `no_upstream_contact` otherwise; nothing returned by the port or upstream MUST set the label
