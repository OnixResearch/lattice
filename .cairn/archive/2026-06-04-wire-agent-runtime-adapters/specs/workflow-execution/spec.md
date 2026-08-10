## ADDED Requirements

### Requirement: Production agent runtime adapters
r[n8nclone.workflow_execution.agent_nodes.production_adapters] The system MUST run generic agent nodes through production Pi and Clankers runtime adapters in default TUI and headless execution while preserving injectable adapter seams for tests.

#### Scenario: Production runtime adapter configuration
r[n8nclone.workflow_execution.agent_nodes.production_adapters.config]
- GIVEN default runtime configuration is loaded
- WHEN a generic agent node targets Pi or Clankers
- THEN the system SHOULD have deterministic command defaults and MAY accept explicit command overrides for each target

#### Scenario: Target-specific production routing
r[n8nclone.workflow_execution.agent_nodes.production_adapters.route]
- GIVEN a generic agent node targets Pi or Clankers
- WHEN production execution prepares the adapter request
- THEN the system MUST route the request only to the matching runtime adapter and MUST NOT silently retry through another target

#### Scenario: Command adapter text and JSON execution
r[n8nclone.workflow_execution.agent_nodes.production_adapters.command]
- GIVEN a generic agent node requests text or JSON output through a configured production command
- WHEN the command exits successfully with compatible stdout
- THEN the system MUST map stdout into the requested typed agent output and include safe adapter metadata

#### Scenario: Production adapter errors remain node-scoped
r[n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
- GIVEN a production agent command is missing, exits non-zero, returns invalid JSON, or receives an unsupported image or artifact output request
- WHEN the workflow runs that generic agent node
- THEN the system MUST fail only that node with a clear adapter diagnostic and MUST NOT corrupt saved workflow state

#### Scenario: Default executor uses production agent routing
r[n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
- GIVEN a workflow contains a Pi-backed or Clankers-backed generic agent node
- WHEN the workflow runs through `Executor::new()`
- THEN the default executor MUST use production agent routing instead of the placeholder no-op adapter
