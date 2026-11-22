# Application checkpoints and restart (Lattice #12)

Lattice can restart a workflow-exchange execution on another state root from a committed
application checkpoint. The first and only supported profile is `workflow-node-boundary-v1`:
completed, non-sensitive node outcomes plus workspace objects. Nothing here preserves process,
memory or socket state; that is Kiln's VM continuation, not an application restart.

Kiln consumes this work only through the `lattice` process and the JSON fixtures in
`fixtures/application-checkpoint/` (OD-1). Kiln never links this code.

## When hooks run

A route publishes checkpoints only when the handler is given a
`lattice.application-checkpoint-binding.v1` whose `checkpoint` is `workflow-node-boundary-v1`.
The binding names the route by `graph_identity` and `workflow_revision`, and names the route's VM
execution profile (C1) by its identity only (`vm_profile`). Lattice treats that identity, and
the optional attempt binding (C2), as opaque.

Without a binding, or with `checkpoint: "none"`, the hook is absent: execution is unchanged, the
handler reports `checkpoint_offer() == Unavailable`, and `restore` refuses with
`checkpoint_unavailable`.

With a binding, the worker:

- publishes a manifest at every completed-node boundary (`NodeProgress::Finished`);
- publishes a manifest **synchronously before every effect reservation**, listing that node in
  `uncertain_nodes`. If that publication fails, the reservation is refused and the effect does
  not run. Every effect that may have happened is therefore named by a committed manifest;
- records a finished node whose outputs carry sensitive fragments as uncertain. Its outputs are
  never written (the `checkpoint_from_results` rule) and it is never replayed.

## Root layout and commit order

```
<root>/requests/<hex>.json      the WorkflowRequest named by execution.request
<root>/objects/<hex>            lattice.application-checkpoint-node-outputs.v1 objects
<root>/workspace/<blake3>       workspace file contents
<root>/manifests/<hex>.json     lattice.application-checkpoint-manifest.v1 (C8)
<root>/commits/<n>-<hex>.json   commit marker { "manifest": "b3:…" }, n = publication order
<root>/drain-request            host-created; asks the hook to drain (see below)
<root>/drained.json             lattice.application-checkpoint-drain.v1 acknowledgement
```

Every file is written to a temporary name, fsynced, renamed, its directory fsynced, then read
back and compared. Objects come first, then the manifest, then the commit marker. A manifest
without a marker is not admissible (`not_committed`).

Identities use the exchange framing: BLAKE3 over
`u64be(len(domain)) ‖ domain ‖ u64be(len(json)) ‖ json`, rendered `b3:<64 hex>`, with the
schema string as domain. Manifest bytes must be canonical (`serde_json` output of the parsed
manifest), so the file bytes hash to the identity. `workspace_objects[].blake3` and
`runtime.lattice_binary_blake3` are plain BLAKE3 content digests (64 lowercase hex).

`runtime.lattice_revision` is `lattice-<version>-runtime-<digest>`, where the digest covers the
executor and restore sources. Separately built binaries of the same source on different
architectures report the same revision.

## Drain

When `<root>/drain-request` exists, the next effect reservation waits until no other effect is
in flight, publishes a quiescent manifest, writes `drained.json` naming it, and parks until the
request is removed. Kiln can then stop the source and restart from a manifest whose
`uncertain_nodes` is empty. This is the application-level counterpart of Kiln's
`DrainDisposition::Checkpointed`.

## Restore

```
lattice application-checkpoint restore \
  --profile <handler profile> --binding <target binding.json> \
  --source <checkpoint root> --request <restart request.json>
lattice application-checkpoint inspect --root <checkpoint root>
```

The restart request is `lattice.application-restart-request.v1`:
`{ schema, checkpoint, admission, restarted_incarnation, request, execution, binding? }`.
`admission` is Kiln's opaque `KilnAdmissionRef`; Lattice never mints it.

Admission (pure, `lattice::checkpoint::admit`) checks, in order: request schema; admission
reference shape; commit marker present and naming the manifest; manifest identity and canonical
bytes; manifest shape; runtime revision and environment (a portable profile then admits any
separately built binary of that revision on any architecture; a non-portable profile would also
need the same architecture and identical binary); crossed request, execution and binding; the `WorkflowRequest`
object; stale admission (`restarted_incarnation` must exceed the producing incarnation and every
sibling's); the advisory deadline; every node object and workspace object present and verified;
sensitive outputs; sibling carry-forward; an existing adoption in the target store (identical →
idempotent receipt, newer or equal incarnation → `stale_admission`, otherwise
`target_not_fresh`).

Siblings are the other committed manifests of the same execution in the root. They can only add:
any node they show as completed or uncertain that the restored manifest does not complete becomes
uncertain; `consumed` takes the component-wise maximum; `cancellation_requested` is OR-ed; a
different deadline or binding is refused. Restoring an older checkpoint therefore never resets
budget, deadline or cancellation, and never replays what a later manifest shows started.

The target then resolves its own route, recomputes every adapter and receipt identity
(`crossed_adapter`, `crossed_receipt`, `crossed_handler_profile`, `crossed_workflow_revision`),
pre-flights `Session::new_with_checkpoint`, copies node objects into its own root, and adopts
atomically into a store that has no trace of the execution (`target_not_fresh` otherwise). The
adopted record keeps the producer's acceptance and execution identity, moves `sequence` past the
manifest, and carries `cancellation_requested`. An independent check (`verify_carry_forward`)
then confirms budget, deadline, cancellation and uncertainty survived.

- With uncertain nodes, the record is `Unknown`, those nodes are `Unknown`, and nothing is
  scheduled. They stay unknown until reconciled through the existing paths.
- Otherwise a worker continues from the checkpoint with the executor's checkpoint path
  (`Session::new_with_checkpoint`); completed nodes are not re-run, and the new incarnation
  publishes its own checkpoints.

## Receipt

`lattice.application-restart-receipt.v1` is `{ checkpoint, kind, restarted_incarnation, admission }`.
`restart_kind` distinguishes:

| kind | meaning |
|---|---|
| `application_restart` | a checkpoint was adopted by a strictly newer incarnation |
| `vm_continuation` | nothing adopted; the producing incarnation still runs (VM pause/restore or move) |
| `fresh_retry` | nothing adopted; a newer incarnation starts from scratch |

`restore` only ever issues `application_restart`.

## Refusal codes

`schema`, `encoding`, `manifest_identity`, `not_committed`, `checkpoint_unavailable`, `runtime`,
`architecture`, `environment`, `crossed_request`, `crossed_execution`, `crossed_binding`,
`crossed_route`, `crossed_handler_profile`, `crossed_workflow_revision`, `crossed_adapter`,
`crossed_receipt`, `duplicate_node`, `node_limit`, `object_missing`, `object_mismatch`,
`workspace_path`, `stale_admission`, `admission_reference`, `deadline_expired`, `carry_forward`,
`target_not_fresh`, `checkpoint_invalid`, `sensitive_output`, `io`, `store`.

## Non-claims

- An application checkpoint does not preserve process, memory or socket state.
- A restart receipt does not prove the source host stopped executing; Kiln's fencing does.
- An uncertain node outcome is not evidence that its effect did or did not happen.
- Checkpoint admission does not establish current placement or budget authority. The deadline
  check is advisory; Kiln's host clock and C3 budget ledger are authoritative.
- Single-host evidence (two state roots on one host) is not cross-host evidence; the durable
  cross-host content store and placement authority belong to Kiln #53/#54 and Aspen.
