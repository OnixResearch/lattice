## ADDED Requirements

### Requirement: Node-kind provider format handlers
r[n8nclone.workflow_format.node_kind_provider] The system MUST preserve typed Nickel workflow import and export semantics while moving built-in node configuration mapping behind node-kind format handlers.

#### Scenario: Provider-backed export
r[n8nclone.workflow_format.node_kind_provider.export]
- GIVEN a saved workflow contains built-in shell, HTTP, file, image, or generic agent nodes
- WHEN the operator exports the workflow to Nickel
- THEN the system MUST use registered node-kind format handlers where available and produce Nickel output equivalent to the existing format contract

#### Scenario: Provider-backed import
r[n8nclone.workflow_format.node_kind_provider.import]
- GIVEN a Nickel workflow file declares built-in shell, HTTP, file, image, or generic agent nodes
- WHEN the operator imports the workflow
- THEN the system MUST use registered node-kind format handlers where available and produce equivalent persisted `NodeConfig` data

#### Scenario: Malformed node config remains rejected
r[n8nclone.workflow_format.node_kind_provider.handlers]
- GIVEN a Nickel workflow node has missing, unknown, or malformed fields for its node kind
- WHEN provider-backed import validation runs
- THEN the system MUST reject the workflow with deterministic diagnostics and preserve existing saved workflows
