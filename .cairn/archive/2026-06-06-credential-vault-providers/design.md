# Design: Credential vault providers

## Architecture

Keep workflow credentials as references and add provider implementations at the execution boundary.

```text
Node credential refs
      │
      ▼
CredentialProviderRegistry ─ provider lookup and capability validation
      │
      ▼
Provider implementation ─ env, local vault, external command
```

The functional core validates reference shape, provider identity, required capabilities, and redaction policy. Provider implementations are imperative shells that read environment variables, decrypt local entries, or invoke external commands.

## Provider registry

A provider registry maps provider ids to implementations and metadata. Metadata declares whether the provider supports fields, listing, test resolution, write/update, and redaction-safe display names. Execution receives only resolved secret facts needed by a node; cache identity continues to use reference identity, not resolved secret values.

## Local vault

The local vault stores secret records encrypted at rest. The vault file should contain provider metadata, entry ids, encrypted payloads, nonce/salt metadata, and integrity hashes. Key material comes from explicit config: environment variable, OS keychain integration when available, or passphrase prompt in an interactive shell. Workflow files never embed vault payloads.

## External command provider

An external command provider invokes a configured command with explicit args/env/cwd and receives secret values over stdout or structured JSON. It should use the shared `ProcessRunner` once available, enforce timeouts/output limits, and redact command output from diagnostics.

## Validation and UX

Validation reports malformed provider ids and unsupported secret field access before execution. TUI/CLI provider tests may report reachable, missing, unauthorized, or timed out without printing secret values. Diagnostics include provider id and key identity only when safe.

## Testing

Use fake providers for positive and negative resolution tests. Use local vault fixture tests for wrong key, tampered data, missing entry, duplicate entry, and successful round trip. Use fake process runner tests for command provider timeout, nonzero exit, malformed JSON, and output-limit failures.
