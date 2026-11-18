# Design: Worker execution bound to provider-owned compute allocations

## Context

Baseline: Lattice `3d0aac9008c22d9c94e4808eaf77d34b74174b8b`. Producer specification: Aspen `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4` (the #42 closing commit and the last commit touching the compute-capacity sources).

What already exists:

- `project-configured-agent-bindings` projects `ConfiguredProfile` (schema `lattice.configured-agent-runtime.v1`, `CompatibilitySupport::{Unsupported, Unknown}`) for configured Pi, Clankers and Animus routes. It reports `compute_reservations: unsupported`.
- The implementation-attempt CLI (`src/implementation_attempt/cli*.rs`) binds the Kiln-owned envelope (`work_id`, `attempt_id`, `fence`, `worker_profile`, `budget`) with an expiring request. It recomputes `worker_profile_identity()` (BLAKE3 over the OMP profile and paths), writes `intent.json` with exclusive create and fsync before the worker starts, and reconciles on re-entry without redispatching. Cancellation propagates through `cancel.request`.
- `CommandWorkerObservation` (`lattice.command-worker-usage.v1`) is stored immutably by `attempt_id` in the candidate store. Recovery already treats a usage row without an attempt result as incomplete.
- `crates/lattice-core` is the pure decision crate. Its dependencies are `blake3`, `content-identity-core`, `serde`, `serde_json` and `rat-nodegraph`, enforced by `tests/test_lattice_core_boundary.rs`.

What Aspen publishes (read as a specification; no code dependency):

| Producer label | Arity | Purpose |
|---|---|---|
| `compute-capacity-allocation-v2` | 24 | Persisted reservation: identities, generations, charge cursor/time, `valid_until`, claims, kind, status, enforcement, activation evidence |
| `compute-capacity-readback-v1` | 6 (+7 per `compute-allocation-readback-v1` row) | Host freshness, health, qualification refs, per-allocation owner/generation/granularity/status/fence/enforcement |
| `compute-capacity-offer-v2` | 16 | Offer at a revision: incarnations, topology generation, label authority, compatibility, pools (`compute-pool-v1`, 8), views |
| `molten.compute.reservation.v1`, `molten.compute.offer.v1` | n/a | Inner `schema` strings |

Record references are `blake3:` + 64 lowercase hex over canonical Preserves bytes. Time is logical ticks (`molten.compute.capacity-profile.v1`, `time_domain = "logical"`). Released IDs leave a 128-record FIFO replay horizon and may then be readmitted.

## Goals and non-goals

Goal: an admitted implementation attempt binds exactly one compute provenance (host-local or one provider execution allocation) and its declared model routes. The binding is persisted before dispatch, revalidated before spawn, retained through recovery and visible in usage and instance observations. Every unknown stays unknown.

Non-goals: reservation, activation or release, serving readiness, request admission, pricing, live qualification, and orchestration Agent-node compute binding (see `r[lattice.execution_binding.deferred]`).

## Decision 1: depend on, do not duplicate, configured-route facts

This change adds `depends_on: ["project-configured-agent-bindings"]`. `ConfiguredProfile` stays the owner of configured-route facts. The new `WorkerProfileFacts` maps each `ConfiguredProfile` (`Unsupported` → `unavailable`, `Unknown` → `unknown`) and adds the OMP attempt worker, which only the implementation-attempt request describes. `CompatibilitySupport` is not changed. The new `SupportLevel` is a separate projection enum, so the existing serialized surface stays stable. The compute half of that package's open producer-adoption task is implemented here. Serving adoption remains open there and in `r[lattice.execution_binding.deferred]`.

## Decision 2: consumer DTO `lattice.compute-allocation-observation.v1`

The DTO is a Lattice serde type in `crates/lattice-core/src/execution_binding/model.rs` with `#[serde(deny_unknown_fields)]`. Field names are snake_case. Enum values use the producer's kebab-case spellings exactly. The embedding host's port adapter decodes producer Preserves records into this DTO. Lattice cannot recompute producer references without a Preserves codec, so references are validated for shape and bound into persisted evidence, and authenticity of the projection is attributed to the recorded port identity (see open question 2).

```text
ComputeAllocationObservation {
  schema: "lattice.compute-allocation-observation.v1",
  producer: ProducerCohort,
  port: PortProvenance,
  allocation: AllocationRecord,
  readback: ReadbackProjection,
  offer: Option<OfferPoolFacts>,
}
ProducerCohort {            // exact constants; any other value is rejected
  allocation_label: "compute-capacity-allocation-v2",
  readback_label:   "compute-capacity-readback-v1",
  offer_label:      "compute-capacity-offer-v2",
  reservation_schema: "molten.compute.reservation.v1",
  offer_schema:       "molten.compute.offer.v1",
  time_domain: "logical",
}
PortProvenance {
  port_id: Token,                 // embedding-host adapter identity
  port_profile_ref: Blake3Ref,    // adapter build/config identity
  observation_sequence: u64,      // > 0, strictly monotonic per port_id
  provider_logical_now: Option<u64>, // port-supplied producer tick; None = expiry unknown
}
```

### Allocation fields (`compute-capacity-allocation-v2`, producer order)

| # | DTO field | Producer field | Type / rule |
|---|---|---|---|
| — | `record_ref` | `canonical::Record.reference` | `Blake3Ref` |
| 0 | `schema` | `schema` | exact `molten.compute.reservation.v1` |
| 1 | `id` | `id` | Token; equals `operation_id` |
| 2 | `host_id` | `host_id` | Token |
| 3 | `host_incarnation` | `host_incarnation` | Token (placement generation) |
| 4 | `device_incarnation` | `device_incarnation` | Token (placement generation) |
| 5 | `offer_revision` | `offer_revision` | u64 > 0 |
| 6 | `topology_generation` | `topology_generation` | u64 > 0 (placement generation) |
| 7 | `charge_cursor` | `charge_cursor` | u64 > 0; reservation-instance identity |
| 8 | `charge_at` | `charge_at` | u64 logical tick |
| 9 | `valid_until` | `valid_until` | u64 > 0 logical tick |
| 10 | `authority_ids` | `authority_ids` | ≤128 Tokens; len = claims len |
| 11 | `view_id` | `view_id` | Token |
| 12 | `granularity` | `granularity` | `chip`/`mmio-bundle`/`tray`/`mesh`/`partition`/`alias` |
| 13 | `consumer_id` | `consumer_id` | Token |
| 14 | `operation_id` | `operation_id` | Token |
| 15 | `attempt_id` | `attempt_id` | Token; must equal Lattice attempt |
| 16 | `owner_generation` | `owner_generation` | u64 > 0 |
| 17 | `grant_ref` | `grant_ref` | `Blake3Ref` (reference only, not a grant) |
| 18 | `claims` | `claims` (`compute-claim-v1`) | 1..=128 `{pool_id: Token, quantity: u64 > 0}`, distinct pools |
| 19 | `serving_allocation_id` | `serving_allocation_id` | `Option<Token>`; `None` when kind is serving |
| 20 | `kind` | `kind` | `execution`/`serving-deployment` |
| 21 | `status` | `status` | `intent`/`active`/`quarantined`/`released` |
| 22 | `enforcement` | `enforcement` | `logical-fence-only` < `backend-acknowledged` < `backend-fenced` |
| 23 | `activation_backend_ref` | `activation_backend_ref` | `Option<Token>`; `Some` iff `active`, `None` if `intent`; persisted only, never projected |

### Readback projection (`compute-capacity-readback-v1`)

| DTO field | Producer field | Rule |
|---|---|---|
| `record_ref` | `canonical::Record.reference` | `Blake3Ref` |
| `host_id` | `host_id` | equals allocation `host_id` |
| `revision` | `revision` | u64 > 0, ≥ allocation `offer_revision` |
| `is_fresh` | `is_fresh` | `false` ⇒ blocked |
| `health` | `health` | `healthy`/`degraded`/`unavailable`/`unknown`; non-healthy ⇒ blocked |
| `qualification_refs` | `qualification_refs` | ≤128 distinct `Blake3Ref`; scope only, not live qualification |
| `row_count` | `allocations.len()` | 1..=128; other consumers' rows are not retained |
| `row.id`, `row.consumer_id`, `row.owner_generation`, `row.granularity`, `row.status`, `row.is_logically_fenced`, `row.enforcement` | `compute-allocation-readback-v1` fields 0–6 | each equals the allocation; `is_logically_fenced == (status != released)` |

### Offer pool facts (`compute-capacity-offer-v2`, optional)

| DTO field | Producer field | Rule |
|---|---|---|
| `record_ref` | `canonical::Record.reference` | `Blake3Ref` |
| `host_id`, `host_incarnation`, `device_incarnation`, `revision`, `topology_generation` | offer fields 1, 2, 3, 6, 7 | equal the allocation (`revision == offer_revision`) |
| `health` | offer field 11 | recorded |
| `label_authority` | offer field 12 | `advisory`<`measured`<`operator-declared`<`authoritative` |
| `compatibility.{backend,runtime,architecture,topology}` | `compute-compatibility-v1` fields 0–3 | `Option<Token>` |
| `compatibility.qualification_refs` | `compute-compatibility-v1` field 4 | 1..=128 `Blake3Ref` |
| `claimed_pools[] {pool_id, kind, authority_id}` | `compute-pool-v1` fields 0, 1, 2 of each claimed pool | exactly one per claim; kind ∈ `dedicated-memory-bytes`/`unified-memory-bytes`/`chips`; authority-id set equals allocation `authority_ids` |

Offer facts are optional because the producer may not retain the offer at the allocation's basis revision. Without them, device kind is unknown, so a direct-accelerator requirement blocks and other requirements are unaffected. The serialized DTO is bounded to 65,536 bytes.

## Decision 3: read-only injected port

`src/execution_binding/port.rs` (shell):

```rust
pub trait ComputeAllocationPort: Send {
    /// Read-only observation. Never reserves, activates, releases or probes.
    fn observe_allocation(&mut self, query: &AllocationQuery) -> Result<AllocationReply, ComputePortIssue>;
}
pub struct AllocationQuery { pub allocation_id: String, pub host_id: String, pub consumer_id: String, pub attempt_id: String }
pub enum AllocationReply {
    Observed(Box<ComputeAllocationObservation>),
    NotFound,
    Unavailable { reason: PortUnavailableReason },  // provider_not_configured | provider_reported_unavailable
}
pub struct ComputePortIssue { pub kind: PortIssueKind }  // transport | timeout | malformed_reply; never carries raw provider text beyond 256 bytes
pub struct UnavailableComputePort;                    // standalone CLI default
pub struct FixtureComputePort { replies: VecDeque<Result<AllocationReply, ComputePortIssue>>, calls: Vec<AllocationQuery> } // deterministic fixtures only
```

Mapping: `NotFound` → `Rejected(AllocationNotFound)`. `Unavailable` and every `ComputePortIssue` → `Blocked`. The trait has no write methods. An embedder supplies a real adapter, for example a Kiln #49 host composition or an Aspen client, through `implementation_attempt::run_command_with_compute_port(args, &mut port)`. `run_command` passes `UnavailableComputePort`.

## Decision 4: pure core API

`crates/lattice-core/src/execution_binding/` (no I/O, no clock):

```rust
pub fn validate_observation(o: &ComputeAllocationObservation) -> Result<(), ObservationRejection>;
pub fn admit(
    declaration: &ExecutionBindingDeclaration,
    requirement: &WorkerRequirement,        // compute + model requirement from profile facts
    attempt: &AttemptContext,               // work_id, attempt_id, fence, envelope_identity, profile_revision
    observation: Option<&ComputeAllocationObservation>,
) -> Admission;                             // Admitted(AdmittedBinding) | Rejected(BindingRejection) | Blocked(BindingBlock)
pub fn revalidate(bound: &AdmittedBinding, current: &ComputeAllocationObservation) -> Revalidation; // Current | Rejected | Blocked
pub fn binding_identity(bound: &AdmittedBinding) -> Result<String, IdentityError>;
pub fn project_profile(facts_input: &ProfileInput) -> WorkerProfileFacts;
pub fn project_instance(state: &RetainedAttemptState, observed_at_epoch_ms: i64) -> WorkerInstanceObservation;
```

Declarations (operator-authored Nickel, exported to the v3 request):

```text
WorkerComputeRequirement = host_local
  | provider_execution { direct_accelerator: Option<{ min_chips: u64 >= 1, accepted_granularities: [Granularity] (1..=6),
                         backend/runtime/architecture: Option<Token>, minimum_enforcement: Enforcement }> }
WorkerModelRequirement   = no_model | routes { names: [Token] (1..=8, distinct) }
ExecutionBindingDeclaration {
  schema: "lattice.execution-binding.v1",
  compute: host_local | provider_allocation { allocation_id, host_id, consumer_id, owner_generation },
  model_routes: no_model | routes [{ name: Token, route_identity: Blake3Ref }],
}
```

Rejections (definite): `RequirementMismatch`, `AllocationNotFound`, `CrossedAttempt`, `CrossedOwner` (id, host, consumer or owner generation), `ServingDeploymentAsExecution`, `Released`, `Expired`, `InsufficientDevice` (CPU-only or too few chips), `GranularityNotAccepted`, `IncompatibleRuntime`, `EnforcementTooWeak`, `ModelBindingMismatch`, `WrongPlacementGeneration`, `ReservationReplaced`, `EnforcementDowngraded`, `StaleObservationSequence`, `Malformed(ObservationRejection)`.
Blocks (unknown or not current): `ProviderUnavailable`, `PortIssue`, `NotActive` (intent), `Quarantined`, `ExpiryUnknown`, `ReadbackStale`, `HostUnhealthy`, `DeviceEvidenceUnknown`.

Device rule: chips = Σ `quantity` over claims whose `claimed_pools.kind == chips`. A requirement is satisfied only when chips ≥ `min_chips`, the granularity is accepted, every required compatibility field equals the offer field, and the enforcement meets the minimum. The serving reference and model-route state are never inputs to this rule.

`AdmittedBinding` holds: `schema` `lattice.admitted-execution-binding.v1`, `binding_identity`, `profile_revision`, `attempt` (work, attempt, fence, envelope identity), `compute` (`HostLocal` | `ProviderAllocation { allocation_id, consumer_id, owner_generation, placement: PlacementGeneration { host_id, host_incarnation, device_incarnation, topology_generation }, charge_cursor, charge_at, kind, view_id, granularity, claims, shared_serving_ref, port_id, enforcement_at_bind, admission_sequence }`) and `model_routes`.

## Decision 5: identity framing

Domain `lattice-execution-binding-v1` uses `content_identity_core::IdentityHasher`, the same role pattern as `schedule_occurrence`. The role wrapper lives in `crates/lattice-core/src/content_identity/mod.rs`. Text uses `update_tagged_str`, integers use `update_tagged_u64_le`, optional values use `update_optional_str`, and lists use `update_ordered_strs` after a deterministic sort. Tags in fixed order:

- `profile-revision`, `work-id`, `attempt-id`, `fence`, `envelope-identity`
- `compute-kind` (`host-local` or `provider-allocation`)
- for provider allocations: `allocation-id`, `consumer-id`, `owner-generation`, `host-id`, `host-incarnation`, `device-incarnation`, `topology-generation`, `charge-cursor`, `charge-at`, `allocation-kind`, `view-id`, `granularity`, `claims` (each encoded `pool_id` + `=` + decimal quantity, sorted by pool id), `shared-serving-ref` (optional), and `port-id`
- `model-kind` (`no-model` or `routes`), then `model-routes` (each encoded `name` + `=` + route identity, sorted by name). Pool ids and names may contain `=`, but quantities are decimal and route identities are `blake3:` hex, so splitting at the last `=` stays unambiguous.

The output is spelled `blake3:<hex>`, the spelling of `ConfiguredBinding` and of producer references. Fixed vectors in tests pin positive, reordered-input and drift cases.

Model route identity: for the OMP attempt, BLAKE3 under domain `lattice-model-route-v1` over the broker `schema`, `endpoint`, `model` and credential reference (provider, key, field). Expiry and credential values are excluded. The identity is never projected with the endpoint text.

## Decision 6: request v3 and the dispatch sequence

`REQUEST_SCHEMA` becomes `lattice.implementation-attempt-run.v3` and v2 is refused (explicit readmission, as v1→v2). New required fields: `worker_compute: WorkerComputeRequirement` and `execution_binding: ExecutionBindingDeclaration`. `worker_profile_identity()` adds `worker_compute` to its hashed tuple, so a changed compute requirement changes the profile revision and is caught by the existing `validate_binding`. The OMP model requirement is fixed at one route named `primary`, whose identity is derived from `omp.model_broker`. A declaration whose routes differ is a `ModelBindingMismatch`, and a `no_model` OMP declaration is unsupported. The Kiln envelope schema is unchanged. The request-bytes identity (`intent.json`) already covers the new fields.

State directory sequence in `Context::run` (new `src/implementation_attempt/cli/binding.rs`):

1. If `intent.json` exists, or the command is recover: existing reconciliation; no port call.
2. Validate the request and project profile facts. Run `admit` with observation #1 (provider allocations only). On reject or block, print the typed decision; nothing is written.
3. `write_once(execution-binding.json, {binding, admission_observation})`. If the file already holds a different binding identity, refuse.
4. `write_once(intent.json, request identity)` (existing).
5. For provider allocations, read observation #2 and run `revalidate`. Any result other than `Current` → `write_once(binding-revalidation.json, decision)` and stop with no spawn.
6. Existing `execute` (worker spawn, cancellation watcher, raw-event retention).
7. Persist the `Scope::Attempt` usage observation with provenance (Decision 9), then publish the existing attempt result.

A crash between steps 4 and 5 leaves intent without results, which existing recovery reports as unknown. That is conservative and correct.

## Decision 7: recovery and cancellation

Recovery reads `execution-binding.json` and checks that its `attempt` and `profile_revision` match the request. It reports the binding identity next to the existing reconciliation result. A `binding-revalidation.json` with a non-`Current` decision and no worker artifacts reports `not_dispatched_binding_rejected`. Recovery never calls the port. Cancellation keeps `cancel.request` semantics and output (`cancellation_requested` / `termination_observed`). The port trait has no release method, so stopping the caller's wait cannot release an allocation.

## Decision 8: instance observations

`implementation-attempt inspect --request --state` prints `{ "profile": WorkerProfileFacts, "instance": WorkerInstanceObservation | null }` (schema `lattice.worker-instance-observation.v1`). The instance holds the profile revision, `work_id`, `attempt_id`, incarnation (intent identity), fence and binding identity. Its compute provenance omits `activation_backend_ref`, `grant_ref` and producer refs. It also holds model route identities and a lifecycle from Decision 6's artifacts: binding without intent → `admitted_not_dispatched`, revalidation rejection → `binding_rejected_before_dispatch`, intent without terminal result → `dispatch_recorded_outcome_unknown`, retained result → `terminal_observed { disposition }`. Cancellation is `not_requested`, `requested_unobserved` or `observed_terminated`. `host_reachability`, `model_readiness` and `allocation_liveness` are always `unknown`. The record also has source `lattice.implementation-attempt-state.v1`, host `observed_at_epoch_ms`, and `state_digest` (BLAKE3 over the identities of the present artifacts). Unreadable artifacts are an error, never a guessed state. The command is bounded to one attempt directory, takes no claim and schedules nothing.

## Decision 9: #13 compute provenance

`CommandWorkerObservation` gains `#[serde(default, skip_serializing_if = "Option::is_none")] execution_binding: Option<ExecutionBindingProvenance>`. Historical rows decode as `None` (unknown provenance). `Scope` gains `Attempt`. For `Scope::Attempt`, validation requires `execution_binding = Some`, `accounting = None` and `run_id = node_id = 0`. Field mapping: `workflow_revision` = envelope identity, `instance_id` = `work_id`, `dispatch_id` = `invocation_id` = intent identity, `worker_profile` = profile revision, `attempt_id` = envelope attempt. The store skips the `COMMAND_WORKER_USAGE_BY_RUN` index for `Scope::Attempt`, and the existing immutable-content comparison keyed by `attempt_id` handles replay and conflict.

```text
ExecutionBindingProvenance {
  schema: "lattice.execution-binding-provenance.v1",
  binding_identity, profile_revision, worker_incarnation, attempt_id, fence,
  compute: host_local | provider_allocation { allocation_id, consumer_id, owner_generation, placement,
           charge_cursor, kind, shared_serving_ref, reserved_claims [{pool_id, quantity}], enforcement_at_bind },
  model_routes: no_model | routes [{name, route_identity}],
  coverage: { worker_process: host_measured | unknown, model_requests: unknown, device_utilization: unknown,
              unknown_owner: { model_requests: "animus#17", device_utilization: "aspen#42" } },
}
```

Attempt receipts carry `provenance = HostMeasured` and `duration_millis` only when the worker observation gives both endpoints. Otherwise `provenance = Unknown` with no values. `tokens`, `cost_microunits`, `model_requests` and `subagent_calls` stay `None` (OMP `usage_complete=false`). Orchestration `HostRun` receipts keep `execution_binding = None` because no Agent-node binding exists. Their settlement code is not modified, and a regression test asserts identical charges.

## Acceptance mapping

| Criterion | This change | Blocking owner for the remainder |
|---|---|---|
| #24 CPU worker + separately hosted model, both bindings recorded | deterministic fixture (`admission.scenario.cpu_remote_model`) | live run: Kiln #49; serving readiness: Animus #17 |
| #24 two workers share a model service without duplicate GPU reservation | binding/usage fixtures (`shared_serving`, `usage_provenance.scenario.shared_allocation`) | per-request inference attribution: Animus #17 |
| #24 hosted-API and no-model without fabricated leases | `hosted_and_no_model` (no-model in core; OMP no-model is unsupported) | — |
| #24 direct-GPU worker not satisfied by CPU-only offer or healthy remote service | `direct_device` | — |
| #24 unsupported requirements fail before effects | compute/model parts (`profile_facts.scenario.unsupported`); direct-adapter limits via `project-configured-agent-bindings` | tool/modality task matching: Lattice #26 / Kiln #83 |
| #24 stale profile, expired lease, crossed attempt, wrong placement generation, unauthorized model-binding change rejected | `crossed`, `revalidation`, `changed_binding` | — |
| #24 inventory separates profiles from instances; stale/missing unknown | `instances.*` | joined presentation: Kiln #60 |
| #24 lost acks, disconnects, inference timeouts cannot relaunch or reroute | `recovery.scenario.lost_acknowledgement` | — |
| #24 cancellation requested vs observed; waiting does not release | `recovery.scenario.cancellation` | — |
| #24 usage reaches #13 with worker/service/request provenance | worker and compute provenance | service/request provenance: Animus #17 |
| #24 explicit Pi/Clankers/Animus compatibility; new executable not trusted | `profile_facts.scenario.compatibility` | — |
| #24 qualification retains schemas, profiles, executables, negatives; manifest ≠ live | `deferred.scenario.live` fixture evidence | live: Kiln #49; qualified promotion: Animus #18 |
| #13 bind observations to profile/incarnation/attempt/fence/allocation/placement | `usage_provenance.scenario.bound` | model-service/request identities: Animus #17 |
| #13 separate worker duration, model usage, device allocation | coverage dimensions | device utilization: Aspen (no readback field) |
| #13 shared deployment not charged per worker | `shared_allocation` | apportionment policy: Kiln #45 |
| #13 hosted/no-model without fabrication | `shared_allocation` | — |
| #13 multi-model per-service provenance | multiple routes modeled in core; OMP path rejects >1 | Animus #17 + a multi-route adapter |
| #13 cancellation/lost telemetry keep unknown remainder | unchanged conservative settlement; `coverage` unknown | — |
| #13 recovery dedup; relocation cannot reset or double charge | `usage_provenance.scenario.replay` | cross-attempt aggregation: Kiln #45 |
| #13 pricing identity | none | Lattice #13 (no producer pricing contract) |
| #24 authority-lifecycle (Secure Exclaves, restore) follow-ups | a binding never adds tools, endpoints or credentials | Animus #21, Lattice #12/#25 |

## Risks and open questions

1. Readback-v1 carries neither the evaluation tick nor per-claim pool kinds, so Lattice relies on a port-supplied `provider_logical_now` and optional offer facts at the allocation's basis revision. A producer readback that carries `observed_at` and the claim pool kinds would remove both port-trust points. Raise this with Aspen when #42 follow-up is scheduled.
2. Lattice cannot recompute producer `record_ref` values without a Preserves codec. Admitting one is a dependency and pin decision owned by the parent.
3. The v3 cutover requires the Kiln dispatcher and operator configuration to readmit requests. The reserving party must use the Lattice envelope `attempt_id` as the producer `attempt_id`, and must use globally unique operation ids, because of the producer's 128-entry replay horizon. `charge_cursor` equality detects a readmitted id.
4. `owner_generation` is compared with the dispatcher-declared expectation, not with the Lattice `fence`. Equating them would assume a Kiln/Aspen mapping that neither owner has published.
5. `project-configured-agent-bindings` still has an unchecked producer-adoption task covering compute and serving. The parent may retarget its compute half to this change.

## Verification (parent, after implementation)

Run `cargo test --locked` for the test files named in `tasks.md`, the existing `test_agent_profile_bindings`, `test_lattice_core_boundary`, `test_content_identity_adoption`, the orchestration usage and settlement tests and the implementation-attempt unit tests. Then run the native Cairn proposal, design and tasks gates and repository validation for this change, `scripts/check.rs`, Octet and the x86_64-linux Nix checks. No live provider, worker, model or host action is part of this change.

## Implementation notes

Owner decisions of 2026-11-18 settle the open questions: `provider_logical_now` stays optional and its absence blocks; producer references stay opaque and are compared exactly; the v3 request cutover follows the v1→v2 precedent; `owner_generation` is the dispatcher-declared expectation, never the fence; direct-accelerator requirements without offer facts block; the standalone port is `UnavailableComputePort`.

Octet's `path_segment_repetition` forbids item names that repeat an ancestor path word (`lattice`, `core`, `execution`, `binding`, module names), so the Rust names differ from the sketches above without changing any schema string:

| Design sketch | Implemented as |
|---|---|
| `ExecutionBindingDeclaration` | `execution_binding::request::Declaration` (`ComputeDeclaration`, `RouteDeclaration`, `NamedRoute`) |
| `Admission` / `AdmittedBinding` | `admission::Decision` / `admission::bound::Admitted` (`AdmittedCompute`, `AllocationProvenance`, `Placement`) |
| `BindingRejection` / `BindingBlock` | `admission::Rejection` / `admission::Block` |
| `Revalidation` | `revalidation::Recheck` |
| `binding_identity` | `admission::bound::identity`; role wrappers `content_identity::{execution_binding, model_route, worker_instance_state}` |
| `Option<&ComputeAllocationObservation>` admission input | `admission::ProviderReading` (`NotRead`, `Observed`, `NotFound`, `Unavailable`, `PortIssue`), so the port mapping is decided in the pure core |
| `ExecutionBindingProvenance` | `projection::accounting::AttemptProvenance` |
| unknown-fact classification | `admission::facts::classify` |
| `src/implementation_attempt/cli/binding.rs` | `cli/binding/mod.rs` with `cli/binding/usage.rs`; `inspect` is `cli/inspect/mod.rs` (Octet `module_file_count`) |

The compute variant tag is `compute_kind` in persisted bindings and provenance, because the allocation record already has a `kind` field. `prepare` normalizes the identity of a single declared `primary` route, like the other derived identities it writes.
