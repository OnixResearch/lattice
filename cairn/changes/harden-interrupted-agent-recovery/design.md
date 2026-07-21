## Context

Lattice writes a round reservation and pending dispatch, then commits `DispatchAcknowledged`, then invokes the headless workflow. If the process terminates during a provider call, the durable instance can contain an acknowledged dispatch with no `RunRecord`. This is intentionally uncertain: the provider may have completed even though Lattice did not persist local completion.

The live drill on instance `555d173a837a54f7795dc139a64e0395b6c0932046f59fc2f841b807392bd475` established the failure shape: two repeated recovery calls returned `PendingUnobserved`, replay remained valid, cancellation was recorded, and the reservation remained held. A separate uninterrupted instance completed, settled its reservation, and replayed validly.

### Completion contract

The change is complete only when an authorized operator can terminalize a cancellation-requested, locally unobserved pending dispatch; replay reconstructs the exact terminal state; acknowledged uncertainty consumes the complete reservation; and no path retries the effect or reports success. Process exit, test count, or a cleared reservation alone is not completion evidence.

False completion includes automatic retry, timeout-based inference, cancellation treated as proof of remote cancellation, zero budget settlement for an acknowledged dispatch, a free-form potentially secret terminal reason, or a transition that can override an observed run.

## Decisions

### Decision: Add an explicit operator abandonment fact

**Choice:** Add `AbandonPendingDispatch { actor, dispatch_id, run_id }` to the pure state machine and expose it through `orchestration abandon`. The shell first requires that the matching `RunRecord` is absent; the core requires authorized actor identity, a matching pending/active dispatch, and a prior cancellation request.

**Rationale:** Abandonment states only that Lattice will no longer wait for local completion. It does not state whether the provider executed. Requiring cancellation first makes intent two-phase and reviewable. Event sourcing keeps the decision replayable and optimistic concurrency keeps duplicate facts idempotent.

### Decision: Fail terminally rather than fabricate cancellation or success

**Choice:** Abandonment clears `active_run_id` and `pending_dispatch`, sets `status = failed`, clears the cancellation request, and records a fixed safe summary that the local outcome was unobserved and no remote outcome is claimed.

**Rationale:** `cancelled` could be misread as observed remote cancellation; `completed` would fabricate success. `failed` is the only existing terminal state that accurately bounds the local claim.

### Decision: Settle according to the durable acknowledgment boundary

**Choice:** If the dispatch is acknowledged, settle the complete reserved upper bound into consumed budget. If it is still planned, settle zero and release the reservation because Lattice commits acknowledgment before invoking the workflow shell.

**Rationale:** An acknowledged remote effect may have consumed the full reservation, so a lesser charge can undercount. A planned dispatch has not crossed the shell invocation boundary under the current ordering invariant. Neither settlement is provider billing truth.

### Decision: Reject automatic mechanisms

**Choice:** Do not add automatic retry, elapsed-time inference, lease expiry, or provider-specific idempotency in this slice.

**Rationale:** Automatic retry can duplicate an effect, timeout cannot establish a remote outcome, and provider idempotency is not available through the generic contract. Those are materially different future mechanisms rather than prerequisites for safe local closure.

## Functional Core / Imperative Shell

The core validates authority, cancellation, identity, acknowledgment-sensitive settlement, and terminal state without I/O. The shell loads the instance, checks for a matching persisted run record, and atomically commits the core decision through the existing optimistic event-store boundary. CLI parsing and rendering remain in `main.rs`.

## Risks / Trade-offs

- A remote provider may finish after the Lattice process dies; abandonment intentionally records no remote outcome and does not prevent that external completion.
- An acknowledged abandonment consumes the full configured upper bound, which can overcount actual use but cannot undercount the declared reservation.
- Operators must know the original executor is stopped before abandoning; this slice does not provide distributed process liveness or remote cancellation proof.
- Headless run-detail inspection and no-clobber `--init` behavior were separate dogfood friction points and remain follow-up work rather than being coupled to recovery semantics.
