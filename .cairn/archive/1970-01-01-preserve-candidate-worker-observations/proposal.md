## Why

Lattice issue #7 requires truthful post-spawn runner failures and an explicit pinned OMP JSONL coding profile. The selected Bounded Exec returns errors after child start, including after successful child cleanup; classifying every error as Spawn erases the evidence needed for safe attempt reconciliation.

## What Changes

- Preserve safe execution/capture/teardown phases and observed-versus-unknown child termination from the actual vendored return paths.
- Keep bounded output on observed timeout/cancellation/overflow; retain best-effort candidate files on post-start infrastructure failure and durably classify that result Unknown.
- Normalize the referenced OMP JSONL profile separately from generic single-document agents, with explicit executable bytes, argv and base admission.
- Capture previously omitted ordinary source files, reject unsupported file types/modes and strengthen the explicit private workspace mount policy.
- Enforce an exact bounded model route through a private Unix-socket host broker, pin immutable sandbox/relay/shell/worker artifacts, and remove shared-network implementation requests through an explicit version-2 cutover.

## Impact

Owned files are candidate worker/capture/receipt, process shell and agent process error types, associated fixtures, and docs/candidate-worker.md. Host integration of new process error variants is coordinated separately. No vendor source, claims/attempt controller, orchestration or store implementation is changed here.

The reviewed-source OMP build and bounded single-shot normal/failure/cancellation route have executed, alongside the endpoint broker and deployment-kernel fixtures. This does not establish durable OMP continuation, external current-grant verification, independent acceptance, publication or merge authority. The pinned producer's restore/retry semantics cannot substitute for no-effect restoration of host-accepted calls. Raw-event publication remains integrated through the implementation-attempt shell, with bounded digest/normalization checks on recovery.
