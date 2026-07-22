# Design: promote node-template prompt evidence

## Success contract

The baseline missing set MUST contain 327 IDs and the promoted result MUST contain 324. The removed set MUST equal exactly:

- `n8nclone.node_templates.parameters.prompt`
- `n8nclone.node_templates.parameters.prompt.cancel`
- `n8nclone.node_templates.parameters.prompt.required`

No ID may be added, no dangling marker may remain, and coverage MUST remain `valid: false`.

## Local binding

The parent collection marker attaches directly to `add_node_flow_prompts_and_renders_template_parameters`. The cancellation marker attaches directly to `cancel_template_parameter_entry_preserves_workflow`. The required-prompt marker attaches directly to `required_template_parameter_prompt_is_visible_before_node_creation`.

Markers MUST NOT be file-scoped, duplicated, attached to helper functions, or inferred from implementation names. Promoted rows MUST leave the review policy.

## Historical identity

Completion evidence remains pinned. Historical source-byte checks remove exactly one expected marker-only line from each promoted test source before hashing; all other byte drift remains detectable.

## Validation

A deterministic promotion core validates sorted unique missing sets, exact allowlisted removal, no additions, zero dangling markers, unsuccessful coverage, assertion-local bindings, and policy exclusion. Positive and adversarial negative tests remain together.

## Non-claims

Promotion establishes only Tracey marker coverage for the accepted clauses. It does not prove arbitrary TUI correctness, complete traceability, release readiness, or production readiness.
