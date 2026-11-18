# Execution Binding Specification

## ADDED Requirements

### Requirement: Versioned worker profile facts
r[lattice.execution_binding.profile_facts] Lattice MUST publish a bounded `lattice.worker-profile-facts.v1` projection for each worker profile its selected runtime actually admits. The projection MUST reuse the existing profile owners: `ConfiguredBinding.revision` for configured Pi, Clankers and Animus routes, and the implementation-attempt `worker_profile_identity` for the OMP attempt worker. It MUST report the adapter, protocol revision, executable byte identity, inputs, outputs, session and checkpoint semantics, workspace, network and isolation requirements, deadline and output bounds. It MUST also report one model requirement (`no_model` or at most 8 named routes) and one compute requirement (`host_local` or `provider_execution` with an optional direct-accelerator requirement). Each fact MUST carry exactly one support level: `declared`, `qualified`, `unavailable` or `unknown`. `qualified` MUST NOT be emitted without bound qualification evidence. The projection MUST omit commands, argv, prompts, endpoints, credential values and physical paths, and MUST NOT grant execution or credential authority.

#### Scenario: Attempt worker profile projection
r[lattice.execution_binding.profile_facts.scenario.projection]
- GIVEN a valid implementation-attempt v3 request with an OMP profile, a brokered model route and a declared compute requirement
- WHEN worker profile facts are projected
- THEN the projection MUST carry the exact `worker_profile_identity` as its revision, the OMP executable BLAKE3 and protocol revision, one named model route identity and the declared compute requirement as `declared`
- AND CPU, memory and storage bounds that Lattice does not enforce MUST be `unknown`, and no endpoint, prompt, path or credential value MUST appear

#### Scenario: Unsupported requirements fail before effects
r[lattice.execution_binding.profile_facts.scenario.unsupported]
- GIVEN a request whose profile declares a no-model OMP worker, more than one model route, a direct-accelerator requirement with `host_local` placement, an unknown compute variant, or an unknown declaration field
- WHEN the request is validated
- THEN admission MUST refuse it before any binding, intent or worker effect is persisted, and MUST NOT downgrade the requirement or substitute another route

#### Scenario: Existing configured routes keep truthful compatibility
r[lattice.execution_binding.profile_facts.scenario.compatibility]
- GIVEN existing Pi, Clankers or Animus runtime configuration
- WHEN worker profile facts are projected for those routes
- THEN compute reservations and model-service bindings MUST be `unavailable` and direct-adapter text/JSON, no-image and no-session limits MUST remain as `project-configured-agent-bindings` reports them
- AND a newly configured or advertised executable MUST NOT become `qualified` or trusted by appearing in configuration

### Requirement: Consumer DTO for producer compute allocation readback
r[lattice.execution_binding.allocation_observation] Lattice MUST admit provider compute state only as a `lattice.compute-allocation-observation.v1` DTO. The DTO MUST project the published Aspen `compute-capacity-allocation-v2` record (all 24 fields in producer order), the matching row and host fields of `compute-capacity-readback-v1` and, optionally, the claimed-pool facts of `compute-capacity-offer-v2` at exactly the allocation's offer revision. Each projected record MUST carry its canonical producer `blake3:` reference. The DTO MUST also carry port provenance: port identity, port profile reference, a monotonic observation sequence and an optional provider logical time. Validation MUST be pure and MUST require exact producer labels and schemas, bounded tokens of at most 256 bytes with no control characters, canonical lowercase `blake3:` references, at most 128 list items and producer-equivalent structural invariants. Validation MUST also require cross-record consistency between the allocation, the readback row and the offer facts. Lattice MUST NOT compare producer logical time with its wall clock.

#### Scenario: Valid joined observation is admitted
r[lattice.execution_binding.allocation_observation.scenario.admitted]
- GIVEN a synthetic DTO derived from the producer's `agent-a-serving-b` fixture: execution allocation `agent-a-1`, status `active`, enforcement `logical-fence-only`, an opaque `serving-b` reference, a fresh healthy readback row with equal owner fields, and provider logical time 30, which is below `valid_until` 80
- WHEN the pure validator admits it
- THEN every producer field MUST be preserved verbatim, including enum spellings such as `mmio-bundle`, `serving-deployment` and `backend-fenced`, and no field MUST be defaulted or inferred

