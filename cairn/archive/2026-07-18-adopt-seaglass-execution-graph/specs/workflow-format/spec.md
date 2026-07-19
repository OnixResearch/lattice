## ADDED Requirements

### Requirement: Markdown agent prompt sources
r[n8nclone.workflow_format.markdown_agent_prompts] Typed Nickel workflows MAY reference role prompts authored as bounded UTF-8 Markdown files under the workflow root.

#### Scenario: Valid Markdown prompt loads
r[n8nclone.workflow_format.markdown_agent_prompts.scenario.valid]
- GIVEN an agent config declares one relative `.md` prompt path contained under the workflow root
- WHEN the workflow file is loaded
- THEN the shell MUST read bounded UTF-8 text, preserve the source path for export, and provide the prompt body to the existing typed agent runtime contract

#### Scenario: Unsafe or malformed prompt source fails closed
r[n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid]
- GIVEN an agent prompt source is empty, duplicated, absolute, traverses outside the workflow root, escapes through a symlink, uses a non-Markdown extension, is missing, exceeds the byte limit, or is invalid UTF-8
- WHEN workflow loading validates the source
- THEN loading MUST fail with a node-scoped diagnostic before graph execution or agent process invocation
