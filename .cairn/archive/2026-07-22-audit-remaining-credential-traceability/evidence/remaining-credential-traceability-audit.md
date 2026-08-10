# Remaining credential traceability audit

## Scope and non-claims

This audit covers exactly the 14 credential rows left unassessed after the 12-ID core credential promotion. It records current evidence strength without changing credential behavior or adding any reviewed credential marker.

An exact classification is not requirement satisfaction. Partial evidence is not a near-complete correctness claim. `no_exact_evidence_identified` would describe only the bounded maintained roots and would not prove global absence; this tranche has no such row because each reviewed ID has concrete maintained subset evidence. Inventory rows do not create Tracey evidence.

The audit does not prove n8n parity, provider truth, arbitrary secret confidentiality, complete traceability, release readiness, or production readiness.

## Portfolio and adversarial audit

Four mechanism families were evaluated:

1. Parent/child marker or archive inference was rejected because related IDs and completed lifecycle claims are not maintained evidence for the reviewed row.
2. Implementation shape identified meaningful subset evidence but was rejected as exact authority for export, persistence, rendered diagnostics, execution boundaries, and multi-surface claims.
3. Binary exact/absent classification was rejected because it would erase concrete partial evidence.
4. Clause-by-clause maintained source and assertion comparison was selected.

The adversarial pass specifically checked the deliberate plaintext parity sentinel, imported-placeholder resolution, credential-bearing export, provider identity dimensions, direct-provider versus executor boundaries, timeout/no-runtime behavior, rendered/persisted diagnostics, webhook safe context, and cookbook artifacts/smoke observations.

A local VibeThinker advisory review noticed the parity sentinel but recommended collapsing all 14 rows to no-exact. That conclusion was rejected because it did not distinguish concrete partial source/assertion evidence from absence. Deterministic repository artifacts, policy checks, and tests remain authoritative.

## Classification result

### Exact maintained evidence: 1

- `n8nclone.credentials.webhook_cookbook_secret_safety`
  - rationale: `assertions.cookbook-artifacts-and-smoke-surfaces`
  - evidence: `examples/README.md`, `examples/workflows/webhook-cookbook.ncl`, `src/triggers.rs`, `src/webhooks/controls.rs`, `src/webhooks/flow.rs`, `tests/webhooks.rs`
  - bounded finding: maintained tests inspect credential-reference workflow/docs artifacts, accepted safe history, rejected responses/history, and no-run outcomes while checking provider values, authorization values, bad tokens, and sensitive payload fields are absent from observed smoke surfaces.

### Partial evidence only: 13

- `n8nclone.credentials.n8n_json.placeholders.export`
  - rationale: `assertions.credential-export-observation-missing`
  - gap: export code emits reference identity, but no maintained test exports a credential-bearing workflow and checks output/report bytes for identity and plaintext omission.
- `n8nclone.credentials.n8n_json.placeholders.plaintext`
  - rationale: `assertions.plaintext-save-boundary-missing`
  - gap: core/parity tests reject plaintext and omit the sentinel from reports, but no CLI/store assertion proves the rejected n8n workflow is never saved.
- `n8nclone.credentials.n8n_json.placeholders.reference`
  - rationale: `assertions.imported-placeholder-resolution-missing`
  - gap: import asserts the generated environment slot, but no maintained test executes the imported placeholder through a configured provider.
- `n8nclone.credentials.n8n_parity_secret_safety`
  - rationale: `contract.fixture-sentinel-and-output-gaps`
  - gap: child tests check report JSON, but the parent contract names fixtures and test output while the unsupported rejection fixture deliberately contains the plaintext sentinel.
- `n8nclone.credentials.provider_cache_identity.no_plaintext`
  - rationale: `assertions.reference-dimensions-incomplete`
  - gap: a test proves two different resolved values leave one reference key unchanged, but does not independently vary provider, key, and optional field identities.
- `n8nclone.credentials.provider_diagnostics.redaction.error`
  - rationale: `assertions.error-fragment-source-missing`
  - gap: the provider probe removes a missing key from one error message, but no failing provider emits text derived from resolved secret material for rendered redaction.
- `n8nclone.credentials.provider_diagnostics.redaction.test`
  - rationale: `assertions.render-persist-observation-missing`
  - gap: the probe result message omits a resolved value, but no user rendering or persistence boundary is directly observed.
- `n8nclone.credentials.providers.command.resolve`
  - rationale: `assertions.executor-boundary-missing`
  - gap: direct provider tests resolve JSON and reject fixed failures, but do not observe executor injection or rendered command-output redaction.
- `n8nclone.credentials.providers.command.timeout`
  - rationale: `assertions.no-runtime-observation-missing`
  - gap: the provider returns a timeout failure, but no executor test proves the side-effecting node runtime is not invoked.
