# Operator Diagnostics Specification

## Purpose

Defines the `operator-diagnostics` capability.

## Requirements

### Requirement: Doctor command
r[n8nclone.operator_diagnostics.doctor_command] The system MUST provide an operator-facing doctor command that checks configuration, store, workflow, credential provider, trigger, and listener readiness without mutating workflow state.

#### Scenario: Healthy configuration succeeds
r[n8nclone.operator_diagnostics.doctor_command.healthy]
- GIVEN configuration, store, workflows, credential providers, triggers, and listener settings are healthy
- WHEN the operator runs the doctor command
- THEN the command reports a healthy summary and exits successfully without mutating workflow state

#### Scenario: Fatal readiness issue fails safely
r[n8nclone.operator_diagnostics.doctor_command.fatal]
- GIVEN one or more fatal readiness issues exist
- WHEN the operator runs the doctor command
- THEN the command reports all collected fatal findings and fails after completing non-destructive checks

### Requirement: Doctor evaluation core
r[n8nclone.operator_diagnostics.doctor_core] The system MUST evaluate collected doctor facts into deterministic findings through a pure functional core.

#### Scenario: Healthy facts produce no fatal findings
r[n8nclone.operator_diagnostics.doctor_core.healthy]
- GIVEN collected doctor facts describe a healthy configuration
- WHEN doctor evaluation runs
- THEN the report contains a healthy summary and no fatal findings

#### Scenario: Failing facts produce stable findings
r[n8nclone.operator_diagnostics.doctor_core.fatal]
- GIVEN collected doctor facts describe missing config, store failure, Nickel parse failure, credential provider failure, or listener readiness failure
- WHEN doctor evaluation runs
- THEN the report contains stable check identifiers, severities, safe messages, and remediation hints for each failing fact

### Requirement: Doctor output
r[n8nclone.operator_diagnostics.doctor_output] The system MUST render doctor reports as deterministic human-readable text and deterministic JSON.

#### Scenario: Text output is actionable
r[n8nclone.operator_diagnostics.doctor_output.text]
- GIVEN a doctor report contains healthy, warning, and fatal findings
- WHEN text output is rendered
- THEN findings are grouped or ordered deterministically and include stable check identifiers, severities, safe messages, and remediation hints

#### Scenario: JSON output is automation-safe
r[n8nclone.operator_diagnostics.doctor_output.json]
- GIVEN a doctor report contains healthy, warning, and fatal findings
- WHEN JSON output is rendered
- THEN the serialized report uses deterministic ordering and includes the same stable identifiers, severities, safe messages, and remediation hints

### Requirement: Doctor checks
r[n8nclone.operator_diagnostics.doctor_checks] The system MUST collect doctor facts for configuration, Nickel evaluation, store reachability, workflow validation, credential provider readiness, trigger readiness, and webhook listener readiness through a thin imperative shell.

#### Scenario: Config and store checks
r[n8nclone.operator_diagnostics.doctor_checks.config_store]
- GIVEN a configured Lattice environment
- WHEN doctor fact collection runs
- THEN config discovery, Nickel evaluation, store opening, schema compatibility, and workflow validation facts are collected without modifying the store

#### Scenario: Credential provider checks
r[n8nclone.operator_diagnostics.doctor_checks.credentials]
- GIVEN credential providers are configured
- WHEN doctor fact collection probes provider readiness
- THEN the collected facts describe provider reachability without resolving or printing secret values

#### Scenario: Webhook listener readiness checks
r[n8nclone.operator_diagnostics.doctor_checks.webhook_listener]
- GIVEN webhook triggers and listener configuration exist
- WHEN doctor fact collection probes listener readiness
- THEN the collected facts describe trigger availability and bind readiness without starting a durable listener or accepting external requests

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
