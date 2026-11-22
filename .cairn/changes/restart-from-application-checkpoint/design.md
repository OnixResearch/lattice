## Context

The exchange worker runs the production headless executor with route-exact authority. A
nonterminal execution without a live worker is marked `Unknown` and never re-run. Approval-gate
suspension already persists terminal node outcomes and refuses sensitive outputs
(`checkpoint_from_results`), and the executor can restore a session from terminal steps
(`Session::new_with_checkpoint`). Node adapter identities derive only from the workflow revision,
node and operation, so they are stable across restarts. C1 and C2 are owned by slice S1; this
change treats them as opaque identities.

## Decisions

### Decision: Publish before every effect, not only after nodes

**Choice:** In addition to completed-node boundaries (executor `NodeProgress::Finished`), the
hook runs synchronously as a reservation fence on the route authority's effect-attempt store and
commits a manifest naming the node as uncertain before the reservation proceeds. A failed
publication refuses the reservation.

**Rationale:** Progress observations are nonblocking by design, so an effect could be dispatched
before an after-the-fact manifest commits; a restore from the previous manifest would then replay
it blindly. With the fence, every effect that may have happened is named by a committed
manifest. This is the only way to keep "no blind retry" without changing the executor's
nonblocking observation contract.

### Decision: Siblings only add uncertainty, budget and cancellation

**Choice:** Admission unions the other committed manifests of the same execution in the root:
nodes they show as completed or uncertain, but which the restored manifest does not complete,
become uncertain; `consumed` is the component-wise maximum; cancellation is OR-ed; a different
deadline or binding is refused.

**Rationale:** Kiln may admit an older checkpoint. Restoring it must not reset budget, deadline
or cancellation (#12) and must not replay what a later manifest shows started.

### Decision: Uncertain nodes freeze the restored execution as `Unknown`

**Choice:** If any node is uncertain after admission, the adopted record is `Unknown`, those
nodes are `Unknown`, and no worker is started.

**Rationale:** This mirrors the existing `mark_unknown` semantics and `PendingUnobserved`
recovery: an uncertain outcome is reconciled, not re-run. Continuing independent branches is a
later refinement.

### Decision: Drain through a file in the checkpoint root

**Choice:** `<root>/drain-request` makes the next effect reservation wait for in-flight effects,
publish a quiescent manifest, write `drained.json`, and park until the request is removed.

**Rationale:** Without a drain, a crash always lands while some effect is in flight in a linear
workflow, and every restart would be `Unknown`. The drain is the application-level counterpart of
Kiln's `DrainDisposition::Checkpointed` and needs no wire-schema change (D6).

### Decision: Runtime match by source revision; portable profiles admit separate builds

**Choice:** `runtime.lattice_revision` is the executor/restore source digest. A portable profile
admits any binary of that revision on any architecture; a non-portable profile would also need
the same architecture and identical binary.

**Rationale:** Cross-ISA restart is by construction a separately built binary. The node-boundary
profile carries only JSON outcomes and workspace bytes.

### Decision: Route-to-C1 binding is a separate record

**Choice:** `lattice.application-checkpoint-binding.v1` names one route by graph identity and
workflow revision, the opaque C1 identity, its declared checkpoint, the checkpoint root, optional
workspace, C2 binding identity, environment and deadline. It is supplied to the handler with
`with_application_checkpoint`.

**Rationale:** The handler profile v1 must stay byte- and identity-compatible, and the plan does
not define how a route references C1. Keeping the binding separate avoids changing any existing
schema.

## Risks / Trade-offs

- The admission reference is opaque; Lattice only checks shape, incarnation ordering and target
  freshness. Currentness of placement and budget is Kiln's (production: Aspen, OD-2).
- The deadline check on restore is advisory; the host clock is authoritative (D4).
- Cross-host durability of checkpoint content is out of scope; the root is a local directory
  transferred by Kiln.
- Q1-emulated aarch64 evidence depends on building Lattice for aarch64 in this dev shell; see the
  verification status in `tasks.md`.