- `n8nclone.credentials.providers.local_vault.resolve`
  - rationale: `assertions.workflow-file-observation-missing`
  - gap: direct provider decryption succeeds, but no maintained observation binds the result to an execution boundary and secret-free workflow bytes.
- `n8nclone.credentials.providers.local_vault.tamper`
  - rationale: `assertions.rendered-tamper-diagnostic-missing`
  - gap: wrong-key and tampered ciphertext return `TamperedSecret`, but no rendered diagnostic assertion proves safe identity context and plaintext omission.
- `n8nclone.credentials.webhook_auth.redaction`
  - rationale: `assertions.safe-reference-context-missing`
  - gap: response and run-record debug observations omit secrets, authorization data, and payload fields, but do not directly assert retained safe trigger and credential-reference context.
- `n8nclone.credentials.webhook_auth.valid`
  - rationale: `assertions.persistence-observation-missing`
  - gap: provider resolution and admission are observed, but the generic scenario lacks a direct persisted-byte assertion after admission.

### No exact evidence identified: 0

Every reviewed row has concrete maintained implementation or assertion evidence for part of its accepted contract.

## Evidence identities

- `src/credentials.rs`: `fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a`
- `src/engine/executor.rs`: `d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664`
- `src/n8n_json.rs`: `668c5adac6f449dfc597108af1ff57fea2e1cacfa3d443a1807a50c16db4f8e3`
- `src/triggers.rs`: `329343ff4685dbac1c6ace1872c22ad280d23958111d68d9a17f4fa06195e467`
- `src/webhooks/controls.rs`: `942f7286a689511ff37359bff729a8d21c2166235f7ab59a3f6d282d7ee7da76`
- `src/webhooks/flow.rs`: `a6c4b933b7c15c4e1adbb2d3a246b3d056bddb51f20aecd5faa0ce1a6ae36922`
- `tests/test_credentials.rs`: `cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835`
- `tests/test_n8n_json.rs`: `fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776`
- `tests/test_triggers.rs`: `1c3a189b10748ebc1d0d5e6884ee96f9b5cb98107b65798a279752251898848a`
- `tests/webhooks.rs`: `52270b64ffbd19fb46b229107049998df30d49bec3bf8b7c724f698e435b5011`
- `tests/fixtures/n8n/plaintext-credentials.json`: `2cf81f633764559cf062cb15d188543947e875fff946a4da3050ac8c12cd8cc3`
- `tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json`: `343a47bab676d60d3df80b43c45b5f062c8b004bcf4792705017f8055017f8ff`
- `examples/README.md`: `9ba2964ecb63eb81823898b1243d0c391e9b4772533ee0958b98cfdeb91e91bb`
- `examples/workflows/webhook-cookbook.ncl`: `e449f943f0af5f6b2f49ce4dca45d9279ca2897410d63b57e3163bff86d03e95`

## Maintained inventory

- total rows: 341
- reviewed rows: 14
- exact rows: 1
- partial rows: 13
- no-exact rows: 0
- unassessed rows: 327
- missing-set BLAKE3: `0070258e8dbf77a97ca7d34a8880b297712733811cf6350b677301b617823e77`
- review-policy BLAKE3: `267299b9a354362efd005e13e6b21326901c5ab4159b95ee3de2df0016dee82b`
- inventory BLAKE3: `2f0454bd877acac89f54b8972864d70b8d76371b4f965f21a06ab4341107d851`
- issues: none
- inventory `valid: true`

## Coverage invariant

The audit contract and its three scenarios add four accepted and four referenced IDs. Credential linkage remains unchanged:

- requirements: 722
- referenced: 381
- missing: 341
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `b8e4756bbdcfcaff5eca23b23c48dff5713b43ccb14a5fb3c6bb60c2fd2cac10`
- receipt BLAKE3: `bbbbf2471c6a176208e3fe05c334027fd74f6a4f65fc6ddd6e0d02e48bd6e9c5`

The pure tranche validator proves the baseline and current missing sets are identical, both dangling arrays are empty, both remain invalid, and none of the 14 reviewed IDs has an implementation or verification marker in the audited maintained sources.

## Positive and negative validation

- Baseline audit and inventory tests pass before changes.
- Two remaining-credential audit tests and six inventory tests pass.
- Positive validation checks exact row membership, classification counts, rationale codes, evidence paths, current file hashes, absent selected markers, and unchanged coverage.
- Negative mutations reject missing/unexpected rows, classification/rationale/evidence drift, removed/added/duplicate/unsorted missing IDs, dangling markers, and incorrectly successful coverage with deterministic issue codes.
- `scripts/check.rs` passes repository formatting, all tests, strict project-only Clippy, identity checks, and accepted Cairn spec checks.
- Cairn validation, sync, and proposal/design/tasks gates pass.
- Deterministic inventory generation and checking agree.
- Octet retains 912 accepted/current findings with zero new and zero stale under `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`. Its underlying Cargo subprocess remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.
