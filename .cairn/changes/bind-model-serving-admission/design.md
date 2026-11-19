# Design: Brokered model requests bound to serving-owner admission

## Context

Baseline: Lattice `661499d282c2e79ebefbd0f9b35beb5f8d7795cd`. Producer: Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b` (`365b0ee6` + Animus #15 `c5fe338e` + Animus #17 `1c370d50`).

What already exists:

- `bind-worker-compute-allocations` admits one exact `lattice.admitted-execution-binding.v1` per implementation attempt: profile revision, attempt and fence, host-local or provider compute, and named model routes (`primary` for the OMP worker). It persists `execution-binding.json` before `intent.json`, revalidates before spawn and freezes `lattice.execution-binding-provenance.v1` into a `Scope::Attempt` usage observation. Model-request coverage is `unknown` with owner `animus#17`.
- The candidate-worker model broker (`src/candidate_worker/broker/`) is attempt-local: it listens on a private Unix socket, accepts only `POST /v1/chat/completions` for the admitted exact model, enforces a request budget, deadline and expiry, resolves the credential on the host and forwards to one admitted endpoint.
- Lattice vendors `animus-adapters` (pure `no_std`) and its dependencies through `scripts/sync_animus.rs`.

What Animus #17 publishes in `animus_adapters::serving` (pure, no clock or I/O):

| Step | API | Output |
|---|---|---|
| Snapshot | `AvailabilitySnapshot::new(&SnapshotDraft)` | validated `animus.serving-availability-snapshot.v1` with identity |
| Profile | `Profile::new(ProfileDraft)` | `animus.serving-capacity-profile.v1` with identity |
| Screen | `screen_routes(&ScreenRequest)` | eligible routes and typed exclusions |
| Admission | `admission_query`, `gate_dispatch(&GateInput)` | `Admitted(DispatchPermit)`, `Rejected`, `Blocked` |
| Receipt | `project_receipt(&ReceiptInput)` | `animus.serving-invocation-receipt.v1` with identity |

The `animus-std` `serving-admission` feature adds a Mesh transport and JSON codec. Lattice does not vendor `animus-std` and does not need either: the broker is Lattice's transport.

## Goals and non-goals

Goal: every brokered request on a declared served route passes the producer's pure gate before any upstream effect, leaves durable exact per-request evidence and reaches #13 as per-service usage, without changing compute admission or settlement.

Non-goals: being the serving owner, reserving capacity, routing among replicas, pricing, Agent-node serving admission and live qualification.

## Decision 1: vendor once, consume only the pure core

Both open Lattice changes (#23 typed policy and this one) re-vendor the same Animus revision with the same crate set: the existing `REQUIRED_CRATES` plus `animus-frozen-policy`. `animus-adapters` has no `serving` feature gate, so no Animus feature changes. `animus-std` and its `serving-admission` feature stay out; the vendored snapshot must be byte-identical between the two changes.

## Decision 2: an optional declaration on the existing binding

`Declaration` gains `serving: Option<ServingDeclaration>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`). The admitted binding carries the same field. `content_identity::execution_binding` appends a tagged `serving-declaration` part (canonical JSON of the frozen declaration) only when present, so all existing identity vectors and persisted bindings remain valid. The declaration is Lattice-owned and validated in the pure core with the producer's token rule mirrored, so an unsupported declaration fails before any binding file is written; the shell then builds the Animus `Profile` and `RouteCandidate` values, and a producer contract issue there is also a pre-effect refusal.

```text
ServingDeclaration {
  schema: "lattice.serving-binding.v1",
  profile: { profile_id, identity_rule, capacity_rule, replica_rule, max_snapshot_age_ticks },
  privacy: local_only | remote_allowed,
  attribution: not_apportioned | declared_estimate { policy_schema, policy_ref },
  routes: [ { name, service_id, endpoint_identity, model_id, configuration_ref,
              requirements: { tools, streaming, structured_output, developer_role,
                              input_tokens, output_tokens } } ],
  stop_on_overrun?: bool,   // omitted from canonical bytes when false
}
```

The OMP attempt request may serve only its `primary` route, and the served `model_id` must equal the broker profile's exact model; otherwise the request is refused before effects. The served `endpoint_identity` is an opaque token and never the endpoint URL; the broker's admitted endpoint is unchanged.

## Decision 3: the port returns producer-validated values

Animus is already a Lattice dependency, so `ServingOwnerPort` returns producer types instead of a duplicate DTO:

```text
trait ServingOwnerPort: Send {
  fn observe_snapshot(&mut self, query: &SnapshotQuery) -> SnapshotReply;      // Observed { snapshot, now_tick } | Unavailable | PortIssue
  fn admit_request(&mut self, query: &AdmissionQuery) -> OwnerReply;           // Animus
  fn recheck(&mut self, query: &AdmissionQuery) -> CurrentObservation;         // owned CurrentState | Unavailable | PortIssue
}
```

