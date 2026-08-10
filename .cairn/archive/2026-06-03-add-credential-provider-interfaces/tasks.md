## Phase 1

- [x] [serial] r[n8nclone.credentials.secret_refs.model] Add opaque secret-reference types for workflow and node configuration.
- [x] [serial] r[n8nclone.credentials.provider_interface] Resolve secret references through a credential-provider interface at execution time.
- [x] [parallel] r[n8nclone.credentials.redaction] Apply deterministic redaction to TUI, CLI, logs, run history, and JSON summaries.
- [x] [parallel] r[n8nclone.credentials.validation] Add validation and negative tests for malformed or unresolved secret references.
- [x] [parallel] r[n8nclone.credentials.cache_identity] Verify cache keys use secret references without hashing plaintext secret material.
