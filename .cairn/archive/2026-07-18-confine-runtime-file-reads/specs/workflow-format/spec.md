## ADDED Requirements

### Requirement: Capability-confined Markdown prompt hydration
r[n8nclone.workflow_format.markdown_agent_prompts.capability_read] Markdown prompt hydration MUST open the workflow directory as a capability and MUST read validated relative prompt paths through that handle within the existing byte and UTF-8 bounds.

#### Scenario: Valid prompt remains reviewable
r[n8nclone.workflow_format.markdown_agent_prompts.capability_read.scenario.success]
- GIVEN an in-root Markdown prompt is valid UTF-8, non-empty, and within the byte limit
- WHEN workflow loading hydrates it
- THEN loading MUST preserve the source path and prompt body while reading only through the workflow-directory capability

#### Scenario: Replaced or escaping prompt fails closed
r[n8nclone.workflow_format.markdown_agent_prompts.capability_read.scenario.rejected]
- GIVEN a validated prompt path resolves outside the workflow directory through a symlink or replaced intermediate component
- WHEN workflow loading opens it through the workflow-directory capability
- THEN loading MUST fail with the existing node-scoped prompt diagnostic without returning outside text
