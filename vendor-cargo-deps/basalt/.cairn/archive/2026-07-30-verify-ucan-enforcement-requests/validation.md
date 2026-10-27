# Validation Status

## Result

**ARCHIVED AND VALIDATED.** The reviewed vendored UCAN boundary and enforcement adapter pass the required implementation, release, and lifecycle rails.
The accepted specification is synced, and the change is archived with date `2026-07-30`.

## Baseline Evidence

- The focused authority-core baseline passed with 8 tests.
- Canonical-policy Cairn validation passed.
- The proposal gate passed.
- The design gate passed.
- The tasks gate passed.

## Reviewed UCAN Boundary

Basalt records UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf`.
The source worktree was clean at that revision.
The UCAN compact-token and holder-signed invocation changes are archived with date `2026-07-30`.

Basalt does not claim that this unpushed revision is available from a remote source.
The durable boundary is the checked-in `vendor/ucan/` snapshot.

`tools/sync-vendored-ucan.rs` imports only the pinned commit.
Its offline check covers the revision, file set, file modes, byte lengths, BLAKE3 identities, and required license files.
Its source-parity mode compares the snapshot with the exact Git commit.
The Nix check runs the checker tests and offline snapshot check.

## Closed Dependency Criteria

The reviewed UCAN snapshot provides these facts:

- borrowed compact-token verification with bounded secret ownership
- explicit key, proof, revocation, caveat, time, and limit inputs
- holder-signed exact request verification
- fixed operation-profile and request-binding types
- one final holder replay commit
- private, non-serde, non-clone live holder authorization

## Composition Invariant

Basalt policy work must finish before holder replay.
A Basalt policy denial must make zero replay calls.

The adapter builds the prospective authority decision and redacted receipt before final holder verification.
Fresh holder verification remains authoritative and performs the one replay commit.
After replay succeeds, Basalt can only move prebuilt values into the live capability.

## Validation Evidence

- The focused adapter suite passed 19 positive, negative, ordering, receipt, and source-guard tests.
- Workspace tests passed with and without the CLI feature. Doctests also passed.
- Strict Clippy passed for all targets and features with warnings denied.
- The vendored checker passed 3 unit tests, the 669-file offline check, and source parity against revision `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf`.
- Each enforcement, integration-test, vendored-sync, and release-verifier module is below 300 lines.
- The bounded Octet run completed with warning-only legacy findings, zero errors, and no enforcement file-length or function-length finding.
- The direct private release verifier passed all steps, including the validated package list and vendored source inventory.
- `nix flake check path:$PWD -L --option secret-key-files ''` passed all checks.
- Canonical-policy Cairn validation and the proposal, design, and tasks gates passed.

Basalt has `publish = false`.
The release rail does not create a Cargo tarball because Cargo would rewrite the reviewed path dependency to the unrelated crates.io package `ucan ^0.1.0`.
The private source inventory, package list, revision, licenses, and BLAKE3 snapshot are the supported release boundary.

## Sync Evidence

- Canonical dry-run and executed sync passed.
- The sync receipt hash is `8dd3f56cc4c84758ec7c65818cf4414a9170e64096ebc44f94f6e9a3a226ee43`.

## Archive Evidence

- Dry-run and executed archive passed with `CAIRN_ARCHIVE_DATE=2026-07-30`.
- The archive receipt hash is `d049d10a84d53cb5fd3554b8c75a7ef0b9cc3d0ae733ffa8a41fac35f3bc41c0`.
- Post-archive canonical validation passed with no substance findings or issues.
