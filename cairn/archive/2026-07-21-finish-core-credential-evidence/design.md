## Context

The maintained credential audit records 12 reviewed missing scenarios: eight exact and four partial. No credential marker is promoted. Coverage remains 714 requirements, 361 referenced requirements, 353 missing identifiers, zero dangling identifiers, and `valid: false`.

## Goals

- Reject any `secret_env.value` payload through a stable typed error before opening the workflow store.
- Prove reference-only save/export after actual secret resolution.
- Redact overlapping resolved values deterministically from execution output before logs, history, JSON, or persistence can observe them.
- Prove validation output retains safe reference identity and omits plaintext values.
- Reclassify all 12 audited rows as exact while preserving missing coverage.

## Non-Goals

- Promoting credential markers in this change.
- Encrypting the workflow store or adding credential providers.
- Persisting resolved values for retries, replay, caching, or diagnostics.
- Claiming host-process confidentiality, external provider correctness, complete credential coverage, or release readiness.

## Portfolio Search Contract

**Exact goal:** close the four audited evidence gaps without increasing credential authority or changing the 353-item missing set.

**Completion evidence:** typed safe import rejection and no-store observation; post-resolution save/export excludes the resolved value; stdout, stderr, captured tracing output, run-detail JSON, and persisted details exclude resolved values; validation text/JSON retain reference identity and omit a plaintext sentinel; all 12 audit decisions are exact and hash-bound; coverage is unchanged and invalid.

**False completion:** absence checks against a value never supplied, matching only a test name, redacting after persistence, replacing reference identity, accepting arbitrary plaintext through a compatibility field, adding credential markers, or reducing unrelated traceability debt.

**Budgets:** four maintained source files, focused credential/history/store tests, current inventory machinery, one advisory review attempt, no network retrieval, one implementation round, one adversarial round, and repository/Cairn/Octet/Nix validation.

**Allowed outcomes:** validated completion, an exact bounded blocker, exhausted advisory budget with deterministic evidence, or user decision if compatibility requires plaintext import.

## Approach Registry

| Family | Mechanism | State | Reason |
|---|---|---|---|
| assertions only | Add tests without changing import or redaction cores | falsified | Current import errors are not credential-specific and replacement order can leak overlapping suffixes. |
| downstream filters | Redact independently in logs, history, and persistence | falsified | Duplicated policy can drift and permits a pre-filter leak. |
| typed boundary plus normalized core | Reject plaintext during portable conversion and redact longest-first before output construction | selected | It centralizes deterministic policy while retaining thin I/O shells. |
| encrypted persistence | Store resolved values under encryption | stronger than goal | The accepted model forbids resolved-value persistence rather than requiring encrypted secret storage. |

## Functional Core and Shell

`workflow_format` remains the pure conversion boundary after Nickel evaluation. `PortableSecretEnvVar` accepts a dynamic optional `value` only so Rust can return `PlaintextCredentialMaterial` deterministically; conversion rejects it before constructing `Workflow`. The CLI continues to load the workflow before opening `Store`, so rejection cannot create or mutate the database.

`credentials::redact_text` remains a pure string transformation. It removes empty values, sorts longest-first with a lexical tie-break, deduplicates, and then replaces. Executor output construction invokes it before `StepResult` exists. Run-detail JSON and persistence therefore receive only redacted values.

Tests own temporary files, process invocation, tracing capture, and store inspection. No filesystem or process behavior enters the cores.

## Claim Boundaries

The store/export proof observes only repository-owned workflow serialization and redb persistence. The output proof covers resolved values emitted by the bounded shell test and the maintained local surfaces named in the accepted scenario. It does not prove confidentiality against debuggers, child processes before capture, kernel observers, compromised adapters, or external systems.