#### Scenario: Malformed or inconsistent observations are rejected
r[lattice.execution_binding.allocation_observation.scenario.malformed]
- GIVEN a DTO with a `-v1` allocation or `-v2` readback label, a wrong reservation schema, `id` different from `operation_id`, a repeated claim pool, zero quantity or generation, an authority-count mismatch, a serving deployment that itself references a serving allocation, an `active` status without activation evidence, a noncanonical or uppercase reference, more than 128 items, or a readback row whose owner, generation, granularity, status, enforcement or logical-fence flag differs from the allocation
- WHEN the pure validator evaluates it
- THEN it MUST be rejected with a typed reason and MUST NOT produce a partial observation

#### Scenario: Unknown provider facts stay unknown
r[lattice.execution_binding.allocation_observation.scenario.unknown]
- GIVEN a valid DTO without provider logical time, without offer pool facts, with readback `is_fresh` false or with health other than `healthy`
- WHEN the observation is classified
- THEN expiry, device kind, freshness or health MUST be reported unknown or unavailable respectively, never as zero, unexpired, idle or healthy

### Requirement: Read-only injected compute port
r[lattice.execution_binding.port_boundary] Lattice MUST obtain compute observations only through an injected read-only `ComputeAllocationPort` that returns `Observed`, `NotFound`, `Unavailable` or a transport issue. Lattice MUST NOT reserve, activate, drain, release, retire or probe compute through this port or any other path in this capability. The standalone CLI MUST install a port that always reports `Unavailable`. Deterministic fixtures MUST inject a fixture port. Lattice MUST NOT add an Aspen crate, a Preserves codec or a sibling-relative dependency for this capability.

#### Scenario: Port is observation-only
r[lattice.execution_binding.port_boundary.scenario.read_only]
- GIVEN a recording fixture port
- WHEN an attempt is admitted, revalidated, cancelled, recovered or inspected
- THEN the only port calls MUST be `observe_allocation` reads during admission and pre-spawn revalidation, and recovery, cancellation and inspection MUST make no port calls

#### Scenario: Standalone CLI without a provider
r[lattice.execution_binding.port_boundary.scenario.unavailable]
- GIVEN the standalone CLI with its default unavailable port
- WHEN a v3 request declares `provider_allocation` compute
- THEN the run MUST stop before binding, intent or worker effects with a blocked `provider_unavailable` reason
- AND a v3 request declaring `host_local` compute MUST proceed through the existing worker path without any port call or fabricated lease

### Requirement: Pure execution binding admission
r[lattice.execution_binding.admission] A pure core MUST admit an exact `AdmittedBinding` from the request's `ExecutionBindingDeclaration`, the worker profile facts, the attempt context (work, attempt, fence and envelope identities) and, for `provider_allocation`, one validated observation. It MUST require the declared compute variant to match the profile requirement. For provider allocations it MUST require the allocation `kind` to be `execution` and require equality of the allocation id, host, consumer and owner generation with the declared expectation. The producer `attempt_id` MUST equal the admitted Lattice attempt id. Status MUST be `active`, the provider logical time MUST be known and below `valid_until`, and the readback MUST be fresh and `healthy`. A direct-accelerator requirement MUST be satisfied only by offer pool facts showing claimed `chips` pools of sufficient quantity, an accepted granularity, matching compatibility fields and at least the required enforcement strength. The core MUST return `Rejected` for definite mismatches and `Blocked` for unknown or not-yet-current facts. Neither outcome MUST permit effects.

#### Scenario: CPU worker with separately hosted model
r[lattice.execution_binding.admission.scenario.cpu_remote_model]
- GIVEN a `provider_execution` profile without a direct-accelerator requirement, one brokered model route, and an active execution allocation on host A whose `serving_allocation_id` is `serving-b`
- WHEN binding admission runs
- THEN the admitted binding MUST record both the compute allocation provenance and the model route identity
- AND it MUST record `serving-b` only as an opaque `shared_serving_ref`, without reading, reserving or copying any serving-deployment pool claim

