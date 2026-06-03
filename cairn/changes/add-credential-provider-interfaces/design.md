## Context

Shell nodes can already carry environment variables, and HTTP nodes will need headers or authentication. Without a credential abstraction, implementations are likely to store plaintext secrets in workflows or scatter environment lookups through execution code.

## Decisions

### 1. Workflows store secret references, not secret values

**Choice:** Introduce a `SecretRef` model that identifies a provider, key, and optional field while keeping secret material out of serialized workflows.

**Rationale:** Workflow files remain shareable and reviewable without leaking credentials.

### 2. Secret resolution uses an interface

**Choice:** Define a `CredentialProvider` interface for resolving `SecretRef` values at the imperative boundary. Initial adapters can support environment variables or local config, while future adapters can use keyrings or external vaults.

**Rationale:** Feature 6 is cross-cutting; an interface keeps execution nodes independent of any concrete secret backend.

### 3. Redaction is shared and deterministic

**Choice:** Add a redaction core that tags secret-bearing values and renders masked forms consistently.

**Rationale:** TUI, CLI, logs, run history, and JSON summaries should apply the same redaction behavior.

### 4. FCIS boundary prevents secret I/O in core

**Choice:** Pure validation may check that a `SecretRef` is syntactically valid, but only the shell/provider adapter may read environment variables, files, keyrings, or network-backed vaults.

**Rationale:** The functional core stays deterministic and secret resolution remains auditable.
