# Worker serving binding

This document describes how an implementation attempt binds its brokered model
requests to a serving owner's request admission, and how per-request usage
reaches the #13 usage observation. It covers the serving half of
[Lattice #24](https://github.com/OnixResearch/lattice/issues/24) and the usage
bindings of [Lattice #13](https://github.com/OnixResearch/lattice/issues/13).
The compute half is described in [worker compute binding](worker-compute-binding.md).
The Cairn change is `.cairn/changes/bind-model-serving-admission/`.

## Producer

Lattice consumes the pure serving core of
[Animus #17](https://github.com/OnixResearch/animus/issues/17),
`animus_adapters::serving`, vendored by `scripts/sync_animus.rs` at Animus
`1c370d5029a2667051c46bdfa1919e78e65e2f4b`. The core is `no_std` and reads no
clock, network, credential or health probe. Lattice calls these functions:

| Step | Producer API | What Lattice supplies |
|---|---|---|
| Profile | `Profile::new` | The declared profile id, identity rule, capacity rule, replica rule and maximum snapshot age |
| Screen | `screen_routes` | The declared route as one `RouteCandidate`, the port's snapshot and logical time, the newest snapshot position this attempt accepted, and the privacy band |
| Admission | `admission_query`, `gate_dispatch` | The caller binding (agent = worker profile revision, session = worker incarnation, attempt = attempt id), the owner's reply, the current recheck and the broker's own checks |
| Response | `response_provenance` | The response `model` and the provider's `usage` counters |
| Receipt | `project_receipt` | Every fact above; the receipt identity is retained |

The `animus-std` `serving-admission` feature (Mesh transport and JSON codec)
is not vendored or enabled. The attempt-local model broker is Lattice's
transport.

## Declaration

An execution binding may carry one optional `lattice.serving-binding.v1`
declaration in `execution_binding.serving`:

```json
{
  "schema": "lattice.serving-binding.v1",
  "profile": {
    "profile_id": "capacity-aware-observed-v1",
    "identity_rule": "observed_required",
    "capacity_rule": "known_required",
    "replica_rule": "same_model_allowed",
    "max_snapshot_age_ticks": 30
  },
  "privacy": "remote_allowed",
  "attribution": { "kind": "not_apportioned" },
  "routes": [
    {
      "name": "primary",
      "service_id": "svc-shared",
      "endpoint_identity": "endpoint:svc-shared",
      "model_id": "model-a",
      "configuration_ref": "blake3:<64 hex>",
      "requirements": {
        "needs_tools": true, "needs_streaming": false,
        "needs_structured_output": false, "needs_developer_role": false,
        "input_tokens": 2000, "output_tokens": 500
      }
    }
  ],
  "stop_on_overrun": true
}
```

The pure core validates it before any binding, intent or worker effect. Every
served route must be a declared model route. Identifiers must satisfy the
producer's opaque-token rule, and `configuration_ref` must be canonical.
Output tokens must be positive and the snapshot age between 1 and 3600 ticks.
A no-model binding cannot carry a declaration. The OMP attempt worker may
serve only its `primary` route, with the broker profile's exact model. The
`endpoint_identity` is an opaque token, never the URL.

`stop_on_overrun` is optional and defaults to `false`; see
[overrun policy](#overrun-policy).

The admitted binding freezes the declaration. The `lattice-execution-binding-v1`
identity frames it as a `serving-declaration` part only when it is present, so
every binding without one keeps its identity, file shape and behaviour. An
unset `stop_on_overrun` is omitted from the canonical declaration bytes, so
declarations written before the field existed keep their identity; a set
policy changes it.

## Port

```text
trait OwnerPort: Send {
  fn observe_snapshot(&mut self, query: &SnapshotQuery) -> SnapshotReply;  // Observed { snapshot, now_tick } | Unavailable | PortIssue
  fn admit_request(&mut self, query: &AdmissionQuery) -> OwnerReply;       // Granted | Rejected | Unavailable | PortIssue
  fn recheck(&mut self, query: &AdmissionQuery) -> Current;                // Observed(CurrentFacts) | Unavailable | PortIssue
  fn provenance(&self) -> PortProvenance { PortProvenance::Embedder }     // Fixture | Unavailable | Embedder
}
```

The port returns producer-validated values. Lattice never reserves, loads,
drains, releases or probes serving capacity. A grant is the serving owner's
decision about one exact request. The standalone CLI installs
`UnavailableOwner`, so a served route blocks before any upstream effect. An
embedding host supplies a live adapter through
`implementation_attempt::cli::run_command_with_ports`, with
`Ports { compute, serving }`. `FixtureOwner` replays scripted replies, records
every call and is fixture evidence only.

### Evidence label

Every record's `evidence` label is derived from the port's `provenance()` and
the outcome; nothing the port or the upstream returns can set it:

| Port provenance | Dispatched (admitted upstream call) | Refused or stopped before upstream |
|---|---|---|
| `Fixture` (`FixtureOwner`) | `fixture` | `fixture` |
| `Unavailable` (`UnavailableOwner`, the standalone CLI) | `no_upstream_contact` (cannot occur) | `no_upstream_contact` |
| `Embedder` (the default for an injected port) | `live_endpoint` | `no_upstream_contact` |

The Animus receipt carries `live-endpoint` only for an embedder-admitted
upstream call and `fixture` otherwise. `live_endpoint` attributes the call to
the embedder's port; Lattice does not authenticate that port.

## Per-request gate

The broker keeps its existing checks for every request: exact model, deadline,
admission expiry, request budget and host-side credential resolution. An
exhausted request budget refuses before any credential, gate or port work; on
a served route that refusal is recorded once as `budget-exhausted` (see
[overrun policy](#overrun-policy)). A request on a served route then passes
the gate:

0. With `stop_on_overrun`, an observed overrun refuses the request as
   `overrun-stopped` before any port call.
1. The request identity is `blake3:` over the binding identity, route and a
   per-route ordinal, so recovery can match it exactly.
2. The port observes a snapshot. If it is unavailable or the port fails, the
   request is refused as `snapshot-unavailable` or `snapshot-port-issue`.
3. `screen_routes` excludes a snapshot that is expired, too old, out of order
   or of unknown freshness. It also excludes a service that is absent or not
   ready (draining, loading, stale, unknown or unavailable) and a wrong model,
   configuration or claimed-only identity. Privacy, capability, envelope,
   missing owner admission and stale, unknown or exhausted capacity exclude
   too.
4. The serving owner is asked to admit the exact request, and only a grant is
   rechecked.
5. `gate_dispatch` rejects an owner rejection, a grant for another request or
   target, a replaced generation or endpoint, a substituted model or changed
   configuration, a not-ready recheck, an expired or revoked grant, and a
   replica change under the `pinned` rule. An unavailable owner, recheck or
   clock blocks dispatch.
6. Only an admitted verdict writes the dispatch intent, with fsync, and lets
   the broker contact the upstream endpoint.

A capacity snapshot is never a reservation. When two attempts both see the
last request slot, only the attempt with the owner's grant dispatches.

After the call, the response's `model` and `usage.prompt_tokens` /
`usage.completion_tokens` (JSON or server-sent events) become provider-reported
counters. A substituted or missing response model is `response-rejected`, and
the broker withholds that response from the worker. A timeout, transport
failure, cancellation or a response the broker withholds after credential echo
is `uncertain` with unknown usage. Nothing is retried or replayed, and no other
route, service or model is selected. If an observation cannot be retained, the
broker fails, because the upstream outcome may be unknown. The intent remains
as evidence.

## Evidence files

All request evidence lives under the attempt directory in `model-requests/`:

| File | Schema | Written |
|---|---|---|
| `<hex>.intent.json` | `lattice.model-request-dispatch.v1` | once, before upstream contact |
| `<hex>.record.json` | `lattice.model-request-observation.v1` | once, after a refusal or the call |

A record carries the binding identity, profile revision, worker incarnation,
attempt and fence, the route name and `lattice-model-route-v1` identity, the
request id and ordinal, and the service. It also carries the selected
endpoint, deployment and generation, the exact model and configuration with
attestation, capacity evidence, the replica and whether it changed, and the
serving resource-consumer reference. Finally it records the snapshot identity,
position and status, the admission record, the outcome class, label and
exclusion detail, the counters, the evidence label and the Animus receipt
identity. A different second write is refused.

## Usage for #13

Before the attempt result is published, the retained intents and records are
summarized into `coverage.model_requests` of the frozen
`lattice.execution-binding-provenance.v1`. That provenance travels in the
existing `Scope::Attempt` `lattice.command-worker-usage.v1` observation:

| Binding | `model_requests` coverage |
|---|---|
| No serving declaration | `"unknown"`, as before |
| No-model worker | `"no_model"`; no counters |
| Served routes | `{"served": {"schema": "lattice.model-request-usage.v1", "is_complete": …, "services": […], "overrun": {…}}}` |

Each service entry keeps the declared route, route identity, service, model
and configuration, and the attribution label. It also records dispatched,
not-dispatched and unknown-usage request counts and the known input and output
tokens, with every request's own deployment generation, replica,
resource-consumer reference, outcome and receipt identity.

- Known tokens are summed only from provider-reported counters. A dispatched
  request without both counters is counted as unknown usage. Unknown usage is
  never zero-filled.
- An intent without a record is `dispatch_unconfirmed` with unknown usage.
  Coverage is incomplete whenever any dispatched request lacks known usage or
  ended uncertain.
- Identical duplicate files count once. Conflicting records or intents, a
  dispatched record without its intent, an intent for a refusal and evidence
  crossed to another binding, attempt, fence, route or service are refused.
- The attempt observation is immutable. A reconnect, relocation or recovery
  that would save a different summary for the same attempt is refused, so
  settled usage cannot be reset.
- A shared deployment is referenced through its resource-consumer reference
  and labelled `not_apportioned`. A declared `declared_estimate` attribution
  carries only a versioned policy schema and reference. Lattice computes no
  apportioned amount.
- Hosted APIs bind host-local compute without a lease or claim. Per-request
  provenance is kept per service, so a worker calling two services reports
  two entries.

Recovery and `implementation-attempt inspect` report the same summary
read-only; recovery never calls the serving port. Orchestration settlement
arithmetic is unchanged. Top-level attempt counters stay absent, so `HostRun`
reservations and charges are unaffected.

## Overrun policy

Each served route's declared token envelope (`input_tokens + output_tokens`)
is the amount one dispatched request reserves. The summary's `overrun` block
keeps the reservation and the observation apart:

| Field | Meaning |
|---|---|
| `stop_on_overrun` | The declaration's policy |
| `reserved_tokens` | Envelopes of every dispatched request; a reservation, never usage |
| `known_tokens` | Provider-reported input plus output tokens of requests with both counters |
| `overrun_requests`, `overrun_tokens` | Known-usage requests that exceeded their own envelope, and by how much in total |
| `unknown_usage_requests` | Dispatched requests without both counters |
| `refused_requests` | Recorded `budget-exhausted` and `overrun-stopped` refusals |
| `is_exact_ceiling` | `true` only when every dispatched request reported usage within its envelope |

Each request is compared with its own envelope, so an under-used request
never offsets another request's overrun, and an unknown request's envelope is
never headroom. `is_exact_ceiling` is `false` whenever any usage is unknown or
any overrun was observed.

An overrun is always recorded and visible. Before each request reaches the
gate, the broker's session compares the known usage settled so far with those
envelopes. With `stop_on_overrun`, once any overrun is known, the next request
on a served route is refused as `overrun-stopped` with no snapshot,
admission or recheck call. Without it, requests continue and the overrun is
reported only. Unknown usage never triggers a stop by itself, and it never
counts as headroom.

The broker's request budget (`max_requests`) still refuses first, before any
credential, gate or port work. On a served route that refusal is a
not-dispatched `budget-exhausted` record. Each stop is recorded once per
route: later refusals reuse that record without a new ordinal or file, and an
identical replay of the file is idempotent and counted once.

These are execution controls over provider reports. A request already in
flight can still overshoot, and no envelope is a billing ceiling. Aggregate
budgets across attempts stay with
[Kiln #45](https://github.com/OnixResearch/kiln/issues/45).

## Inventory readiness

`implementation-attempt inspect` prints the profile's `serving_readiness`
and the instance's `model_readiness`, plus the attempt's `model_requests`
summary. Readiness reports the actual cause rather than an owner placeholder:

| State | When |
|---|---|
| `no_serving_port` | No serving-owner port was injected (the standalone CLI and configured runtime profiles) |
| `no_served_route` | A port was injected, but the binding declares no served route |
| `observed` | For each served route, the injected port's latest snapshot: `reading` `observed` with its identity, position, the service's `readiness` (or `not-listed`) and the producer's `freshness` label (`current`, `expired`, `too-old`, `freshness-unknown`), or `unavailable` / `port_issue` |

Inspection calls only `observe_snapshot`, once per served route, and never
admission or recheck. A snapshot read is not a reservation, and a draining or
stale observation is reported as such, never as ready. Freshness has no
accepted-position history here, so `out-of-order` cannot be reported.

## Compatibility: reservation-only versus serving-bound usage

| Mode | Applies to | Reserved | Measured | Model usage |
|---|---|---|---|---|
| Reservation-only | Orchestration Agent nodes (Pi, Clankers, Animus, direct OpenAI-compatible) and attempts without a serving declaration | Configured per-round charges settle conservatively | Host duration when both endpoints are observed | `unknown` |
| Serving-bound | Implementation attempts whose binding declares served routes | Unchanged | Host duration when observed | Per-request provider-reported counters with exact provenance; unknown remainder counted |

Provider counters are provider reports, not measurements or invoices. A
configured model name does not prove which weights were served. The core does
not verify provider counters, and no pricing or currency identity exists.

## Independence from compute

Serving admission never satisfies, replaces or weakens compute admission. A
direct-accelerator worker still needs its own provider allocation with
sufficient chips. A ready serving route with capacity does not admit it from a
CPU-only offer. Serving facts add no tool, endpoint, credential or disclosure
destination.

## Non-claims

- Every serving result in this repository comes from deterministic fixtures.
  Records carry `fixture` or `no_upstream_contact`. The one test that wraps
  fixture replies in a default-provenance port checks only how
  `live_endpoint` is derived; no endpoint was contacted. No live serving
  owner, endpoint, provider, replica or device was exercised. Live
  qualification belongs to
  [Kiln #49](https://github.com/OnixResearch/kiln/issues/49).
- The standalone CLI's serving port is unavailable. A served route therefore
  never dispatches without an embedder-supplied port.
- Animus receipts carry the producer's non-claims: not GPU scheduling,
  production availability, physical placement truth, model correctness,
  capacity reservation, provider honesty, or cost or pricing.
- Device utilization stays `unknown` with owner `aspen#42`. Aggregate budgets
  belong to [Kiln #45](https://github.com/OnixResearch/kiln/issues/45), and
  joined presentation to [Kiln #60](https://github.com/OnixResearch/kiln/issues/60).
