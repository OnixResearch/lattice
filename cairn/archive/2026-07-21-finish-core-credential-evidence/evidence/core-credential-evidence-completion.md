# Core credential evidence completion

## Scope

This change closes the four partial rows from the bounded core credential audit and reclassifies all 12 selected credential scenarios as exact maintained evidence. It deliberately adds no credential marker; promotion remains a separate change.

## Typed plaintext rejection

`PortableSecretEnvVar` accepts an optional dynamic `value` field only at the Nickel-to-portable decoding boundary. Pure portable conversion rejects every non-null value with `WorkflowFormatError::PlaintextCredentialMaterial { node_id }` before constructing `Workflow`. The diagnostic contains a fixed category and numeric node identity, not the supplied value.

The CLI loads and converts the workflow before opening `Store`. The maintained CLI test supplies a real plaintext value, observes the stable safe diagnostic on failure, proves stdout/stderr omit the value, and proves the configured redb path was never created. Additional cases reject string, nested record, array, and numeric values. Missing and explicit-null compatibility cases remain reference-only and import successfully. Export construction always sets the internal field to `None` and never renders it.

## Save and export boundary

The maintained test resolves `super-secret-token` through the real executor boundary, then saves and reloads the same workflow and renders its Nickel export. JSON, Nickel, and raw redb bytes contain provider/key reference identities and omit the resolved value. This binds the absence check to a value that was actually resolved during the test rather than an unrelated sentinel.

## Redaction core and surfaces

`credentials::redact_text` is deterministic: it removes empty values, sorts longest-first with a lexical tie-break, deduplicates, and replaces before output construction. The negative overlap case supplies `super-secret-token` and its `secret-token` suffix in adversarial order and proves no suffix remains.

A credential-backed shell emits the resolved value to both stdout and stderr. The maintained test proves:

- stdout and stderr contain `<redacted>` and omit the value;
- `StepResult` debug output omits the value;
- captured tracing output retains the safe node label and omits the value;
- run-detail JSON contains `[redacted]` and omits the value;
- saved and reloaded run details contain `[redacted]` and omit the value; and
- raw redb bytes omit the value.

Redaction occurs before `StepResult` construction, so downstream history and persistence receive only redacted values.

## Validation rendering

A registry contains a real plaintext value while a field-bearing reference fails capability validation. The resulting fatal diagnostic is rendered as text and JSON. Both preserve the full safe reference identity and omit the configured plaintext value.

## Portfolio audit

Four mechanism families were considered:

- assertions-only was falsified because the import error was not credential-specific and replacement order could leak an overlapping suffix;
- independent downstream filters were falsified because duplicated policy can drift and permit pre-filter leakage;
- encrypted persistence was rejected as stronger and contrary to the accepted no-persistence model; and
- typed boundary rejection plus normalized core redaction was selected.

VibeThinker challenged non-null handling, nested values, error continuation, explicit null, export leakage, and overlap behavior. Deterministic tests reject all non-null JSON/Nickel shapes, return immediately on conversion error, retain null/absent compatibility, prove export omission, and cover overlap. Its suggestions about recursively redacting nested plaintext are inapplicable because the entire `value` field is rejected rather than accepted for traversal. Advisory output is not validation authority.

## Evidence identities

- `src/credentials.rs`: `fc74f4d7c14a7079b43711680a05a32fd07a750396fb7a0ba3738b658ead839a`
- `src/engine/executor.rs`: `d50f7b97fbd39999a80163aa8e5037d80d277dd52c6741b8cab67fae1dc09664`
- `src/workflow_format.rs`: `083eccc368e65dd7f7378b20b7bdd62d6bf3dc11f712d3b5a57c99d48db4a276`
- `src/workflow_validation.rs`: `49402cc2ef8c47a1f83d778ee4606b43a639cd4cead99899239c9dc399dd6374`
- `tests/test_credentials.rs`: `3b64eb9c03ac3f912e5735826df916d6a3b043cab4d6eb10192da58db3f1921e`

## Exact reclassification

The maintained inventory records:

- total missing rows: 353
- reviewed: 12
- exact: 12
- partial: 0
- no-exact: 0
- unassessed: 341
- issues: none
- inventory `valid: true`
- missing-set BLAKE3: `0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1`
- review-policy BLAKE3: `4a5618c793b05757edd1f4bf6327c97853591b32e9310745cfd507cd443d54f2`
- inventory BLAKE3: `ef99b488a8eb41839be052b34b8a9d9e1ca7983791352ac999c0ada95e1f354f`

Coverage remains deliberately unchanged:

- requirements: 714
- referenced: 361
- missing: 353
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `9e1a9bc59d6fdb98fa41fc47c7e3e0218db086a6b95893078b2d0f861f683a9c`
- receipt BLAKE3: `30d9362c2553184dc181ccd56c7f9fc4505ce6861162c027576674019dee7c19`

## Octet migration

The first Octet comparison found six identity replacements and one resolved assertion-density finding. The retained migration script has positive and negative self-tests, requires exact stale/new ID sets, verifies semantic shape for every replacement, permits only the reviewed resolved function finding, and proves the candidate IDs are the exact allowlisted transformation:

```text
old=1163 current=1162 replacements=6 resolved=1
```

After migration, Octet retains 912 accepted/current findings, zero new, zero stale, and 1,162 total findings under baseline `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`. The resolved finding was `portable_config_to_node_credentials` assertion density; two explicit successful-path assertions now bind plaintext absence and complete reference transfer. The underlying Cargo process remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result.

## Validation

- 17 credential tests pass with positive and negative import, resolution, redaction, registry, cache, vault, command, and persistence cases.
- Workflow-format, store, and run-history focused suites pass.
- Two credential audit tests and six inventory tests pass.
- `scripts/check.rs` passes repository formatting, all tests, strict project-only Clippy, identity checks, and accepted Cairn spec checks.
- Cairn policy freshness, validation, and proposal/design/tasks gates pass.
- Deterministic inventory generation and checking agree.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.

## Non-claims

This evidence proves only the checked repository-owned import, conversion, execution-output, tracing-capture, run-detail, JSON, redb, validation, inventory, and coverage observations. It does not prove confidentiality against child processes before capture, debuggers, kernel or host observers, compromised providers/adapters, external logs, arbitrary storage engines, the remaining 14 credential rows, complete traceability, release readiness, or production readiness.
