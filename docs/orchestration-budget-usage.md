# Orchestration budget usage

## Production settlement and recovery

`StartRound` / `RetryRound` reserve `planned_charge` before dispatch. Agent-node
count is a conservative invocation reservation, **not** the number of model
requests, subagents, physical GPUs, or observed worker processes.

The orchestration shell persists a `host_run` command-worker observation before
settling either normal completion or recovery. Its optional `accounting` member
has schema `lattice.host-run-accounting.v1` and retains:

- `reserved`: the exact admitted reservation;
- `charged`: the frozen conservative settlement amount, not a measurement;
- `host_interval`: host run start/finish epoch milliseconds, when both exist;
- `provider_reports`: bounded per-node OpenAI-compatible reports, separate from
  host measurements and conservative charges;
- explicit unknown token, cost, model-request and subagent coverage, plus the
  qualification diagnostic.

Each provider report may also retain `configured_runtime`, with schema
`lattice.configured-agent-runtime.v1`, the existing target and the exact host
runtime configuration revision. The production adapter attaches it outside the
model response. Its revision is exactly the existing `Adapter::execution_identity`
used by admission; this addition does not change that identity or grant authority.
The configuration digest conservatively covers the full runtime options, including
other configured targets. Dynamic request inputs do not change it. It does not
authenticate executable bytes, served model weights, worker ownership or placement.

The existing receipt store freezes this optional binding with all other usage
content. Exact duplicate writes remain idempotent, restart retains it, and changing
it conflicts rather than rebinding previously settled usage. Historical reports
without it remain readable with unknown configuration provenance. A malformed
binding or a target crossed against the exact workflow Agent node cannot establish
a report; its usage remains under the admitted conservative fallback, not zero.

Duration is elapsed **owned workflow-run wall time**, in milliseconds. It is not
worker CPU time, per-process duration, memory residency, or accelerator time.
Negative, overflowing, missing, or interrupted intervals retain the duration
reservation. A valid terminal interval can replace only an enforced duration
dimension. Recovery checks that interval against the exact owned run record.

Tokens use provider-reported total tokens or consistent input/output token sums.
A valid total is retained when only one component is present and does not exceed
the total; two present components must sum exactly to it. Invalid, negative,
overflowing or conflicting fields invalidate the token report. Reports can
increase an enforced charge beyond the reservation, but incomplete coverage
cannot lower it: charge = max(reservation, known reported tokens).
Provider microunit cost is retained separately; the existing
adapter has no currency/pricing identity, so that value cannot replace the
admitted monetary charge. It is not an authoritative invoice or a hard ceiling.

Current production emits `RunFinishedObserved` with actual owned cancellation
evidence and `conservative_observation` as `last_usage_basis`. Historical
`RunFinished`, `RunFinishedMeasured`, and `CancellationObserved` retain their
original semantics solely for durable replay, not dispatch-time fallback. An
observed cancellation settles before closing ownership; a successful observation
wins a late cancellation request. A failed/requested round cannot become retry
evidence. The event variant name alone is not measurement evidence. `budgets.consumed`
is charged budget; `budgets.reservations` is still-pending budget. The store's
`load_command_worker_usage_for_run` returns the per-dispatch receipt for consumers
such as Kiln; it does not aggregate campaign budgets or invent Kiln attempt IDs.

The evidence commit precedes the separately atomic ledger/event transition.
After a crash in between, recovery reuses the identical frozen usage receipt
rather than recomputing a cheaper amount from later telemetry. Exact duplicate
receipt writes are idempotent. Only owned outcome evidence may monotonically
refine absent cancellation telemetry into a known value; that atomic store
update preserves every usage field, including the reservation and charge.
Conflicting known outcomes, changed frozen content, and crossed workflow,
instance, dispatch, run or reservation bindings block settlement. Stale partial
writes cannot erase known evidence. A committed settlement clears the pending
dispatch, so repeated recovery does not charge it again. Store errors are
propagated, not converted into missing/zero usage.

A terminal `RunRecord` alone does not establish observed non-cancellation. If
cancellation telemetry is absent, recovery returns `PendingUnobserved`, keeps
the exact pending dispatch and reservation plus known/fallback receipt, and
does not manufacture `false`, close the unknown outcome as failure, refund
budget, or admit a replacement. If recovery sees the terminal record before the
live producer returns its typed outcome, that later exact owned observation
can refine the receipt and permit settlement, including after restart. Until
such evidence exists, record-only historical recovery remains pending.

