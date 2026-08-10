## Phase 1

- [x] [serial] r[n8nclone.workflow_execution.expression_mapping.syntax] Define expression syntax, AST, and parser with invalid-syntax diagnostics.
- [x] [serial] r[n8nclone.workflow_execution.expression_mapping.evaluate] Evaluate expressions against completed upstream values before node execution.
- [x] [parallel] r[n8nclone.workflow_execution.expression_mapping.config] Apply expression resolution to command, environment, HTTP, and file node configuration fields.
- [x] [parallel] r[n8nclone.workflow_execution.expression_mapping.errors] Add negative tests for missing references, type mismatches, and cyclic/early references.
- [x] [parallel] r[n8nclone.workflow_execution.expression_mapping.redaction] Preserve sensitivity metadata and redaction through expression results.
