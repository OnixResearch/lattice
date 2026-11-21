# Worker compute binding

An implementation attempt binds exactly one compute provenance and its declared
model routes before the worker starts. The compute is either host-local or one
provider-owned execution allocation published by Aspen/Molten. Lattice consumes
the producer's records only as its own consumer DTO, reads them through an
injected read-only port, and never reserves, activates, drains, releases,
retires or probes compute.

The change package is `.cairn/changes/bind-worker-compute-allocations/`.
It covers the compute half of
[Lattice #24](https://github.com/OnixResearch/lattice/issues/24) and the
compute provenance of [Lattice #13](https://github.com/OnixResearch/lattice/issues/13).

## Producer specification

The producer is Aspen at `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4`
([Aspen/Molten #42](https://github.com/brittonr/aspen/issues/42)). Lattice reads
these sources as a specification only:

- `docs/compute-capacity.md`
- `config/compute-capacity/contracts.ncl`, `config/compute-capacity/profile.ncl` and the positive `agent-a-serving-b` fixture
- `crates/molten-core/src/compute_capacity/mod.rs`
- `src/compute_capacity/canonical/{allocations,readbacks,offers,enums}.rs`

Lattice adds no Aspen crate, Preserves codec, Cargo or Nix input, or
sibling-relative path for this boundary.

## Consumer DTO

`lattice.compute-allocation-observation.v1` lives in
`crates/lattice-core/src/execution_binding/model.rs`. Every struct denies
unknown fields. Field names are snake_case; enum values keep the producer's
kebab-case spellings. The serialized DTO is bounded to 65,536 bytes; lists hold
at most 128 items; tokens are 1 to 256 bytes without control characters.

| DTO part | Content |
|---|---|
| `schema` | exactly `lattice.compute-allocation-observation.v1` |
| `producer` | exact `compute-capacity-allocation-v2`, `compute-capacity-readback-v1`, `compute-capacity-offer-v2`, `molten.compute.reservation.v1`, `molten.compute.offer.v1` and time domain `logical` |
| `port` | `port_id`, `port_profile_ref`, `observation_sequence` (> 0, strictly increasing per port) and optional `provider_logical_now` |
| `allocation` | the 24 fields of `compute-capacity-allocation-v2` |
| `readback` | host fields of `compute-capacity-readback-v1` and the one matching row |
| `offer` | optional claimed-pool facts of `compute-capacity-offer-v2` at the allocation's offer revision |

### Allocation fields

| # | DTO field | Rule |
|---|---|---|
| — | `record_ref` | canonical `blake3:` reference |
| 0 | `schema` | `molten.compute.reservation.v1` |
| 1 | `id` | token, equals `operation_id` |
| 2 | `host_id` | token |
| 3 | `host_incarnation` | token (placement generation) |
| 4 | `device_incarnation` | token (placement generation) |
| 5 | `offer_revision` | > 0 |
| 6 | `topology_generation` | > 0 (placement generation) |
| 7 | `charge_cursor` | > 0; reservation-instance identity |
| 8 | `charge_at` | logical tick |
| 9 | `valid_until` | > 0 logical tick |
| 10 | `authority_ids` | tokens, one per claim |
| 11 | `view_id` | token |
| 12 | `granularity` | `chip`, `mmio-bundle`, `tray`, `mesh`, `partition` or `alias` |
| 13 | `consumer_id` | token |
| 14 | `operation_id` | token |
| 15 | `attempt_id` | token; must equal the Lattice attempt |
| 16 | `owner_generation` | > 0; compared with the dispatcher-declared expectation, never with Lattice's `fence` |
| 17 | `grant_ref` | canonical reference; a reference, not a grant |
| 18 | `claims` | 1 to 128 `{pool_id, quantity > 0}`, distinct pools |
| 19 | `serving_allocation_id` | optional token; absent for serving deployments |
| 20 | `kind` | `execution` or `serving-deployment` |
| 21 | `status` | `intent`, `active`, `quarantined` or `released` |
| 22 | `enforcement` | `logical-fence-only` < `backend-acknowledged` < `backend-fenced` |
| 23 | `activation_backend_ref` | present iff `active`, absent for `intent`; persisted with the admission observation only |

### Readback and offer facts

| DTO field | Rule |
|---|---|
| `readback.record_ref` | canonical reference |
| `readback.host_id` | equals the allocation host |
| `readback.revision` | > 0 and at least the allocation's offer revision |
| `readback.is_fresh`, `readback.health` | `false` or non-`healthy` blocks admission |
| `readback.qualification_refs` | distinct references; qualification scope only |
| `readback.row_count` | 1 to 128; other consumers' rows are not retained |
| `readback.row.*` | id, consumer, owner generation, granularity, status and enforcement equal the allocation; `is_logically_fenced == (status != released)` |
| `offer.host_id`, `host_incarnation`, `device_incarnation`, `revision`, `topology_generation` | equal the allocation (`revision == offer_revision`) |
| `offer.health`, `offer.label_authority` | recorded |
| `offer.compatibility` | optional backend, runtime, architecture and topology tokens; 1 to 128 qualification references |
| `offer.claimed_pools` | one `{pool_id, kind, authority_id}` per claim; kinds `dedicated-memory-bytes`, `unified-memory-bytes` or `chips`; the authority set equals `authority_ids` |

## Read-only port

`lattice::execution_binding::ComputeAllocationPort` has one method,
`observe_allocation`. It returns `Observed`, `NotFound`, `Unavailable` or a
`ComputePortIssue` (transport, timeout or malformed reply, carrying at most 256
bytes of provider text). `NotFound` rejects; `Unavailable` and every issue
block. The standalone CLI installs `UnavailableComputePort`, so a request that
declares a provider allocation stops before any binding, intent or worker
effect. An embedding host supplies a real adapter through
`implementation_attempt::cli::run_command_with_ports`.
`FixtureComputePort` replays scripted replies and records every query for
deterministic tests only.

## Admission and revalidation

The pure core in `crates/lattice-core/src/execution_binding/` admits one
binding from the request's declaration, the profile requirement, the attempt
identities and at most one port reading. Definite mismatches are `Rejected`;
unknown or not-yet-current facts are `Blocked`. Neither permits an effect, and
a blocked allocation is never retried into another allocation.

- The declared compute variant must match the requirement. Host-local
  bindings make no port call and carry no allocation, claims or placement.
- A provider allocation must be an `execution`, match the declared id, host,
  consumer and owner generation, carry the Lattice attempt id, be `active`,
  have known provider logical time below `valid_until`, and have a fresh,
  healthy readback.
- When `provider_logical_now` is absent, expiry is unknown and admission
  blocks. Lattice never compares producer logical time with its wall clock.
- A direct-accelerator requirement counts only `chips` claims of the
  allocation itself. It also needs an accepted granularity, equal compatibility
  fields and at least the required enforcement. Without offer facts the device
  kind is unknown and admission blocks. The referenced serving deployment never
  satisfies it.
- `serving_allocation_id` is kept as the opaque `shared_serving_ref`. Lattice
  never reads, reserves, copies or charges the serving deployment's claims.

Before spawn, the shell re-observes a provider allocation. Revalidation requires
equal placement generation, owner fields, attempt, kind, claims, view,
granularity, shared-serving reference, charge cursor and charge time; a
strictly greater observation sequence from the same port; an `active`,
unexpired allocation; a fresh, healthy readback; and enforcement no weaker than
at admission. A changed charge cursor identifies a reservation readmitted after
the producer's 128-entry replay horizon.

## Identity

The binding identity is `blake3:<hex>` under domain
`lattice-execution-binding-v1`, framed by `content_identity_core`. It covers the
profile revision, work, attempt, fence and envelope identities, the compute
kind and, for provider allocations, allocation id, consumer, owner generation,
host and device incarnations, topology generation, charge cursor and time,
kind, view, granularity, sorted claims, shared-serving reference and port id,
and the sorted model routes. Sequence, logical time, readback revision,
freshness, status, enforcement and producer record references are excluded.

The OMP attempt's `primary` route identity uses domain `lattice-model-route-v1`
over the broker schema, endpoint, model and credential reference (provider, key,
field). Expiry and credential values are excluded, and the endpoint text is
never projected.

## Attempt state lifecycle

Within one attempt state directory:

1. An existing `intent.json`, or `recover`, reconciles retained evidence only.
   It makes no port call.
2. The request is validated and admitted. A refused or blocked decision is
   printed, and nothing is written.
3. `execution-binding.json` (`lattice.persisted-execution-binding.v1`, the
   binding plus the admission observation) is written once with fsync. A
   repeated run must re-admit the same binding identity; a different binding is
   refused and the file is never overwritten.
4. `intent.json` is written once with fsync.
5. A provider binding is revalidated. Any result other than current is written
   once to `binding-revalidation.json`, and the worker never spawns. Recovery
   then reports `not_dispatched_binding_rejected`.
6. The worker runs under the existing cancellation watcher.
7. The `Scope::Attempt` usage observation is persisted, then the attempt
   result is published.

Recovery reports the persisted binding identity with its reconciliation result.
Cancellation keeps `cancel.request` semantics: `cancellation_requested` is not
`termination_observed`, and waiting stops nothing in the provider. The port has
no release method.

## Profile and instance facts

`lattice.worker-profile-facts.v3` reuses the existing profile owners: the
`ConfiguredBinding.revision` of configured Pi, Clankers and Animus routes
(`RuntimeOptions::worker_profile_facts`), and the attempt's
`worker_profile_identity` for the OMP worker. Each fact has exactly one support
level: `declared`, `qualified`, `unavailable` or `unknown`. Only
`task_families` can be `qualified`, and only when every declared family has a
current lifecycle qualification of the profile's revision, listed in
`capability_qualifications`. CPU, memory and storage bounds are `unknown`.
Configured routes report compute and model-service bindings as `unavailable`
and executable bytes as `unknown`. Each profile also carries a definition
identity that is stable across revisions, declared `task_families` and `tools`
(`unknown` when undeclared), Lattice #25's revision slots (`unmanaged` without
a revision store or lifecycle, otherwise `recorded` or `unknown` with an
owner; always `unmanaged` for configured routes), and wait reasons; see
[worker inventory](worker-inventory.md).

`lattice implementation-attempt inspect --request REQUEST --state STATE
[--revision-store DB]` prints one `lattice.worker-inventory.v3` document with
separate `profile` and `instance` sections. The
`lattice.worker-instance-observation.v3` instance carries the definition
identity, profile revision, work, attempt, incarnation (the request identity),
fence, binding identity, compute provenance without backend handles or grant
references, model route identities, the `pinned` revision slot of its session,
and one lifecycle: `admitted_not_dispatched`,
`binding_rejected_before_dispatch`, `revision_blocked_before_dispatch`,
`dispatch_recorded_outcome_unknown` or `terminal_observed`. Cancellation is
`not_requested`, `requested_unobserved` or `observed_terminated`. Single-shot
liveness is `not_started`, `exited` or `unknown` (`unobserved`, or `stale` past
the admitted window), never idle or live. Host reachability and allocation
liveness are always `unknown`. Model readiness names its actual cause
(`no_serving_port` when no serving-owner port is injected) or carries the
injected port's latest snapshot readiness and freshness; see
[inventory readiness](worker-serving-binding.md#inventory-readiness). A
missing intent binding, an unreadable artifact, a malformed capability
declaration, a named revision store that does not exist, or a
lifecycle-managed attempt inspected without its revision store is an error,
never a guessed state. Inspection takes no claim, schedules nothing and writes
no revision store.

## Usage coverage

`CommandWorkerObservation` may carry `execution_binding`
(`lattice.execution-binding-provenance.v1`). Historical rows decode without it,
which is unknown provenance. `Scope::Attempt` receipts require it, carry no
accounting, use zero run and node ids, and skip the run index while keeping the
immutable attempt-keyed replay and conflict rules.

| Dimension | Coverage |
|---|---|
| Worker process | `host_measured` with a duration only when the host observed both endpoints; otherwise `unknown` |
| Compute | reserved claims labelled `reserved`; never measured device use |
| Model requests | `unknown` without a serving declaration; `no_model` for no-model workers; per-service `served` request usage for served routes (see [worker serving binding](worker-serving-binding.md)) |
| Device utilization | `unknown` (owner `aspen#42`; no readback field) |

Orchestration `HostRun` receipts carry no execution binding, and their
settlement is unchanged.

## Deferred capabilities and owners

| Capability | Owner | Lattice reports |
|---|---|---|
| Serving readiness, request admission and per-request model-service usage | [Animus #17](https://github.com/OnixResearch/animus/issues/17), consumed in [worker serving binding](worker-serving-binding.md) | gated and recorded for declared served routes; `unknown` otherwise; no live serving owner qualified |
| Live host, device and cross-host qualification; a live provider port | [Kiln #49](https://github.com/OnixResearch/kiln/issues/49) with Aspen #15/#20 | not claimed |
| Qualification evidence that promotes `declared` to `qualified` | [Animus #18](https://github.com/OnixResearch/animus/issues/18) attachments, admitted at promotion ([agent revisions](agent-revisions.md#qualification-attachments)) | `task_families` `qualified` with `capability_qualifications`, including `fixture` provenance; no live producer exercised |
| Aggregate budgets and shared-deployment apportionment | [Kiln #45](https://github.com/OnixResearch/kiln/issues/45) | not computed |
| Joined inventory presentation | [Kiln #60](https://github.com/OnixResearch/kiln/issues/60) | `lattice.worker-inventory.v3` profile and instance facts with revision slots and wait reasons ([worker inventory](worker-inventory.md)) |
| Revision promotion and pinning | [Lattice #25](https://github.com/OnixResearch/lattice/issues/25), [agent revisions](agent-revisions.md) | revision slots from the lifecycle with `--revision-store`, `unmanaged` without it |
| Checkpoint and restore authority | [Lattice #12](https://github.com/OnixResearch/lattice/issues/12) | `unavailable` |
| Pricing and currency identity | [Lattice #13](https://github.com/OnixResearch/lattice/issues/13) | none; no producer contract |

## Non-claims

- Producer record references are opaque canonical `blake3:` identifiers.
  Lattice validates their shape and compares them exactly. It does not decode
  Preserves or recompute them; authenticity of a projection is attributed to
  the recorded port identity.
- The producer owner generation is not Lattice's fence. The dispatcher declares
  the expected owner generation, and a mismatch rejects.
- A binding never adds tools, endpoints, credentials or compute authority.
- Synthetic fixtures under `tests/fixtures/execution-binding/` and the fixture
  port are not live capacity, enforcement or serving evidence. The retained
  evidence in the change package states `live_qualification: false` and
  `backend_enforcement_observed: false`.
