# Core credential traceability audit

## Scope and success contract

This audit classifies 12 of the 26 missing credential scenarios at the core credential boundary. Success requires one explicit decision per selected ID, current safe file-byte evidence identities, exact preservation of the missing and dangling sets, no credential marker promotion, and still-unsuccessful global coverage.

Parent markers, test names, archived tasks, lexical similarity, unrelated absent sentinels, successful commands without exact accepted-clause assertions, and generated inventory rows were excluded as satisfaction evidence.

## Baseline

Before adding the audit contract, maintained coverage reported:

- requirements: 710
- referenced: 357
- missing: 353
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `8eac7421fbd9f64acda404ddf4936ce5268dbdad22ef876ac06392e293fcaaf5`
- receipt BLAKE3: `72e6ac2a7fae5933c17631506f54d38f7016203054b3e619f66f739319e35c06`
- missing-set BLAKE3: `0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1`

The existing credential and inventory test suites passed before audit changes.

## Portfolio result

Four mechanisms were evaluated:

- Parent-marker inheritance was falsified because a requirement-level marker does not establish every child scenario.
- Lexical and test-name matching was falsified because names such as `fails_before_shell_execution` are not assertions.
- Implementation-only inference was falsified for operator-visible save/export, diagnostics, and multi-surface redaction claims.
- Bounded accepted-clause review against deterministic source and direct assertions was selected.

The local VibeThinker endpoint reported reachable, but two specialized-review calls and one compatible chat call returned `fetch failed`. The advisory route was exhausted and supplied no audit evidence. Source-order and assertion-coverage reviews were performed as explicit serial lenses and are correlated rather than independent. Deterministic repository tests and receipts remain authoritative.

## Evidence identities

- `src/credentials.rs`: `415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0`
- `src/engine/executor.rs`: `d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664`
- `src/workflow_format.rs`: `d0a33436c867c123db1057560622a5fd9718e97430a785dd13e2db8c3cfc66ac`
- `src/workflow_validation.rs`: `49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374`
- `tests/test_credentials.rs`: `2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1`

## Exact maintained evidence

Eight rows are exact:

- `n8nclone.credentials.cache_identity.reference`: cache hashing includes environment binding and provider/key/field identity while excluding resolved values.
- `n8nclone.credentials.provider_interface.missing`: failed resolution returns before shell execution and the integration test observes the missing-secret failure.
- `n8nclone.credentials.provider_interface.resolve`: executor resolution, environment injection, and redacted shell observation establish the execution-boundary path.
- `n8nclone.credentials.provider_registry.resolve`: registry validation and delegation resolve the registered provider without workflow persistence.
- `n8nclone.credentials.provider_registry.unknown`: registry-backed validation emits a fatal stable unknown-provider diagnostic before runtime preparation.
- `n8nclone.credentials.providers.env.missing`: missing and empty environment values return `MissingSecret` before shell execution.
- `n8nclone.credentials.providers.env.resolve`: a valid environment value is returned through `SecretValue` and observed at the provider boundary.
- `n8nclone.credentials.validation.malformed`: an empty provider identity produces a fatal `MalformedSecretRef` diagnostic before execution.

## Partial evidence only

Four rows remain partial:

- `n8nclone.credentials.redaction.output`: shell stdout and the text helper are asserted, but logs, run history, JSON summaries, and persisted surfaces are not comprehensively covered by this scenario.
- `n8nclone.credentials.redaction.validation`: source preserves reference identity without a resolved value, but no maintained assertion checks rendered identity and secret absence together.
- `n8nclone.credentials.secret_refs.model.reject_plaintext`: Nickel import rejects an injected plaintext field, but the test does not assert a credential-specific diagnostic or no-save effect.
- `n8nclone.credentials.secret_refs.model.store_ref`: the model and serializers contain references and direct serialization excludes a sentinel, but the accepted save/export boundary is not exercised by an exact assertion.

No selected row is classified `no_exact_evidence_identified`; every partial row has bounded relevant evidence that does not satisfy all accepted clauses.

## Maintained inventory

The regenerated inventory records:

- rows: 353
- reviewed: 12
- exact: 8
- partial: 4
- no-exact: 0
- unassessed: 341
- credential rows: 26
- inventory issues: none
- inventory `valid: true`
- missing-set BLAKE3: `0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1`
- review-policy BLAKE3: `0f710370ea982d2418245420f1799867149bdbfc251071682254dc512787ce05`
- inventory BLAKE3: `d99adb105b25faaa0598535920def7ae781aa248dfa5607985ab3a757e722c45`

The remaining 14 credential rows and 327 non-credential rows remain `unassessed`. Inventory validity does not satisfy any requirement.

## Final coverage

The four accepted/referenced audit-contract IDs changed only repository-validation accounting. The credential missing set is unchanged:

- requirements: 714
- referenced: 361
- missing: 353
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `6ad53ac2bdbc49f87c091b3e2549f6426755e873dc7fb131548bcab6c815f4a3`
- receipt BLAKE3: `eacea95ada460ab890aecf787122102dcb5aa40eedf3e1142c6af658a243e545`

The maintained comparator proves the exact 353-item missing array is unchanged, both dangling arrays are empty, and both receipts remain invalid.

## Validation

- 15 existing credential tests pass.
- 2 credential-audit tests pass, including missing/unexpected rows, classification/rationale/evidence drift, missing-set additions/removals/duplicates/ordering, dangling markers, and invalid coverage-state mutations.
- 6 inventory tests pass.
- `scripts/check.rs` passes repository formatting, all tests, strict project-only Clippy, identity checks, and Cairn spec checks.
- Deterministic inventory generation and checking agree.
- Cairn policy freshness, repository validation, and proposal/design/tasks gates pass.
- Octet retains 913 accepted/current findings with zero new and zero stale IDs under baseline `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`. Its underlying Cargo process remains classified as exit code 101 while the reviewed zero-delta wrapper accepts it.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.
- Full workspace `cargo fmt` remains blocked by the known vendored Rats missing-module/edition traversal failure; targeted Rustfmt and repository/Nix formatting checks pass.

## Non-claims

This audit proves only the stated maintained-source/test classifications, evidence identities, inventory partition, and unchanged Tracey debt set. It does not prove arbitrary secret confidentiality, external provider safety, complete save/export behavior, complete diagnostic or persisted-surface redaction, remote execution safety, credential requirement satisfaction, complete traceability, release readiness, or production readiness.
