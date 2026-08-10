## MODIFIED Requirements

### Requirement: Doctor redaction
r[n8nclone.operator_diagnostics.doctor_redaction] The system MUST omit plaintext secrets, authorization headers, raw request payloads, credential provider outputs, and environment variable values from doctor reports, MUST sanitize every output-bearing identifier, message, and remediation field before rendering, and MUST omit the redaction source list from rendered output.

#### Scenario: Text redaction
r[n8nclone.operator_diagnostics.doctor_redaction.text]
- GIVEN doctor facts include secret-like provider, environment, header, payload, identifier, message, or remediation values
- WHEN the facts are evaluated and text output is rendered
- THEN those values MUST be absent from identifiers, messages, and remediation text and only safe identifiers or redacted markers MUST appear

#### Scenario: JSON redaction
r[n8nclone.operator_diagnostics.doctor_redaction.json]
- GIVEN doctor facts include secret-like provider, environment, header, payload, identifier, message, or remediation values
- WHEN the facts are evaluated and JSON output is rendered
- THEN those values and the redaction source list MUST be absent and only safe identifiers or redacted markers MUST appear