## Uncertainty and malformed telemetry

Missing, truncated, malformed, unsupported and partial worker telemetry is not
zero and is not a refund. Failure, cancellation, timeout and partial execution
retain known provider reports plus unknown remainder. Missing host observations
produce an explicitly conservative receipt, not a synthetic measured receipt.
A pending run without terminal evidence remains governed by existing recovery
and authorized abandonment rules; a missing observation does not authorize retry.

Receipt JSON rejects unknown fields, duplicate struct fields, unsupported schema,
invalid identities, invalid host intervals, duplicated provider-node reports and
inputs over 1 MiB. A legacy worker receipt without host accounting remains
readable, but cannot silently override host settlement: it requires explicit
reconciliation and leaves the reservation pending. It is not automatically
relabeled as authoritative provider/host measurement. Provider metadata is
accepted only from a completed, uncached Agent node's exact metadata port with
the existing OpenAI-compatible adapter marker; arbitrary model output is not a
usage source. The production headless path persists
`run::history::detail_record_from_results_with_completion_order`, which splits
pretty-printed Agent JSON into ordered, port-labeled rows. Accounting reassembles
all those rows with newlines, preserving their order and whitespace, before
parsing. The reconstructed document must fit the 1 MiB bound: it is rejected,
never truncated, on overflow. Crossed port identities, duplicate node details,
multiple JSON documents, malformed/truncated metadata, and cached results do not
establish fresh provider calls.

An observed token or duration overrun is retained and reaches the existing
ledger breach transition, which exhausts the instance and prevents further
admission. Unsupported monetary units cannot establish a comparable monetary
overrun. Post-hoc reports and conservative estimates do not enforce provider
billing: in-flight requests may continue or overshoot. Adapter request and
deadline limits are execution controls, separate from these accounting facts.

## Worker and inference qualification prerequisites

**Orchestration command workers remain reservation-only.** The command adapter
does not supply a qualified worker telemetry protocol; existing direct
OpenAI-compatible metadata has model labels but lacks exact service, request,
worker-incarnation, attempt/fence, allocation/placement and pricing bindings. No
self-declared profile/service field establishes qualification, and no new
producer schema is invented by this accounting boundary.

