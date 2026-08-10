## Phase 1: Doctor core and output contract

- [x] [serial] r[n8nclone.operator_diagnostics.doctor_core] Add pure doctor fact, finding, severity, summary, and report types with deterministic evaluation.
- [x] [parallel] r[n8nclone.operator_diagnostics.doctor_core.healthy] Add positive core tests for healthy facts and warning-only facts.
- [x] [parallel] r[n8nclone.operator_diagnostics.doctor_core.fatal] Add negative core tests for missing config, store failure, Nickel parse failure, credential provider failure, and listener readiness failure.
- [x] [serial] r[n8nclone.operator_diagnostics.doctor_output] Add deterministic text and JSON report presenters.

## Phase 2: CLI shell and checks

- [x] [serial] r[n8nclone.operator_diagnostics.doctor_command] Add the `doctor` CLI command as a thin shell that collects facts without mutating workflow state.
- [x] [parallel] r[n8nclone.operator_diagnostics.doctor_checks.config_store] Wire config, Nickel, store, and workflow validation checks.
- [x] [parallel] r[n8nclone.operator_diagnostics.doctor_checks.credentials] Wire credential provider readiness checks without resolving or printing secret values.
- [x] [parallel] r[n8nclone.operator_diagnostics.doctor_checks.webhook_listener] Wire trigger and webhook listener readiness checks without starting durable listeners.

## Phase 3: Redaction and gates

- [x] [serial] r[n8nclone.operator_diagnostics.doctor_redaction] Add positive and negative redaction tests for text and JSON output.
- [x] [serial] r[n8nclone.operator_diagnostics.doctor_command] Run focused doctor tests, `./scripts/check.rs`, Cairn validation/gates, sync accepted specs, and archive or report blockers.
