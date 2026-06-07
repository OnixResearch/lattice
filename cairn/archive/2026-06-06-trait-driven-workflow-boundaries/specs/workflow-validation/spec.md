## ADDED Requirements

### Requirement: Node-kind provider validation
r[n8nclone.workflow_validation.node_kind_provider] The system MUST validate node-specific configuration and runner bindings through registered node-kind and runtime providers when providers exist.

#### Scenario: Provider-backed config validation
r[n8nclone.workflow_validation.node_kind_provider.config_validation]
- GIVEN a workflow contains built-in shell, HTTP, file, image, or generic agent nodes
- WHEN workflow validation runs
- THEN the system MUST obtain node-specific validation diagnostics from the matching node-kind provider and preserve existing diagnostic severity, location, and message stability

#### Scenario: Runtime binding admission diagnostics
r[n8nclone.workflow_validation.node_kind_provider.runtime_binding]
- GIVEN a runner plan references a missing runtime binding or a runtime that does not satisfy the node-kind provider contract
- WHEN runner-admission validation runs
- THEN the system MUST report a fatal diagnostic before any process, network, filesystem, database, clock, or UI side effect occurs

#### Scenario: Provider fallback preserves safety
r[n8nclone.workflow_validation.node_kind_provider.fallback]
- GIVEN a built-in node kind has not yet migrated every validation rule to a provider
- WHEN workflow validation runs during the migration
- THEN the system MUST keep the existing validation branch active as a guarded fallback and MUST NOT silently drop fatal diagnostics
