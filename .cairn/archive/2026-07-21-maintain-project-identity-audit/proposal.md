# Change: Maintain the project identity audit

## Why

Two accepted project-identity scenarios remain missing because their only marker lives in an archived one-off checker that Tracey intentionally excludes. That checker also broadly exempts complete files and marks historical evidence without actually checking accepted-ID or archive immutability.

## What Changes

- Promote project-identity validation into maintained repository tests with a pure classification and manifest core plus a thin filesystem shell.
- Scan current Cargo, Nix, source, test, example, script, and operator-document surfaces for stale public `n8nclone` branding while allowing only exact compatibility, stable-protocol, and durable marker identities.
- Pin BLAKE3 manifests for accepted `n8nclone.*` requirement identities and the original rename archive's paths and bytes.
- Add positive live-repository tests and negative stale-branding, allowlist-bypass, accepted-ID mutation, and archive-mutation tests.
- Include the bounded public audit inputs and only the immutable rename archive in the isolated Nix test source closure.
- Reduce the exact missing set from 369 to 367 while preserving zero dangling markers and fail-closed `valid: false` coverage.

## Capabilities

### Modified Capabilities

- `project-identity`: Make the canonical-branding and historical-evidence audits maintained, self-checking repository behavior rather than archived one-off evidence.

## Impact

- **Files**: `tests/test_identity.rs`, focused identity-audit test modules, `flake.nix`, project-identity accepted specification, and Cairn lifecycle evidence.
- **Runtime behavior**: Unchanged.
- **Validation**: Repository tests now fail on stale public branding, changed durable accepted IDs, or changed original rename-archive bytes.
- **Non-claims**: This audit does not prove all historical semantics, arbitrary repository history, remote publication state, full traceability coverage, or release readiness.
