# Design

## Context

Canonical Mantle `6eb1c8201293fd010d186516c5dee879190f81cf` has a real asynchronous gateway and file-only consumer verifier. The producer source-tree encoding already exists in `crunch-wasm-component/src/files.rs`; consumers must use this encoding rather than hashing directories as files. The gateway already verifies exact Basalt holder invocations and signs terminal completions.

## Goals / Non-Goals

Provide typed exact offline bundle verification and authenticated nonterminal gateway observations without granting execution or release authority. Do not introduce another scheduler, permissive authority path, synthetic results, mutable source pins or a fallback runtime. Publishing and lifecycle acceptance are outside this change.

## Decisions

1. Keep structural bundle validation in the existing pure core. Extend only std remeasurement with optional capability dependencies. Open every path component relative to a retained directory descriptor with no-follow semantics. Preserve the existing file-closure case; distinguish its identity algorithm from directory source closures.
2. Tree identity matches the existing sorted breadth-first producer encoding, including directory entries. Enforce the same 10,000-entry, depth-64 and 512-MiB bounds. Include regular-file source-relative path/digest/length observations so a consumer can enforce exact manifest membership, including empty files. Never expose absolute consumer paths or payloads.
3. Extract native gateway transport DTOs and request-binding canonicalization to `mantle-build-contract::async_api`. The native host and downstream client consume the same DTOs. Existing API scheduling, journal recovery, replay, quota, cancellation and verified result paths remain unchanged.
4. Add versioned signed response snapshots through the existing native private socket receiver. Sign the full typed native response, verified owner/project/audience, canonical operation digest, producer key and current timestamp with the existing completion signing key. Keep terminal completion signatures independent. A signed event page attests exactly its native journal records; event identities remain stable across fresh snapshots.
5. Exact holder resources and operation profiles remain owned by Basalt's native Mantle SDK; consumers must not reconstruct a different profile vocabulary. Missing, stale, malformed, unsupported or invalid signatures fail closed. Cursor MACs remain producer-owned opaque state.

## Risks / Trade-offs

Source-relative inventory reveals public bundle member names but no host-absolute location or payload. Signing whole snapshots adds bounded serialization work and intentionally changes signed transport schema; unsigned responses cannot be accepted by the signed consumer path. Existing terminal completion identity and bundle identity are unchanged.

## Validation

The integration owner runs producer/consumer checks once after concurrent implementation. Required cases cover real complete tree bundles, legacy file bundles, tree drift/extra empty files/renames/empty directories/symlink ancestors, signed async submission/recovery/event paging/results/cancellation, exact holder/profile/resource binding and stale/malformed failures. Native proposal/design/tasks admission does not establish runtime behavior or production readiness.
