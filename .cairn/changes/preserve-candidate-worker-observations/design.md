## Context

The root manifest selects vendor-cargo-deps/bounded-exec/crates/bounded-exec at SOURCE revision 29dac88ecded94457572db3fdfaaaab95fa91525. Its RunError is not a no-start enum. Wait errors are returned only after terminate_and_reap succeeds; stream-worker errors are surfaced after wait/owned-process cleanup; worker teardown deadlines still have observed child termination. Kill/reap/deadline/PID-conversion failures do not establish child termination.

OMP reference e325ffc73ab86e03bdd20a91b77d8e2b3964c906 documents headless --mode json as session-header plus JSONL events. Its print-mode.ts emits message_end and agent_end, omits partial snapshots, and uses terminal assistant error/aborted reasons for failure. A final message is never a candidate receipt.

## Decisions

Use typed safe phase and termination labels, not raw OS diagnostics. Preserve existing generic command APIs while adding an observed-output route for candidate workers. Interrupted workers retain best-effort file evidence and durable Unknown; reconciliation never promotes Unknown to Published or redispatches. Unknown is not a claim that the child is still running.

Select OMP through a separate explicit wrapper with reviewed executable byte pin, exact reference revision, model, base and fixed argv. Keep raw bounded stdout accessible independently of normalization and report usage completeness false. Require the explicit bubblewrap provider; do not silently fall back. Capture only ordinary file bytes and explicitly reject unsupported file types and mode changes. Provider identity includes its executable and effective mount/argument policy.

## Qualification boundary

The provider policy uses documented bubblewrap namespace, tmpfs and remount controls, but no live deployment-kernel qualification was executed. network=true remains shared networking, not endpoint-specific authority. Reviewed immutable OMP source/build provenance, actual provider execution, authorized model endpoint and selected-route terminal-failure/cancellation fixtures remain required for issue #7 completion. The implementation-attempt shell now retains separately fsynced raw-event artifacts before candidate publication and checks their exact digest and normalization on recovery. No synthetic executable is presented as OMP.