#### Scenario: Two workers share one serving deployment
r[lattice.execution_binding.admission.scenario.shared_serving]
- GIVEN two attempts with distinct attempt ids and execution allocations `agent-a-1` and `agent-a-2` that both reference `serving-b`
- WHEN both bindings are admitted
- THEN they MUST have distinct binding identities and their reserved claims MUST contain only their own host-A pools
- AND no binding MUST contain any host-B claim or a second charge for `serving-b`

#### Scenario: Hosted-API and no-model workers bind without leases
r[lattice.execution_binding.admission.scenario.hosted_and_no_model]
- GIVEN a `host_local` profile with one brokered hosted-API route, and separately a `host_local` profile declaring `no_model`
- WHEN binding admission runs without any observation
- THEN both bindings MUST carry compute provenance `host_local` with no allocation, claims or placement fields, and the no-model binding MUST carry no model route
- AND supplying an allocation observation to either MUST be rejected as a requirement mismatch

#### Scenario: Direct-accelerator worker cannot use CPU-only or remote-serving facts
r[lattice.execution_binding.admission.scenario.direct_device]
- GIVEN a profile requiring one directly allocated chip
- WHEN the observed execution allocation claims only a memory pool (CPU-only offer), when offer pool facts are absent, when the granularity is not accepted, when enforcement is weaker than required, or when only the referenced serving deployment holds chips and is healthy
- THEN admission MUST reject the CPU-only, granularity, enforcement and serving-only cases and MUST block the missing-evidence case, and no case MUST be admitted

#### Scenario: Crossed, expired or non-current allocations are refused
r[lattice.execution_binding.admission.scenario.crossed]
- GIVEN an observation whose producer `attempt_id`, consumer, owner generation, allocation id or host differs from the admitted expectation, whose kind is `serving-deployment`, whose status is `released`, or whose provider logical time is at or after `valid_until`
- WHEN binding admission runs
- THEN it MUST return `Rejected` with the typed reason
- AND status `intent` or `quarantined`, stale or unhealthy readback, unknown logical time, port `Unavailable` or a transport issue MUST return `Blocked` and MUST NOT be retried into a different allocation

### Requirement: Stable binding identity
r[lattice.execution_binding.identity] The binding identity MUST be a domain-separated BLAKE3 identity under domain `lattice-execution-binding-v1`, framed by the admitted content-identity-core mechanics and spelled `blake3:<64 lowercase hex>`. It MUST cover the profile revision, attempt context, compute provenance (host-local tag, or allocation id, consumer, owner generation, placement generation, charge cursor and time, kind, claims, shared-serving reference and port identity), and the model routes. It MUST exclude volatile observation fields: sequence, provider logical time, readback revision, freshness, status, enforcement and producer record references. Fixed positive and negative vectors MUST pin the framing.

#### Scenario: Re-observation preserves identity
r[lattice.execution_binding.identity.scenario.stable]
- GIVEN the same admitted binding observed again with a higher sequence, later logical time, newer readback revision and enforcement upgraded from `backend-acknowledged` to `backend-fenced`
- WHEN the identity is recomputed
- THEN it MUST equal the pinned vector

#### Scenario: Semantic drift changes identity
r[lattice.execution_binding.identity.scenario.drift]
- GIVEN a change to any covered field, including a different model route, owner generation, topology generation, charge cursor, shared-serving reference or profile revision
- WHEN the identity is recomputed
- THEN it MUST differ from the pinned vector

### Requirement: Persist before dispatch and revalidate before spawn
r[lattice.execution_binding.persistence] The implementation-attempt shell MUST write the admitted binding and its admission observation once, with fsync, to `execution-binding.json` before writing `intent.json`. After the intent is persisted and immediately before worker spawn, it MUST re-observe the allocation and run pure revalidation. Revalidation MUST require an equal placement generation, owner fields, attempt, kind, claims, shared-serving reference, charge cursor and charge time. It MUST also require a strictly greater observation sequence, status `active`, an unexpired logical time, a fresh healthy readback and enforcement no weaker than at admission. Any other result MUST be written once to `binding-revalidation.json` and the worker MUST NOT spawn. Host-local bindings MUST skip the port and still persist the binding before the intent.

