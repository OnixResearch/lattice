## ADDED Requirements

### Requirement: Generic command agent runtimes
r[n8nclone.workflow_execution.agent_nodes.command_runtimes] The system MUST execute production generic agent nodes through configurable command runtime entries rather than hard-coded Pi or Clankers command branches.

#### Scenario: Runtime map configuration
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.config]
- GIVEN default application configuration is created or loaded
- WHEN production agent runtime settings are inspected
- THEN the system MUST expose deterministic command runtime presets for Pi and Clankers in a generic runtime map

#### Scenario: Runtime-map command planning
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.planning]
- GIVEN a generic agent node targets a configured runtime entry
- WHEN production command planning runs
- THEN the system MUST select the matching runtime entry and render configured argument placeholders without target-specific command branches

#### Scenario: Runtime configuration errors
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.errors]
- GIVEN a generic agent node targets a missing runtime entry or a runtime entry with an empty command
- WHEN production command planning runs
- THEN the system MUST reject execution before spawning a process with a clear node-scoped diagnostic

#### Scenario: Preset behavior remains compatible
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.compatibility]
- GIVEN existing Pi or Clankers generic agent nodes use default runtime presets
- WHEN workflows run through production TUI or headless execution
- THEN text, JSON, image handoff, and artifact handoff behavior MUST remain compatible with the prior production adapter contract
