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
Through the command that is `lattice workflow-exchange serve --profile <profile>
--checkpoint-binding <binding.json>`. The binding names the route by `graph_identity` and
`workflow_revision`, and names the route's VM execution profile (C1) by its identity only
(`vm_profile`). Lattice treats that identity, and the optional attempt binding (C2), as opaque.

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

`runtime.lattice_revision` is `lattice-<version>-runtime-<runtime digest>-checkpoint-<checkpoint
digest>`. The runtime digest (`LATTICE_RUNTIME_IMPLEMENTATION_BLAKE3`) covers the construction
runtime: authoring construction, `src/engine/executor.rs`, effect execution and the workflow
format. The checkpoint digest (`LATTICE_CHECKPOINT_IMPLEMENTATION_BLAKE3`) covers, as
length-framed path and bytes in sorted path order, `Cargo.lock`, every file under
`src/checkpoint/` and `src/workflow_exchange/`, `src/engine/executor/choice.rs`
(`run_authorized_from_checkpoint`), `src/headless/shell/checkpoint.rs`,
`src/headless/shell/record.rs` and `src/workflow/gate/shell/checkpoint.rs`. Separately built
binaries of the same source on different architectures report the same revision; builds whose
publication, admission or restore code differs do not admit each other's manifests.

`consumed` carries only `elapsed_ms`. Model requests and cost are not measured at node
boundaries, so the manifest does not record them; across a restart they are enforced by the
host's per-binding budget ledger (C3), not by the checkpoint.

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
lattice workflow-exchange serve --profile <handler profile> --checkpoint-binding <target binding.json>
lattice application-checkpoint inspect --root <checkpoint root>
```

`restore` only adopts: it admits the checkpoint, writes the adoption and prints the receipt and
the adopted state (`queued`, or `unknown` with uncertain nodes). It runs nothing. The target's
`workflow-exchange serve` on the same data directory then starts every adopted execution that
never started, publishing that incarnation's checkpoints under its binding, and serves Observe,
Cancel and log reads for it through the same listener and admission as a fresh dispatch. The
start is a compare-and-swap from `queued` to `running` in the single-writer store, so exactly one
process runs an adopted execution; a second process cannot open the store. `restore` must run
before the target's exchange starts. `serve` binds its listener before it starts any adopted
execution, so a bind failure (such as the stale socket a killed `serve` leaves behind) starts
none.

The serving binding publishes the restarted run's checkpoints, so it must declare a checkpoint
and reproduce the deadline and attempt binding that `restore` admitted. An adoption served under
a binding with `checkpoint: "none"`, another deadline or another attempt binding is recorded
`unknown` and runs nothing.

Adopted executions count against the profile's `maximum_active_executions` like fresh ones.
`serve` starts at most that many, lowest run id first; the rest stay `queued`, observe as
`queued` (never `unknown`), and start on the first exchange request after a slot frees, before
any fresh dispatch is considered. A waiting adoption accepts Cancel: the cancel is recorded, it
stays `queued`, and once a slot frees it starts cancelled and ends `cancelled` with no new
effect. An adopted execution that was `running` when its exchange process died observes as
`unknown` and never runs again.

The restart request is `lattice.application-restart-request.v1`:
`{ schema, checkpoint, admission, restarted_incarnation, request, execution, binding?,
cancellation_requested }`. `admission` is Kiln's opaque `KilnAdmissionRef`; Lattice never mints
it. `cancellation_requested` is required: Kiln owns the cancel decision and is the only
restorer, so a Cancel it acknowledged after the last publication (for example after a drain, or
while the source sat in a long effect-free node) reaches the restarted run here. Admission ORs it
with the manifest's and its siblings' flags; when the result is set, the adopted record starts
cancelled and the restarted run dispatches no new effect. The manifest flag still records
cancels observed before publication.

Admission (pure, `lattice::checkpoint::admit`) checks, in order: request schema; admission
reference shape; commit marker present and naming the manifest; manifest identity and canonical
bytes; manifest shape; runtime revision and environment (a portable profile then admits any
separately built binary of that revision on any architecture; a non-portable profile would also
need the same architecture and identical binary); crossed request, execution and binding; the `WorkflowRequest`
object; stale admission (`restarted_incarnation` must exceed the producing incarnation and every
sibling's); the advisory deadline; every node object and workspace object present and verified;
sensitive outputs; sibling carry-forward; an existing adoption in the target store (identical,
including the cancel decision → idempotent receipt, newer or equal incarnation →
`stale_admission`, otherwise `target_not_fresh`).

Siblings are the other committed manifests of the same execution in the root. They can only add:
any node they show as completed or uncertain that the restored manifest does not complete becomes
uncertain; `consumed.elapsed_ms` takes the maximum; `cancellation_requested` is OR-ed; a
different deadline or binding is refused. Restoring an older checkpoint therefore never resets
the elapsed budget, deadline or cancellation, and never replays what a later manifest shows
started.

The target then checks that its binding reproduces the checkpoint's deadline (`carry_forward`
otherwise) and attempt binding (`crossed_binding` otherwise), resolves its own route, recomputes
every adapter and receipt identity (`crossed_adapter`, `crossed_receipt`,
`crossed_handler_profile`, `crossed_workflow_revision`), pre-flights
`Session::new_with_checkpoint`, and refuses a store that already holds the execution
(`target_not_fresh`). An independent check (`verify_carry_forward`) confirms elapsed budget,
deadline, cancellation and uncertainty survive in the record it will write. Only then are node
objects copied into the target root and workspace objects written: every object is verified
against the target workspace first, and existing different content refuses
(`workspace_path`) before anything is written. The adoption is committed last, in one store
transaction that re-runs the check; if it fails, the workspace files this restore created are
removed. A refused restore therefore adopts nothing, and a committed adoption is complete, so an
identical retry after a crash is idempotent and the served exchange runs it.

The adopted record keeps the producer's acceptance and execution identity, moves `sequence` past
the manifest, and carries `cancellation_requested`.

- With uncertain nodes, the record is `Unknown`, those nodes are `Unknown`, and nothing is
  scheduled. They stay unknown until reconciled through the existing paths.
- Otherwise the serving exchange continues from the checkpoint with the executor's checkpoint
  path (`Session::new_with_checkpoint`); completed nodes are not re-run, and the new incarnation
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