#### Scenario: Binding is durable before intent
r[lattice.execution_binding.persistence.scenario.persisted_first]
- GIVEN an admissible request and a fixture port
- WHEN the run is interrupted after `execution-binding.json` and before `intent.json`
- THEN a repeated run MUST re-admit, require an identical binding identity before writing the intent, and refuse a different binding without overwriting the persisted one

#### Scenario: Pre-spawn revalidation refuses changed allocations
r[lattice.execution_binding.persistence.scenario.revalidation]
- GIVEN a persisted binding and intent
- WHEN the pre-spawn observation reports a different topology generation or host or device incarnation (wrong placement generation), a different charge cursor (a reservation readmitted after producer replay-horizon eviction), a crossed owner generation, an equal or lower sequence, a `quarantined` or `released` status, an expired logical time or downgraded enforcement
- THEN the worker MUST NOT spawn, the typed decision MUST be persisted once, and recovery MUST report `not_dispatched_binding_rejected` rather than unknown or success

#### Scenario: Binding changes require a new attempt
r[lattice.execution_binding.persistence.scenario.changed_binding]
- GIVEN a persisted binding for an attempt
- WHEN a request for the same attempt arrives with a different profile revision, model route, compute declaration or expected allocation
- THEN it MUST be refused as crossed, and only a separately admitted request with a new attempt identity MUST be able to bind a different route

### Requirement: Recovery and cancellation keep the exact binding
r[lattice.execution_binding.recovery] Recovery, cancellation and reconnect MUST retain the persisted binding identity and the existing one-shot reconciliation. A lost port acknowledgement, port timeout, worker disconnect, inference timeout or controller crash after the intent MUST NOT launch another attempt, rebind, re-observe for authorization or change the admitted route. Cancellation MUST reuse the existing `cancel.request` propagation. Requested cancellation MUST stay distinct from observed termination. Lattice MUST NOT release, retire or shorten any compute allocation when the caller stops waiting.

#### Scenario: Lost acknowledgement after intent
r[lattice.execution_binding.recovery.scenario.lost_acknowledgement]
- GIVEN an intent and binding persisted and a port transport failure, inference timeout or controller crash before any terminal observation
- WHEN `recover` or a repeated `run` executes
- THEN it MUST reconcile retained results only, report outcome unknown when no terminal evidence exists, and report the original binding identity unchanged

#### Scenario: Cancellation does not release compute
r[lattice.execution_binding.recovery.scenario.cancellation]
- GIVEN a dispatched attempt bound to an active allocation
- WHEN cancellation is requested and the caller stops waiting
- THEN Lattice MUST report `cancellation_requested` separately from any observed termination, and it MUST make no port call and emit no release, drain or retire intent for the allocation

### Requirement: Owned worker instance observations
r[lattice.execution_binding.instances] Lattice MUST publish a read-only, bounded `lattice.worker-instance-observation.v1` projection for each owned implementation-attempt execution. The projection is built from retained state artifacts and MUST be separate from profile facts. It MUST carry the profile revision, logical worker identity (work and attempt), incarnation (the persisted request identity), fence, binding identity, compute provenance without backend handles, model route identities, lifecycle, cancellation and termination observations, observation source, a host observation time and a state digest. Lifecycle MUST be one of `admitted_not_dispatched`, `binding_rejected_before_dispatch`, `dispatch_recorded_outcome_unknown` or `terminal_observed`. Host reachability, model readiness and current allocation liveness MUST be reported `unknown`. The projection MUST NOT create a queue, claim or scheduling decision.

#### Scenario: Inventory separates profiles from instances
r[lattice.execution_binding.instances.scenario.projection]
- GIVEN a retained attempt with binding, intent and terminal candidate result
- WHEN `implementation-attempt inspect` runs
- THEN its output MUST contain separate `profile` and `instance` sections, and the instance MUST report `terminal_observed` with the exact binding, incarnation and fence

#### Scenario: Missing or stale lifecycle stays unknown
r[lattice.execution_binding.instances.scenario.unknown]
- GIVEN an intent without terminal evidence, a missing or unreadable state artifact, or a profile with no retained execution
- WHEN instances are projected
- THEN the first MUST report `dispatch_recorded_outcome_unknown`, the second MUST fail closed with a typed read error rather than an inferred state, and the third MUST report no instance
- AND no projection MUST report idle, running, healthy or available

