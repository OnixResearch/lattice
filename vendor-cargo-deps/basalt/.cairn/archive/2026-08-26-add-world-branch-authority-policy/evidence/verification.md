# World-branch authority policy verification

Verified on 2026-08-26 in the dedicated Basalt worktree.

## Baseline

- The existing Nickel policy export and runtime-effect policy checks passed with local builders.
- Focused durable-authority replay and revocation tests passed.
- Existing runtime-effect permission positive and negative tests passed.

## Nickel and generated policy

The pinned Nickel `1.17.0` cohort typechecked and exported:

- `contracts/world-branch-authority/policy.ncl`;
- `contracts/world-branch-authority/consumer-mappings.ncl`.

Generated JSON matches the checked-in exports byte for byte. Six hostile Nickel fixtures reject unknown mode, unsafe default, stale generation, weakened non-claims, lossy mapping, and duplicate rules.

`nix build path:$PWD#checks.x86_64-linux.world-branch-authority-policy -L --builders ''` passed.

## Rust policy core

Six focused unit tests passed. They cover all seven modes and deny these cases:

- authority widening;
- non-narrow attenuation;
- missing UCAN composition;
- stale revocation or replay facts;
- ambiguous rules;
- lossy mapping;
- linear copying;
- simulation live fallback;
- promotion bypass;
- malformed mode input;
- bearer material;
- receipt overclaim.

Two external API tests passed with default-disabled and CLI-enabled feature cohorts. The consumer mapping retains Basalt policy ownership and Molten runtime enforcement ownership.

Clippy passed for all targets with warnings denied under both feature cohorts.

## Release rail

The direct release verifier passed. Its receipt reports:

- `passed: true`;
- 243 tests passed;
- two doctests passed;
- vendored UCAN revision `c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e` and 693-file snapshot verified;
- library and CLI Clippy passed;
- private package contents include the branch policy, generated artifacts, docs, source, and external tests;
- library documentation built.

The release receipt is at `target/release-verify/summary.json`. This generated target artifact is not committed.

## Claim boundary

Basalt decisions and receipts describe portable policy meaning only. They do not mint, derive, transfer, activate, promote, or execute capabilities.

Molten retains token parsing, signature checks, currentness, revocation and replay observations, durable transfer, adapter selection, activation, promotion, and effects.