**Measured model-request usage exists only for implementation attempts.** An
implementation attempt whose execution binding declares served model routes
runs Animus's pure serving gate per request in its attempt-local model broker
and records per-request provider-reported counters with exact worker, binding,
service, deployment and request provenance in its `Scope::Attempt` usage
observation. An attempt without a serving declaration has its broker, the
worker's only network egress, record every relayed request without service
attribution (`brokered_unattributed`, Lattice #31). Unknown usage is counted,
never zero-filled, and shared deployments are referenced, never charged per
worker. When either ledger is complete, meaning every intent has a terminal
record and no request ended uncertain, the attempt's Kiln lineage draw charges
exactly the dispatched requests with `host_measured` provenance instead of the
broker ceiling; otherwise it charges the ceiling as `unknown`. See
[worker serving binding](worker-serving-binding.md) for the contract and
[worker compute binding](worker-compute-binding.md) for the compute half.

| Mode | Reserved | Measured | Model usage |
|---|---|---|---|
| Reservation-only (Agent nodes; attempts run before brokered evidence existed) | conservative configured charges | host duration when both endpoints are observed | `unknown` |
| Brokered (attempts without a serving declaration) | one broker ceiling per step; the exact count is charged when the ledger is complete | host duration when observed; every relayed request | `brokered_unattributed` per-request counters, no service attribution |
| Serving-bound (attempts with served routes) | declared per-request token envelopes, labelled `reserved_tokens` | host duration when observed | provider-reported per-request counters plus an explicit unknown remainder |

Provider counters are provider reports, not measurements or invoices; no
pricing or currency identity exists. No live serving owner, endpoint or device
has been exercised; all serving evidence is fixture evidence and live
qualification belongs to Kiln #49.

**Serving overrun and exact ceilings.** A served attempt's usage summary keeps
an `overrun` block beside its per-service counters. Each dispatched request
reserves its route's declared token envelope (`reserved_tokens`); only
requests with both provider counters add `known_tokens`. Each known request is
compared with its own envelope, so an observed overrun stays visible
(`overrun_requests`, `overrun_tokens`) and is never offset by an under-used
request or by an unknown request's envelope. `is_exact_ceiling` is `true` only
when every dispatched request reported usage within its envelope, and is
`false` whenever any usage is unknown or any overrun was observed. When the
serving declaration sets `stop_on_overrun`, the attempt-local broker refuses
every later served request as `overrun-stopped` before any serving-port call.
Its existing request budget refusal is recorded as `budget-exhausted`. Both
count in `refused_requests`. These are execution controls over provider
reports. They do not reach this orchestration ledger and do not bound
in-flight overshoot, so no envelope is a billing ceiling. See
[worker serving binding](worker-serving-binding.md#overrun-policy).

### Producer source boundary

- Aspen `aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4` compute-capacity records are
  consumed as a specification through `lattice.compute-allocation-observation.v1`
  (`bind-worker-compute-allocations`).
- Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b` supplies the pure
  `animus_adapters::serving` core (Animus #17), vendored through
  `scripts/sync_animus.rs` and recorded in `vendor-cargo-deps/animus/source.ncl`
  (`bind-model-serving-admission`). The `animus-std` serving host path is not
  vendored.
- Earlier inspected sources (Aspen `1e9c36edc437fdf4118120776d94f71570190511`,
  Animus `1785984ec15bf8e799da13c895abe80e58788b2d`) carried only placement and
  Mesh catalog/dispatch contracts and are not consumed for this purpose.

`RuntimeOptions::configured_profiles()` and `configured_profile(Target)` project
only existing Pi/Clankers/Animus routes. Missing, ambiguous and invalid routes
reject through existing planning. Inventory is bounded to known targets and
contains no command, argv, prompt, endpoint, credential or physical path. Output
modes are declared adapter support, not a qualified executable or live capacity.
Direct and Animus leaf routes expose their text/JSON-only and no-image/no-session
limits. Generic command session and image protocol support remain unknown, rather
than assuming arbitrary programs implement them. Compute reservations, named
model-service binding and qualified worker telemetry are explicitly unsupported.
The projection is not an owned-instance inventory: it creates no worker, idle or
healthy observation, lease, GPU allocation or request grant.

For orchestration Agent nodes, producer-backed admission and usage attribution
still require exact profile/consumer/attempt/fence/generation bindings,
freshness and expiry rechecks, definite-denial versus unknown-after-effect
outcomes, deduplication semantics, producer-owned per-service/request/deployment
observations and units/pricing identity. Implementation attempts now have these
for compute allocations and served model routes; Agent nodes do not, and they
keep reservation-only accounting. Shared physical allocations stay
provider-owned and charged once; no-model and hosted-API routes stay explicit
without synthesized inference or device usage. Pricing identity has no producer
contract. Live endpoint and cross-host qualification remain separately
authorized work under Kiln #49.

Until an accounted shell-worker profile is admitted, any Shell node is rejected
before dispatch when invocation, token or monetary accounting is enforced. This
applies to **all** shell commands, including wrappers, without command-name
heuristics. Reservation-only workflows without those enforced dimensions retain
their existing shell behavior; duration-only shell workflows retain host run
duration accounting. An Agent-node reservation does not prove complete telemetry.

The following remain unavailable rather than fabricated: exact command-worker
process CPU/memory measurements; model-request and subagent counts; exact served
weight bytes; per-service multi-model coverage; deployment device allocation or
utilization; shared-deployment attribution; and qualified cross-worker reconnect
deduplication. Hosted APIs are not assigned fictitious local GPU leases, and
no-model workers are not assigned fictitious inference usage. A model label is
not a weight digest, a profile is not a running worker, and missing service data
is not healthy/idle/zero. Shared physical resources belong to their provider;
any future apportionment must be explicit/versioned rather than charging a
deployment's resident allocation once per worker. These are open #13 addendum
qualification criteria, not coverage claims from the local accounting tests.

Focused source tests cover the production history projection through multiline
provider metadata, conservative charge and visible ledger exhaustion; early
recovery followed by durable outcome refinement, restart and once-only settlement;
crossed/conflicting bindings, partial and contradictory totals, invalid host
intervals, and frozen usage. They are not live worker/service qualification.