### Requirement: Compute provenance in usage observations
r[lattice.execution_binding.usage_provenance] Command-worker usage observations MUST be able to carry an optional frozen `lattice.execution-binding-provenance.v1` record. The record MUST contain the binding identity, profile revision, worker incarnation, attempt, fence, compute provenance, model route identities and per-dimension coverage. Compute provenance is `host_local`, or a provider allocation with its id, consumer, owner generation, placement generation, charge cursor, kind, shared-serving reference, claims labelled `reserved` and the enforcement observed at binding. Coverage MUST keep `worker_process` as `host_measured` or `unknown`, and `model_requests` and `device_utilization` as `unknown` until their owners supply them. The implementation-attempt shell MUST persist a host-owned `Scope::Attempt` observation before publishing the attempt result. Provenance MUST NOT change reserved or charged budget, refund consumption, or be interpreted as measured device use.

#### Scenario: Attempt usage carries exact binding
r[lattice.execution_binding.usage_provenance.scenario.bound]
- GIVEN a terminal attempt bound to allocation `agent-a-1`
- WHEN its usage observation is persisted
- THEN it MUST carry the exact binding identity, placement generation, owner generation and charge cursor
- AND reserved claim quantities MUST remain labelled `reserved`, host duration MUST appear only when the host observed both endpoints, and model and device usage MUST be `unknown`, never zero

#### Scenario: Shared or absent allocations are not charged per worker
r[lattice.execution_binding.usage_provenance.scenario.shared_allocation]
- GIVEN two attempts sharing `serving-b`, a hosted-API attempt and a no-model attempt
- WHEN their usage observations are persisted
- THEN neither shared attempt MUST include serving-deployment claims, the hosted-API attempt MUST carry `host_local` compute without a lease, and the no-model attempt MUST carry no model route or inference usage

#### Scenario: Replay deduplicates and conflicts are refused
r[lattice.execution_binding.usage_provenance.scenario.replay]
- GIVEN a persisted attempt usage observation with binding provenance
- WHEN an identical observation is saved again, a restart repeats persistence, or an observation with a different binding identity, placement generation or attempt is saved for the same attempt
- THEN the identical save MUST be idempotent, the restart MUST apply the same receipt once, and the conflicting save MUST be rejected
- AND orchestration `HostRun` settlement MUST produce identical reserved and charged amounts with or without provenance present

### Requirement: Deferred producer capabilities stay non-claims
r[lattice.execution_binding.deferred] Lattice MUST report every capability whose owner has not published a consumer-ready contract as unavailable or unknown, and MUST name that owner. Serving readiness, request admission, replica routing and per-request model-service usage belong to Animus #17. Pricing and currency identity belong to Lattice #13 and have no producer contract yet. Live host, device and cross-host qualification belong to Kiln #49 and Aspen #15/#20. Qualification evidence that promotes `declared` to `qualified` belongs to Animus #18. Aggregate budgets belong to Kiln #45. Revision promotion belongs to Lattice #25. Checkpoint and restore authority belongs to Lattice #12. Lattice MUST NOT fabricate these facts. Producer record references MUST stay opaque canonical `blake3:` identifiers that are validated for shape and compared exactly; Lattice MUST NOT claim to decode Preserves or recompute them, and attributes the authenticity of a projection to the recorded port identity. Deterministic fixtures MUST be labelled synthetic and MUST NOT count as live-capacity, enforcement or serving evidence.

#### Scenario: Serving facts remain unavailable
r[lattice.execution_binding.deferred.scenario.serving]
- GIVEN a binding referencing a shared serving deployment and a brokered model route
- WHEN profile facts, instance observations or usage provenance are projected
- THEN serving readiness, request admission and per-request usage MUST be `unknown` with owner `animus#17`, and a healthy compute readback MUST NOT be reported as model readiness

#### Scenario: Fixtures are not live qualification
r[lattice.execution_binding.deferred.scenario.live]
- GIVEN only synthetic DTO fixtures and the fixture port
- WHEN qualification or inventory evidence is reported
- THEN it MUST state `live_qualification: false` and `backend_enforcement_observed: false`, MUST retain exact producer revision, consumer schema and negative-case results, and MUST NOT claim capacity, enforcement or cross-host operation
