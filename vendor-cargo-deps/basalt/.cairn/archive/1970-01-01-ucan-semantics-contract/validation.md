# Validation Evidence

## Bound identities

- Implementation commit: `5920fabade199f681e75bfbb68c2b1be15d80f06`
- Fixture schema: `basalt.ucan-semantics-fixture.v1`
- Fetchable Basalt runtime UCAN revision: `2aad993027d48ff148028c537cdaf91f6e5285ca`
- Reviewed UCAN support-catalog revision: `e362c3fa2236d5b9d61423921a820cd12dd64e51`
- UCAN support-catalog BLAKE3: `02fb9fec563b5df9f40a58bbbddfd0bcf5abe74da373ae0b506712647e0ee6ad`
- Generated Basalt policy BLAKE3: `7d223f48abcf31c9ba932d7434babfa675122f393100547ff09a595ee1fe8adb`
- UCAN semantics contract-tree BLAKE3: `4d07dd015d3541cae4de35f9925e8a2a5af8873535b950db0e7220ab8bbdf4ad`
- Saved release receipt BLAKE3: `33d2b2e1bc1024e575829d184efd3dd46972bbe6f6b63416e38052ffa2fa27b1`

The release receipt is saved at `evidence/release-receipt.json`. It binds both UCAN revisions, the support catalog digest, the Basalt policy digest, and successful policy-freshness, fixture-freshness, and runtime-replay results.

The runtime and catalog revisions are intentionally distinct. An attempted immutable runtime-pin update could not fetch the catalog revision from GitHub because that UCAN commit is three local commits ahead of `origin/main` and is not available from a remote ref. No push was performed and no workspace-relative release dependency was introduced. Basalt therefore retains its existing fetchable exact runtime revision while identifying the reviewed catalog source separately.

## Baseline

Before implementation:

- `cargo test --locked --test production_ucan_token_flow --no-default-features` passed all 3 existing token-flow tests.
- The policy-export Nix check passed.
- Proposal, design, and tasks gates passed for this change.
- Current-schema root Cairn validation was already blocked only by the two accepted legacy specs described below.

## Fixture contract and replay

The checked matrix contains two positive fixtures, five fail-closed semantics fixtures, and four malformed contract fixtures.

- Positive: matching policy/capability allow and matching policy/capability denial.
- Fail closed: over-delegation, wildcard operation request, missing caveat policy, stale proof traversal, and replay-backend failure.
- Malformed: missing operation, weakened non-claims, incompatible expected result, and stale catalog identity.

The following checks passed on the implementation commit:

- `nix build path:$PWD#checks.x86_64-linux.ucan-semantics-fixtures --no-link -L --option secret-key-files ''`; all seven valid Nickel fixtures reproduced their checked-in JSON byte-for-byte and all four malformed fixtures failed export.
- `cargo test --locked --test ucan_semantics_contract --no-default-features`; 7 of 7 tests passed.
- `cargo clippy --locked --all-targets --no-default-features -- -D warnings`.
- `cargo package --locked --allow-dirty --no-verify --list`; all required contract, fixture, adapter, documentation, and test paths were present.

The adapter validates immutable identities, bounded capability counts, UCAN capability syntax, exact non-claims, and expectation coherence. It rejects every non-admitted caller-supplied UCAN decision before combining admitted direct and effective delegated grants and calling the existing Basalt `enforce` core.

## Release and Nix rails

The direct `tools/release-verify.rs` rail passed every step:

- policy and UCAN-semantics fixture freshness;
- 54 no-default-feature tests and 54 CLI-feature tests;
- the 7 focused semantics replay tests;
- property, Steel-adapter, and production UCAN-token fixtures;
- doctests and strict Clippy for both feature profiles;
- package validation and asserted 40-file package contents;
- downstream fixture discovery and library documentation.

The direct summary BLAKE3 is `4e10b6060c323814655d51e05ce2b7fb5e03711a9469c6b50cfa701982e80513`; the direct log BLAKE3 is `37a70717f385b8a7c772f404e37cab0ec98232aed3247c48e79b4293369ef190`.

`nix flake check path:$PWD -L --option secret-key-files ''` also passed, including the policy export, deterministic fixture export, downstream compatibility configuration, and sandbox-safe release receipt. The flake log BLAKE3 is `e6414214d0479cc870328212ab79198c288e728bc28bfc58ddc3a19a27544a46`.

The empty `secret-key-files` override accommodates this machine's absent configured Nix signing key. Failed remote-builder connection attempts fell back to the local builder and did not change derivations or results.

## Cairn lifecycle

Policy-compatible Cairn revision `8774a1f8fa2bea8c8e7bc6741ae31d56563c8802` validated all 7 active changes and 11 accepted specs with no issues.

Current proposal, design, and tasks gates pass for `ucan-semantics-contract`:

- Proposal receipt: `28189e6f9eec1620e67ab37c23c23887fa281afe657f33b41fab598e128e7b0d`
- Design receipt: `41f2ca7f6ea6322855b8f65fddbf37d6b7a65cac12da8d55e8fc8580a6afc29a`
- Tasks receipt: `b2250ffb9956d1ef007f77d45153538f11d25a5296e6a85074c7a42a458787f7`

The executed sync used plan hash `797d1eec76d0f30207cc024e492e35e47f234776169bdbcd84ac53c7b8f15db2` and receipt hash `bff42f7b4a25fe52b9f2a37af6b59d364209f55cb3d42f96e9f1fec936747f5d`. All 11 `ucan_semantics_contract.*` requirement and scenario IDs are present in `cairn/specs/ucan-token-flow/spec.md` after execution.

At baseline, current-schema root validation was blocked by two unrelated accepted legacy specs, `cairn/specs/test-hardening/spec.md` and the then-empty `cairn/specs/ucan-token-flow/spec.md`, whose historical headings predated the current substance parser. Syncing this change supplied four substantive requirements and seven scenarios to `ucan-token-flow`; current validation is now blocked only by the unrelated `test-hardening` legacy spec. The target change has no change issues and passing current gates. Lifecycle mutation uses the policy-compatible Cairn revision; postconditions are checked with current gates and exact accepted-spec ID inspection.

## Non-claims

This evidence does not establish revocation-backend freshness, key-resolution truth, UCAN cryptographic correctness, consumer replay-store durability, or application-level authorization completeness. The fixtures consume bounded UCAN decision facts as data; they do not prove those facts' provenance or reimplement token, delegation, caveat, proof, revocation, replay, clock, or key-resolution semantics.
