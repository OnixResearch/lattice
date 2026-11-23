## ADDED Requirements

### Requirement: Checkpoint hooks only for declared profiles

r[lattice.application_checkpoint.hook] The workflow exchange worker MUST publish application checkpoints only for a route bound to an opaque VM execution profile identity whose declared checkpoint is `workflow-node-boundary-v1`. Without such a binding the hook MUST be absent and execution MUST be unchanged.

#### Scenario: Route without a declared checkpoint
r[lattice.application_checkpoint.hook.scenario.absent]
- GIVEN a route with no checkpoint binding or a binding declaring `none`
- WHEN an execution of that route runs to completion
- THEN no checkpoint root MUST be written and the node effects MUST run exactly as without hooks.

### Requirement: Unsupported workloads stay explicit

r[lattice.application_checkpoint.unavailable] A handler whose route declares no checkpoint MUST report its checkpoint offer as `Unavailable`, MUST NOT build a manifest, and MUST refuse restore with `checkpoint_unavailable`.

#### Scenario: Restore without hooks
r[lattice.application_checkpoint.unavailable.scenario.restore]
- GIVEN a target handler whose binding declares `none`
- WHEN a restart request names a committed checkpoint
- THEN the restore MUST be refused with `checkpoint_unavailable` before any store write.

### Requirement: Publication at node boundaries

r[lattice.application_checkpoint.publish] The hook MUST publish a `lattice.application-checkpoint-manifest.v1` at every completed-node boundary containing completed, failed and skipped node outcomes as content-addressed objects with their producer receipt identities. Node outputs carrying sensitive fragments MUST NOT be written and the node MUST be listed as uncertain.

#### Scenario: Sensitive output
r[lattice.application_checkpoint.publish.scenario.sensitive]
- GIVEN a finished node whose outputs carry sensitive fragments
- WHEN the boundary is published or a manifest naming such an object is admitted
- THEN its outputs MUST NOT be persisted and admission MUST refuse with `sensitive_output`.

### Requirement: Effect fence

r[lattice.application_checkpoint.fence] Before any effect attempt is reserved, the hook MUST synchronously commit a manifest listing that node as uncertain. If that publication fails, the reservation MUST be refused. A host drain request MUST make the next reservation wait for in-flight effects, commit a quiescent manifest, acknowledge it, and park.

#### Scenario: Crash during an effect
r[lattice.application_checkpoint.fence.scenario.mid_effect]
- GIVEN a source process killed after a node started its effect
- WHEN the latest committed checkpoint is restored on another state root
- THEN the node MUST be `Unknown`, the execution MUST be `Unknown`, and the effect MUST NOT run again.

### Requirement: Commit protocol

r[lattice.application_checkpoint.commit] Publication MUST write and verify every object, then the manifest, then the commit marker `{ manifest }` last, each fsynced. A manifest without a marker MUST NOT be admissible.

#### Scenario: Crash between objects and marker
r[lattice.application_checkpoint.commit.scenario.partial]
- GIVEN objects and a manifest written without their commit marker
- WHEN a restart names that manifest
- THEN the restore MUST be refused with `not_committed`.

### Requirement: Manifest admission

r[lattice.application_checkpoint.admission] Restore MUST admit a manifest only when its identity and canonical bytes match the marker and the request, its schema is supported, its runtime revision and environment match, every object is present and verified, the request, execution and binding are not crossed, the restarted incarnation is strictly newer than the producing incarnation and every sibling, and the advisory deadline has not passed.

#### Scenario: Crossed or stale admission
r[lattice.application_checkpoint.admission.scenario.refused]
- GIVEN a committed checkpoint and a restart request with a crossed execution, crossed binding, wrong schema, missing object or stale incarnation
- WHEN restore runs
- THEN it MUST be refused with the matching code and nothing MUST be adopted.

### Requirement: Portable restart

r[lattice.application_checkpoint.portable] The `workflow-node-boundary-v1` profile MUST be admitted by any separately built binary of the same runtime revision on any architecture; a non-portable profile MUST require the same architecture and identical binary.

#### Scenario: Other architecture
r[lattice.application_checkpoint.portable.scenario.cross_isa]
- GIVEN a portable manifest produced on x86_64
- WHEN a separately built aarch64 binary of the same revision restores it
- THEN admission MUST succeed.

### Requirement: Workspace objects

r[lattice.application_checkpoint.workspace] Workspace objects MUST use relative normalized paths without parent or current-directory parts and MUST be verified by size and BLAKE3 before materialization.

#### Scenario: Unsafe path
r[lattice.application_checkpoint.workspace.scenario.unsafe]
- GIVEN a manifest naming `../escape` as a workspace path
- WHEN it is admitted
- THEN it MUST be refused with `workspace_path`.

### Requirement: Carry-forward

r[lattice.application_checkpoint.carry_forward] Restore MUST carry the measured elapsed budget, deadline and cancellation forward unchanged, MUST take the maximum elapsed budget and any cancellation across sibling manifests of the same execution, and MUST treat nodes that siblings show as started as uncertain. The target binding MUST reproduce the checkpoint's deadline and attempt binding. An independent check MUST refuse an adoption that drops any of them. Model requests and cost are not measured at node boundaries; the manifest MUST NOT record them, and the host's per-binding budget ledger enforces them across a restart.

#### Scenario: Older checkpoint
r[lattice.application_checkpoint.carry_forward.scenario.older]
- GIVEN a root with a newer committed manifest than the one restored
- WHEN the older manifest is restored
- THEN the elapsed budget MUST be at least the newer one, nodes the newer one started MUST be `Unknown`, and nothing MUST be replayed.

