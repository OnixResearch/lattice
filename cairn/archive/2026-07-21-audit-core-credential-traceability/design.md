## Context

Maintained coverage currently reports 710 requirements, 357 referenced requirements, 353 missing identifiers, zero dangling identifiers, and `valid: false`. Twenty-six missing rows belong to `credentials`. This review selects 12 scenarios at the core credential boundary and leaves provider-specific vault/command, n8n interchange, webhook, cookbook, parity, and provider-probe scenarios for later tranches.

The selected evidence is bounded to `src/credentials.rs`, `src/engine/executor.rs`, `src/workflow_format.rs`, `src/workflow_validation.rs`, and `tests/test_credentials.rs`.

## Goals

- Classify the selected 12 credential rows exactly once.
- Preserve distinctions between exact and partial evidence.
- Bind every reviewed row to safe current file-byte BLAKE3 identities.
- Preserve the exact 353-item missing array, zero dangling markers, and unsuccessful coverage.
- Add deterministic positive and negative validation for the tranche.

## Non-Goals

- Adding credential Tracey markers.
- Changing credential behavior or tests to close identified gaps.
- Reviewing the other 14 missing credential rows.
- Treating parent markers, test names, lexical similarity, or archived tasks as scenario evidence.
- Claiming credential confidentiality, remote-provider safety, complete traceability, release readiness, or production readiness.

## Portfolio Search Contract

**Exact goal:** classify 12 currently missing core credential scenarios without changing their coverage state or runtime behavior.

**Completion evidence:** every selected ID has one explicit non-`unassessed` decision; the partition is eight exact and four partial; every row binds current maintained file identities; all other rows remain unchanged; before/after missing and dangling arrays are equal; coverage remains invalid.

**False completion:** adding credential markers, using parent markers as child-scenario proof, treating a test name as an assertion, calling incomplete save/export or redaction coverage exact, omitting an ID, reviewing an unrelated credential row, changing the missing set, or reporting successful coverage.

**Risks:** secret absence inferred from an unrelated sentinel, execution-order claims inferred only from function names, broad redaction claims inferred from one shell output, and validation errors treated as credential-specific diagnostics without checking their asserted shape.

**Budgets:** the credentials and repository-validation specs; five maintained source/test files; current inventory policy/tool/report; one advisory local-model attempt; no network retrieval; one classification round and one adversarial validation round.

**Allowed outcomes:** validated classification, an exact bounded blocker, exhausted advisory budget with deterministic repository evidence, or user decision if evidence authority is ambiguous.

## Approach Registry

| Family | Mechanism | State | Evidence or blocker |
|---|---|---|---|
| parent-marker inheritance | Treat existing requirement-level markers as all child-scenario evidence | falsified | A parent marker does not prove each child GIVEN/WHEN/THEN clause. |
| lexical/test-name matching | Infer exactness from names such as `fails_before_shell_execution` | falsified | Assertions and direct source ordering, not names, determine evidence strength. |
| implementation-only inference | Treat type shape and control flow as sufficient for every operator-visible claim | falsified | Save/export, diagnostic rendering, and multi-surface redaction require direct maintained observations. |
| bounded clause review | Compare accepted clauses with deterministic source and direct assertions | selected | It preserves exact versus partial strength without changing coverage. |

The local VibeThinker endpoint reported reachable, but two specialized-review calls and one compatible chat call returned `fetch failed`. Advisory review is therefore exhausted and not used as evidence. The serial source-order and assertion-coverage lenses are correlated; deterministic tests and inventory validation remain authoritative.

## Classification Decisions

### Exact maintained evidence

- `provider_interface.resolve`: executor control flow resolves references before shell execution, supplies the environment value, and the integration test observes the resolved value only as redacted shell output.
- `provider_interface.missing`: the resolver returns a failed node before `execute_shell`, and the integration test observes the missing-secret failure.
- `validation.malformed`: validation emits a fatal malformed-reference diagnostic and the test observes its stable code.
- `cache_identity.reference`: the cache core hashes environment binding plus provider/key/field identity only; tests distinguish reference identities and preserve keys across different resolved values.
- `provider_registry.resolve`: the registry validates and delegates to the registered provider, while the test observes exact resolution and clean registry-backed validation.
- `provider_registry.unknown`: registry-backed validation emits a fatal unknown-provider diagnostic before runtime preparation, and the test observes the stable code.
- `providers.env.resolve`: the provider validates and returns a `SecretValue`, and the test observes successful resolution.
- `providers.env.missing`: missing and empty values return `MissingSecret`; executor control flow returns before shell execution.

### Partial evidence only

- `secret_refs.model.store_ref`: the model and serializers contain only `SecretRef` fields, and a direct serialization test excludes a sentinel value, but no maintained assertion exercises the accepted save/export boundary.
- `secret_refs.model.reject_plaintext`: Nickel import rejects an injected `value` field, but the test does not assert a credential-specific diagnostic or a no-save effect.
- `redaction.output`: shell stdout and the redaction helper are asserted, but logs, run history, stderr/JSON summaries, and persisted surfaces are not comprehensively covered by this scenario test.
- `redaction.validation`: source diagnostics carry secret-reference identity without resolved values, but no maintained assertion checks rendered identity preservation and secret-material absence together.

No selected row is classified `no_exact_evidence_identified`; each partial row has bounded relevant source or test evidence, but not enough to satisfy every accepted clause.

## Evidence Identities

- `src/credentials.rs`: `415ece186e976d49dea538975fcc4726ca97c26753630a8271084cbea8d2a4b0`
- `src/engine/executor.rs`: `d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664`
- `src/workflow_format.rs`: `d0a33436c867c123db1057560622a5fd9718e97430a785dd13e2db8c3cfc66ac`
- `src/workflow_validation.rs`: `49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374`
- `tests/test_credentials.rs`: `2c9d7d84f33d5d62f9b75d60478ea868fbeb9fd8713838e642f045fbf773edf1`

## Validation Design

The existing pure tranche validator compares an explicit expectation table with generated inventory rows and validates classifications, rationales, evidence paths, duplicates, omissions, unexpected selected-subsystem rows, and unchanged coverage snapshots. The test shell deserializes checked-in reports and hashes compile-time file bytes.

Negative mutations cover missing and unexpected rows, classification/rationale/evidence drift, removed and added missing IDs, duplicates, unsorted inputs, dangling markers, and an incorrectly successful coverage state.
