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

r[lattice.application_checkpoint.carry_forward] Restore MUST carry consumed budget, deadline and cancellation forward unchanged, MUST take the maximum budget and any cancellation across sibling manifests of the same execution, and MUST treat nodes that siblings show as started as uncertain. An independent check MUST refuse an adoption that drops any of them.

#### Scenario: Older checkpoint
r[lattice.application_checkpoint.carry_forward.scenario.older]
- GIVEN a root with a newer committed manifest than the one restored
- WHEN the older manifest is restored
- THEN budget MUST be at least the newer one, nodes the newer one started MUST be `Unknown`, and nothing MUST be replayed.

### Requirement: Restore into a fresh store

r[lattice.application_checkpoint.restore] Restore MUST adopt into a store holding no trace of the execution, keep the producer's acceptance and execution identity, reproduce every adapter and receipt identity on the target route, restore completed nodes through `Session::new_with_checkpoint` without re-running them, and continue only when no node is uncertain.

#### Scenario: Restart after source loss
r[lattice.application_checkpoint.restore.scenario.source_loss]
- GIVEN a drained source process killed with SIGKILL and its state root discarded
- WHEN the transferred checkpoint is restored on a second state root
- THEN completed nodes MUST NOT run again, the remaining nodes MUST run once, and the execution identity MUST be unchanged.

### Requirement: Restart receipt

r[lattice.application_checkpoint.receipt] Restore MUST emit `lattice.application-restart-receipt.v1` naming the checkpoint, the opaque admission reference and the restarted incarnation, and restart classification MUST distinguish `application_restart`, `vm_continuation` and `fresh_retry`. An identical repeated restart MUST return the same receipt without dispatching.

#### Scenario: Repeated restart
r[lattice.application_checkpoint.receipt.scenario.idempotent]
- GIVEN a target that already adopted a checkpoint
- WHEN the identical restart request is repeated, or another admission names the same incarnation
- THEN the identical request MUST return the same receipt and the other MUST be refused with `stale_admission`.
