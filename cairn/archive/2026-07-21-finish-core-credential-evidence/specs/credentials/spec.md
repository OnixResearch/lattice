## MODIFIED Requirements

### Requirement: Secret reference model
r[n8nclone.credentials.secret_refs.model] The system MUST represent workflow credentials as opaque secret references instead of plaintext secret values and MUST reject plaintext credential fields through a stable redaction-safe credential diagnostic before store access.

#### Scenario: Store secret reference
r[n8nclone.credentials.secret_refs.model.store_ref]
- GIVEN a node resolves an execution-only credential and the workflow is subsequently saved or exported
- WHEN repository-owned store and Nickel export boundaries serialize the workflow
- THEN they store only provider, key, optional field, and binding identities and do not serialize the resolved plaintext value

#### Scenario: Reject inline secret field
r[n8nclone.credentials.secret_refs.model.reject_plaintext]
- GIVEN an imported workflow provides plaintext secret material in a credential field
- WHEN workflow import runs
- THEN the system returns a stable credential diagnostic, omits the plaintext value from diagnostics, and does not create or mutate the workflow store

### Requirement: Secret redaction
r[n8nclone.credentials.redaction] The system MUST redact resolved secret material before user-visible output, tracing logs, run history, JSON summaries, or persisted run-detail surfaces can observe it.

#### Scenario: Redact run output
r[n8nclone.credentials.redaction.output]
- GIVEN a node emits a resolved secret or overlapping secret fragment through stdout and stderr
- WHEN execution output, tracing logs, run history, JSON summaries, and persisted run details are observed
- THEN every surface MUST omit the plaintext values, output/history/JSON/persisted details MUST preserve explicit redaction markers, and tracing logs MUST retain safe non-secret node context

#### Scenario: Redact validation message
r[n8nclone.credentials.redaction.validation]
- GIVEN a credential diagnostic mentions a secret reference and a separate plaintext value exists
- WHEN text and JSON diagnostics are rendered
- THEN both renderings MUST preserve the reference identity and MUST omit the plaintext value