`UnavailableServingPort` answers every call unavailable. `FixtureServingPort` replays scripted replies and records calls. An embedding host (Kiln #49) supplies a live adapter through `implementation_attempt::cli::run_command_with_ports`.

## Decision 4: the gate session

`ServingSession` holds the frozen declaration, the Animus profile and candidates, the caller binding (`agent` = profile revision, `session` = worker incarnation, `attempt` = attempt id), the newest accepted snapshot position, per-route ordinals and a durable sink. For one request it:

1. assigns `request_id = blake3(binding identity, route, ordinal)` (a canonical `blake3:` token);
2. observes a snapshot; unavailable or port issues block;
3. builds a fresh `Screening` with `Freshness { now_tick, newest_accepted }`; no eligible route records `not-dispatched/no-eligible-route`;
4. derives the `AdmissionQuery`, asks the owner and rechecks current state;
5. runs `gate_dispatch` with `DispatchAdmission` from the broker's own checks (exact model, request budget, deadline);
6. on `Admitted`, writes the dispatch intent and returns a ticket; otherwise projects the receipt and writes a `not-dispatched` observation.

After the upstream call, the session maps the broker result to `Completed` or `ResponseRejected` (via `response_provenance` over the response `model` and provider `usage`), or `Uncertain(TimedOut | TransportFailed | Cancelled)`, projects the receipt and writes the observation. A snapshot position is accepted only after a successful screen, so an older snapshot is excluded as `out-of-order`.

## Decision 5: broker integration through a narrow gate trait

`candidate_worker::broker` gains `ModelRequestGate` with `admit(BrokerChecks) -> Option<Ticket>` and `settle(Ticket, UpstreamResult) -> Result<(), String>`. `DirectGate` (no serving declaration) admits without records, which preserves today's behaviour. Credential resolution and the currentness check move before the gate so a grant is never obtained for a request the broker would then refuse. A failed settlement fails the broker like an undelivered admitted response: the upstream outcome may be unknown and the intent remains as evidence.

## Decision 6: usage summary in the pure core

`serving::summarize(admitted, intents, observations)` is pure. It refuses crossed, conflicting and intent-less dispatched observations, collapses identical duplicates, adds `dispatch-unconfirmed` entries for intents without observations and groups per declared route and service. Coverage becomes:

```text
ModelRequestCoverage = unknown            // no serving declaration (legacy spelling kept)
                     | no_model
                     | served { schema: "lattice.model-request-usage.v1", complete, services: [...], overrun }   // Decision 8
```

The attempt observation's top-level counters stay `None`, so `Scope::Attempt` validation and settlement arithmetic are unchanged; the immutable attempt row is the dedup and anti-reset mechanism. Shared deployments are referenced through `resource_consumer_ref` and labelled `not-apportioned`; an explicit policy reference is labelled `declared-estimate` and carries no Lattice-computed number.

## Decision 7: evidence label from port provenance

`OwnerPort::provenance()` returns `Fixture`, `Unavailable` or `Embedder` (the default for any injected port). A record is `fixture` for the fixture port, `live_endpoint` only for a request an embedder port admitted to upstream, and `no_upstream_contact` otherwise. The Animus receipt label is `live-endpoint` for the same case and `fixture` otherwise. Nothing a port or upstream returns can set the label; port authenticity stays with the embedder.

## Decision 8: overrun block and stop policy

The "envelope budget" of #13 is the declared per-request token envelope of the served route (`input_tokens + output_tokens`), because the attempt envelope's `AttemptBudget` has no token dimension and no pricing identity exists. `serving::envelope::Overrun` is pure and shared by the summary and the gate session. Each dispatched request reserves its envelope. Only requests with both provider counters add known tokens, and each is compared with its own envelope, so neither an under-used request nor an unknown request's envelope offsets an overrun. `is_exact_ceiling` is false whenever any usage is unknown or any overrun was observed.

The gate session accounts every settled dispatched record. Before the gate, with `stop_on_overrun`, a known overrun refuses the request as an unscreened `overrun-stopped` record without any port call. The broker's existing early request-budget refusal calls `ModelRequestGate::exhausted`; `DirectRoute` does nothing, and `OwnerGate` writes an unscreened `budget-exhausted` record. Each stop is written once per route and label, and later refusals reuse it without a new ordinal or file. The summary counts both labels as `refused_requests`. `stop_on_overrun` is `#[serde(default, skip_serializing_if = "Not::not")]`, so existing declarations keep their canonical bytes and binding identity.

## Decision 9: inventory readiness from the injected port

`projection::ServingReadiness` replaces the `OwnedUnknown { unknown, animus#17 }` placeholder in `WorkerProfileFacts.serving_readiness` and `WorkerInstanceObservation.model_readiness`. Its states are `no_serving_port` (port provenance `Unavailable`, and configured runtime profiles), `no_served_route`, or `observed` with one `SnapshotReading` per served route. The shell's `serving::readiness::observe` makes one `observe_snapshot` call per route. It screens the snapshot with the declared profile and a route candidate without a route reference, reporting the producer freshness label, and it takes the service readiness from the snapshot (`not-listed` when absent). It never calls admission or recheck. `implementation-attempt inspect` receives the serving port from `run_command_with_ports` and also prints the retained `model_requests` summary. This supersedes the readiness part of `lattice.execution_binding.deferred.scenario.serving`; usage provenance without a serving declaration keeps the `animus#17` owner.

## Risks

- Port authenticity is attributed to the embedding host; Lattice validates shapes and identities only.
- Provider token counters are provider reports; the core does not verify them.
- Producer revision is unpublished at authoring time; the parent re-syncs if the producer gate forces a new revision.

## Verification

Focused core and shell tests over the fixture port, the existing reservation, ledger, execution-binding and broker suites, clippy `-D warnings`, fmt, Cairn validate and gates, Octet semantic delta and `nix build .#checks.x86_64-linux.nextest`. No live serving owner, endpoint or provider is exercised.