#### Scenario: Moved deadline or attempt binding
r[lattice.application_checkpoint.carry_forward.scenario.binding]
- GIVEN a target binding whose deadline or attempt binding differs from the checkpoint's
- WHEN the checkpoint is restored
- THEN restore MUST refuse with `carry_forward` or `crossed_binding` and adopt nothing.

### Requirement: Kiln's cancel decision

r[lattice.application_checkpoint.cancellation] The restart request MUST carry Kiln's cancel decision as a required `cancellation_requested`. Admission MUST OR it with the cancellation of the manifest and its siblings; when the result is set, the adopted record MUST start cancelled and the restarted run MUST dispatch no new effect. On the serving exchange, an execution, adopted or freshly dispatched, whose cancel is recorded before its worker marks it running MUST end `cancelled` without running, even when that cancel won the race against the worker's `queued` to `running` transition. A run whose ready frontier stalled on cancellation before dispatching any node MUST end `cancelled`; a run with any other node result MUST keep its observed outcome.

#### Scenario: Cancel acknowledged after a drain
r[lattice.application_checkpoint.cancellation.scenario.after_drain]
- GIVEN a drained, uncancelled manifest and a restart request with `cancellation_requested: true`
- WHEN the checkpoint is restored and the target exchange serves it
- THEN the execution MUST end `cancelled` and no further effect MUST run.

### Requirement: Restore into a fresh store

r[lattice.application_checkpoint.restore] Restore MUST adopt into a store holding no trace of the execution, keep the producer's acceptance and execution identity, reproduce every adapter and receipt identity on the target route, and restore completed nodes through `Session::new_with_checkpoint` without re-running them. Every check and the workspace write MUST precede the adoption commit, and the carry-forward check MUST run inside the adoption transaction, so a refused restore adopts nothing and a committed adoption is complete. Restore MUST NOT run the execution.

#### Scenario: Restart after source loss
r[lattice.application_checkpoint.restore.scenario.source_loss]
- GIVEN a drained source process killed with SIGKILL and its state root discarded
- WHEN the transferred checkpoint is restored on a second state root
- THEN completed nodes MUST NOT run again, the remaining nodes MUST run once, and the execution identity MUST be unchanged.

#### Scenario: Conflicting target workspace
r[lattice.application_checkpoint.restore.scenario.workspace_conflict]
- GIVEN a target workspace holding different content at a checkpointed path
- WHEN the checkpoint is restored, and again
- THEN every attempt MUST refuse with `workspace_path`, adopt nothing and leave the workspace unchanged.

### Requirement: Restarted runs through the exchange

r[lattice.application_checkpoint.resume] `lattice workflow-exchange serve` MUST accept a checkpoint binding, publish checkpoints for its route, and start adopted executions that never started, observable and cancellable through the same listener and admission as a fresh dispatch. It MUST bind that listener before starting any adopted execution, so a bind failure starts none. It MUST start an adopted execution only when its binding declares a checkpoint and reproduces the adoption's deadline and attempt binding, and otherwise MUST record it `Unknown` without running it. Adopted executions MUST count against `maximum_active_executions`: at most that many MUST run, lowest run id first, and the rest MUST stay and observe as `queued` until a slot frees. A waiting adoption MUST accept a cancel, and once a slot frees it MUST start cancelled. Exactly one process MUST run an adopted execution, it MUST be started only when no node is uncertain, and one that was running when its process died MUST become `Unknown`.

#### Scenario: Observe and cancel a restored execution
r[lattice.application_checkpoint.resume.scenario.cancel]
- GIVEN an adopted execution whose remaining node is running on the target exchange
- WHEN a client observes and cancels it over the exchange socket
- THEN the observation MUST show it running, it MUST end `cancelled`, and a second process MUST NOT serve the same state root.

#### Scenario: More adoptions than the active-execution limit
r[lattice.application_checkpoint.resume.scenario.limit]
- GIVEN two more adopted executions than the profile's `maximum_active_executions`, one of the waiting ones cancelled while `queued`
- WHEN the target exchange serves them
- THEN at most the limit MUST run and the extra ones MUST observe as `queued`; once slots free, the uncancelled one MUST start and complete, and the cancelled one MUST end `cancelled` without running any effect.

#### Scenario: A serving binding that differs from the adoption
r[lattice.application_checkpoint.resume.scenario.binding]
- GIVEN an adoption restored under the target binding
- WHEN `serve` runs with a later deadline, another attempt binding, or `checkpoint: "none"`
- THEN the adopted execution MUST observe as `unknown` and run no effect.

#### Scenario: A listener that cannot bind
r[lattice.application_checkpoint.resume.scenario.bind]
- GIVEN an adoption and a stale socket at the profile's socket path
- WHEN `serve` runs
- THEN it MUST exit with an error, the adoption MUST stay `queued` with no effect run, and a later `serve` that binds MUST run it.

### Requirement: Restart receipt

r[lattice.application_checkpoint.receipt] Restore MUST emit `lattice.application-restart-receipt.v1` naming the checkpoint, the opaque admission reference and the restarted incarnation, and restart classification MUST distinguish `application_restart`, `vm_continuation` and `fresh_retry`. An identical repeated restart MUST return the same receipt without dispatching.

#### Scenario: Repeated restart
r[lattice.application_checkpoint.receipt.scenario.idempotent]
- GIVEN a target that already adopted a checkpoint
- WHEN the identical restart request is repeated, or another admission names the same incarnation
- THEN the identical request MUST return the same receipt and the other MUST be refused with `stale_admission`.
