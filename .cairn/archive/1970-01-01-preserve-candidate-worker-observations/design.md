## Context

The root manifest selects vendor-cargo-deps/bounded-exec/crates/bounded-exec at SOURCE revision 29dac88ecded94457572db3fdfaaaab95fa91525. Its RunError is not a no-start enum. Wait errors are returned only after terminate_and_reap succeeds; stream-worker errors are surfaced after wait/owned-process cleanup; worker teardown deadlines still have observed child termination. Kill/reap/deadline/PID-conversion failures do not establish child termination.

OMP reference e325ffc73ab86e03bdd20a91b77d8e2b3964c906 documents headless --mode json as session-header plus JSONL events. Its print-mode.ts emits message_end and agent_end, omits partial snapshots, and uses terminal assistant error/aborted reasons for failure. A final message is never a candidate receipt.

## Decisions

Use typed safe phase and termination labels, not raw OS diagnostics. Preserve existing generic command APIs while adding an observed-output route for candidate workers. Interrupted workers retain best-effort file evidence and durable Unknown; reconciliation never promotes Unknown to Published or redispatches. Unknown is not a claim that the child is still running.

Select OMP through a separate explicit wrapper with reviewed executable byte pin, exact reference revision, model, base and fixed argv. Keep raw bounded stdout accessible independently of normalization and report usage completeness false. Require the explicit bubblewrap provider; do not silently fall back. Capture only ordinary file bytes and explicitly reject unsupported file types and mode changes. Provider identity includes its executable and effective mount/argument policy.

## Qualification boundary

The generic provider retains explicit shared networking for trusted commands, but OMP and implementation-attempt version 2 refuse it. The selected route pins canonical immutable Nix store executable paths before dispatch, preventing mutable symlink substitution between hashing and execution. A private Unix socket carries only bounded exact-model chat requests to the host; the host resolves the selected credential and uses the existing bounded native HTTP client without redirects/retries/proxies. The broker mount is bound to the complete attempt, argv and enforcement request. Four actual deployment-kernel scenarios passed: workspace and host-path confinement, unauthorized writes, cancellation, and the isolated host-credential broker route. The reviewed OMP build also produced a real host-verified candidate using host-only OMP authentication and gpt-6.1-sol; selected-route local HTTP-400 failure and post-request cancellation fixtures passed. This qualifies only the bounded single-shot profile, not general hostile-sandbox safety or durable continuation. The pinned producer's restore/retry path can re-execute outcome-unknown tools and lacks a host-accepted result/cursor/no-effect restore contract. Kiln #46 still owns independently executed acceptance; supplied-observation admission is not a verifier.
