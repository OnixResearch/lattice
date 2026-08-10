# Core credential Tracey promotion

## Scope

This change promotes only the 12 credential scenario IDs classified as exact in the completed bounded review. It changes marker linkage and repository validation evidence only; it does not change credential runtime, import, provider, persistence, or redaction behavior.

## Exact allowlist

- `n8nclone.credentials.cache_identity.reference`
- `n8nclone.credentials.provider_interface.missing`
- `n8nclone.credentials.provider_interface.resolve`
- `n8nclone.credentials.provider_registry.resolve`
- `n8nclone.credentials.provider_registry.unknown`
- `n8nclone.credentials.providers.env.missing`
- `n8nclone.credentials.providers.env.resolve`
- `n8nclone.credentials.redaction.output`
- `n8nclone.credentials.redaction.validation`
- `n8nclone.credentials.secret_refs.model.reject_plaintext`
- `n8nclone.credentials.secret_refs.model.store_ref`
- `n8nclone.credentials.validation.malformed`

Each marker is a complete closed marker-only line immediately attached to an exact maintained test in `tests/test_credentials.rs`. Combined registry and environment tests carry the two scenario IDs they directly assert. Plaintext rejection is attached to both the typed non-null-shape test and the CLI no-store test because the complete accepted scenario spans those maintained assertions.

The local-binding validator recognizes only `#[test]` and `#[tokio::test]` functions and checks every required test-name/ID pair. It does not accept file scope, test-name similarity, generated inventory rows, parent markers, implementation comments, or archived evidence.

## Historical exact review

The pre-promotion reviewed fixture remains pinned to the 353-item baseline:

- reviewed: 12
- exact: 12
- partial: 0
- unassessed: 341
- test evidence BLAKE3: `cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835`
- missing-set BLAKE3: `0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1`
- review-policy BLAKE3: `9cfeadc9cb728af74c5050d6705923ec4c4cff46267bac9c102bdd8b4bd0b80d`
- inventory BLAKE3: `ca16bc8b9c1714b208afd6e5d183f20b04a3b95250fda14844c7de4a4d25b8c4`

The audit test retains this historical baseline/current comparison while requiring every exact scenario marker in the maintained credential test source. The separate promotion test owns the set-transition and local-attachment proof.

## Exact set transformation

The pure promotion comparator validates sorted duplicate-free baseline/current sets and an explicit duplicate-free allowlist. It rejects a missing expected removal, unrelated removal, added missing ID, dangling marker, mismatched profile, or successful coverage.

Observed transition:

- baseline requirements: 714
- baseline referenced: 361
- baseline missing: 353
- current requirements: 718
- current referenced: 377
- current missing: 341
- baseline minus current: exactly the 12-ID allowlist
- current minus baseline: empty
- baseline dangling: 0
- current dangling: 0
- baseline `valid: false`
- current `valid: false`

The four added and referenced requirements are the maintained promotion contract and its three scenarios; they do not change the missing-set delta.

Final coverage identity:

- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `a2f6bffdc624bc089900ca1372cf76f968746abec68fd25a14fc5a484cf7c19c`
- receipt BLAKE3: `d581a3a5e5e6d4450502170d780c3988548bcaa4306c864592ec00326acdd344`

## Post-promotion debt inventory

Promoted IDs are absent from `cairn-policy/traceability-inventory.ncl`. The regenerated inventory records:

- total: 341
- reviewed: 0
- unassessed: 341
- issues: none
- inventory `valid: true`
- missing-set BLAKE3: `0070258e8dbf77a97ca7d34a8880b297712733811cf6350b677301b617823e77`
- review-policy BLAKE3: `e27f763824e3823ae585a337d52c94b2edfb7027a28f42861780a08d6bac07de`
- inventory BLAKE3: `6d9d91702caa17429117b6f9989e05332c0bbc6d5ebe0a440151c36efa5c4ad9`

`unassessed` does not mean behavior or evidence is absent. Inventory rows do not satisfy requirements or create Tracey evidence.

## Negative validation

Maintained mutations prove deterministic rejection of:

- duplicate and unsorted baseline missing IDs;
- duplicate and unsorted current missing IDs;
- a duplicate allowlist ID;
- a missing expected removal;
- an unrelated removal;
- an unexpected added missing ID;
- a dangling marker; and
- incorrectly successful global coverage.

## Validation

- 17 credential tests pass.
- Two exact credential-audit tests pass.
- Three credential-promotion tests pass.
- Six inventory tests pass.
- `scripts/check.rs` passes repository formatting, all tests, strict project-only Clippy, identity checks, and accepted Cairn spec checks.
- Cairn policy freshness, validation, sync, and proposal/design/tasks gates pass.
- Deterministic inventory generation and checking agree.
- Octet retains 912 accepted/current findings with zero new and zero stale under `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`. The underlying Cargo subprocess remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.

## Non-claims

This promotion proves only the exact checked marker, set-transition, inventory, and fixture observations. It does not promote the remaining 14 credential scenarios, prove n8n parity, establish arbitrary secret confidentiality, complete credential or repository traceability, or establish release or production readiness.
