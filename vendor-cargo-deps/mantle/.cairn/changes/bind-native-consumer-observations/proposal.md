# Bind native consumer observations

## Why

The published verifier treats directory source closures as files even though the component producer emits tree identities. The native asynchronous gateway exports terminal completion signatures but does not export a standalone typed transport contract or signed nonterminal observation envelopes. Independent consumers must not fabricate these APIs or promote unsigned observations into signed evidence.

## What Changes

- Remeasure source closures using the exact existing producer tree encoding, sorted breadth-first traversal, directory records and named bounds. Preserve legacy file closures and bundle identities.
- Report typed file/tree observations and a bounded source-relative file inventory, never absolute consumer paths or payload bytes.
- Extract the existing native gateway wire DTOs and canonical request binding into the standalone build contract. Keep existing operation authorization, durable journal, host execution, quota and cancellation paths.
- Sign complete native response snapshots with the existing completion key, binding the verified holder, project, audience, canonical operation digest and observation time. Consumers verify signatures before reducing observations; terminal completions retain their independent signatures.

## Impact

- Files: `mantle-wasm-consumer-verifier`, `mantle-build-contract`, `crunch-nix-gateway`, and `src/nix_gateway/api_host.rs`.
- Compatibility: existing bundle and completion identities remain unchanged; existing native requests remain accepted. Signed response envelopes use a separately versioned schema.
- Testing: tree/file parity, no-follow capability confinement, exact source inventory, malformed/stale signatures, request/owner/fence substitution, async recovery, result verification, cancellation and producer/consumer integrated smoke. Product verification is performed by the integration owner after concurrent source work completes.
- Publication: this correction is local and unpublished until separately reviewed and published. Lifecycle admission is not acceptance, release authority, or production readiness.
