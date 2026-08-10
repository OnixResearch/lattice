# Design

The migration retains the existing semantic key over identity status, lint, group, severity, file, message, and crate. It additionally normalizes only dynamic line-count text for assertion-density and function-length diagnostics while retaining the function identity.

Candidate-only semantic keys are always fatal. Old-only keys are accepted only when exactly one normalized key identifies the resolved assertion-density finding for `workflow_diagnostic_from_runner` in `src/workflow_validation.rs`. Positive tests cover stable identity, dynamic-count normalization, and the exact resolved allowlist; negative tests reject arbitrary drift and unapproved removals.
