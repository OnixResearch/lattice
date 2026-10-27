# Final UCAN 1.0 adoption evidence

## Baseline

Before implementation, the direct Basalt release verifier passed 201 Rust tests, two doctests, both Clippy scopes, vendored-source checks, package checks, and documentation checks.

Cairn proposal, design, and repository validation gates also passed.

## Reviewed upstream cohort

- UCAN revision: `c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e`
- Support catalog BLAKE3: `fe77fd63076eaef555b74e702e4c14e483bdba70151ccc955c75b17d97412427`
- Vendored manifest BLAKE3: `803c5204d7b28622b87e16d6c7e64bf0e4e5d7468b961d6579db2c84104ecb9d`
- Wire evidence BLAKE3: `bdca91b7f15351a1f5b56ce9eb524c467663f8985fd3e05440158609538b7a13`
- Wire specification BLAKE3: `3276ec9cfbfc504ea9be8278bc97869e448548eda410b411b24019c161d2bef3`
- Authorization evidence BLAKE3: `1aed251777a88ff08b1ca9124b5d5feedd1df395fd1d86c73e67cdf9423e13d2`
- Authorization specification BLAKE3: `706dd6ba6fb05e6def6c46f0c51a8b7937a92dc8baffa4a834c524ad414fb45d`

The repository importer reported source parity for all 693 vendored files.

The source contract rejects failed, missing, stale, unsupported, untested, profile-mismatched, manifest-mismatched, and license-mismatched evidence.

## Implementation

`ucan_v1_profile` adds a separate final-profile adapter. It accepts UCAN's final Invocation shell and does not change the compact adapter.

A Basalt replay gate performs these steps before the caller-owned final replay port:

1. Select one exact Nickel-exported command mapping.
2. Ask a caller-owned port for the exact subject and resource owner observation.
3. Bind the selected action to the exact Basalt request.
4. Run the existing pure Basalt authority core.
5. Build the metadata-only receipt and prospective private authority.
6. Call the final replay port once.

Unknown commands, incomplete owner facts, policy denials, profile substitutions, and mapping substitutions make zero final replay calls.

`LiveFinalProfileEnforcement` contains UCAN's private live authorization. Persisted receipts cannot construct that type.

## Checked fixtures

The Rust suite covers direct and delegated final Invocations, unknown commands, owner denial, policy denial, compact-profile substitution, final replay denial, deterministic mappings, stale sources, duplicate mappings, and redacted persisted receipts.

The Nickel suite contains one valid source record, one valid mapping, one final cohort, and 12 invalid fixtures.

## Validation

- Focused final-profile tests passed: five positive and negative cases.
- Strict Clippy passed for all targets and all features with denied warnings.
- The direct release verifier passed 195 no-default tests and 208 CLI-feature tests.
- Two doctests, property tests, fixture tests, package checks, and both release Clippy scopes passed.
- The final-profile Nix check passed with 12 rejected negative Nickel fixtures.
- Every repository Nix check passed with local builders.
- Cairn repository, proposal, and design validation passed.
- The eight new requirement identifiers have implementation or verification markers.
- The broader traceability report retains inherited advisory gaps outside this change.

## Non-claims

This evidence does not prove key trust, external resource ownership, global revocation freshness, replay durability, transport security, policy completeness, effect execution, effect correctness, release eligibility, or whole-system safety.
