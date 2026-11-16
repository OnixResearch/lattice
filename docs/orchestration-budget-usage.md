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

**Qualified measured command-worker support is not established.** The current
command adapter does not supply a qualified worker telemetry protocol; existing
direct OpenAI-compatible metadata has model labels but lacks exact service,
request, worker-incarnation, attempt/fence, allocation/placement and pricing
bindings. [Execution bindings #24](https://github.com/OnixResearch/lattice/issues/24)
must supply admitted identities and compatibility before they can be consumed
here. No self-declared profile/service field establishes qualification, and no
new producer schema is invented by this accounting boundary.

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
