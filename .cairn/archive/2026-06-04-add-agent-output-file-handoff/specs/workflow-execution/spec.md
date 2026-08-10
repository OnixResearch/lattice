## ADDED Requirements

### Requirement: Agent output file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff] The system MUST support bounded local file handoff for production generic agent image and artifact outputs while preserving text and JSON stdout behavior.

#### Scenario: Handoff envelope contract
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
- GIVEN a production generic agent command runs in image or artifact output mode
- WHEN it exits successfully
- THEN stdout MUST be parsed as a JSON envelope with a required local file path and optional media type and metadata

#### Scenario: Image file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
- GIVEN an image-mode production generic agent command returns a valid handoff envelope for a PNG or JPEG file
- WHEN the adapter decodes the command result
- THEN the system MUST return a typed image value with sniffed media type, stable content hash, source metadata, and adapter metadata

#### Scenario: Artifact file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact]
- GIVEN an artifact-mode production generic agent command returns a valid handoff envelope for a local file
- WHEN the adapter decodes the command result
- THEN the system MUST return the file bytes as the typed artifact output and include safe adapter metadata

#### Scenario: Handoff safety failures are node-scoped
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
- GIVEN a handoff envelope is invalid, missing its file, outside the command working directory, oversized, has mismatched media type, or references unsupported image bytes
- WHEN the workflow runs that generic agent node
- THEN the system MUST fail only that node with a clear adapter diagnostic and MUST NOT corrupt saved workflow state

#### Scenario: Default runtime maps image handoff to workflow ports
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.default_runtime]
- GIVEN a saved workflow contains an image-mode generic agent node using a configured production command
- WHEN the workflow runs through the default headless runtime path
- THEN the image file handoff MUST be mapped to the node's real image output port and cached as a typed image value
