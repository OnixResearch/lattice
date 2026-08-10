## ADDED Requirements

### Requirement: Shared capability-backed runtime reads
r[n8nclone.workflow_execution.capability_reads] Authority-bounded runtime file readers MUST admit targets through a deterministic rooted-path core and MUST read through an open root directory capability without reopening admitted targets through ambient paths.

#### Scenario: Existing local file effects use the shared boundary
r[n8nclone.workflow_execution.capability_reads.scenario.file-effects]
- GIVEN a local file node has an explicit allowed root and safe target
- WHEN it reads, writes, or appends the target
- THEN root selection and relative-path admission MUST use the shared capability core while operation-specific behavior remains unchanged

#### Scenario: Path replacement after root acquisition
r[n8nclone.workflow_execution.capability_reads.scenario.path-swap]
- GIVEN a root capability is open and an intermediate target component is replaced by an escaping symlink
- WHEN a bounded reader opens the admitted relative target
- THEN the read MUST fail without returning outside bytes

### Requirement: Capability-confined production image input
r[n8nclone.workflow_execution.image_input_nodes.capability_read] Production local image input MUST require an explicit allowed root and MUST inspect and read one descriptor-confined file handle within the configured byte limit.

#### Scenario: Allowed image succeeds
r[n8nclone.workflow_execution.image_input_nodes.capability_read.scenario.success]
- GIVEN a supported image is beneath an explicit allowed root and within its byte limit
- WHEN production image input loads it
- THEN the runtime MUST return the existing typed image value from bytes read through the root capability

#### Scenario: Missing root or escaping image fails closed
r[n8nclone.workflow_execution.image_input_nodes.capability_read.scenario.rejected]
- GIVEN an image plan omits allowed roots, traverses, or resolves through a symlink outside its root
- WHEN production image input loads it
- THEN the runtime MUST reject the read without returning outside bytes

### Requirement: Capability-confined agent handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read] Generic-agent image and artifact handoffs MUST read bounded output bytes relative to an open command working-directory capability.

#### Scenario: Relative handoff succeeds
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read.scenario.success]
- GIVEN an agent returns a safe relative handoff path beneath its command working directory
- WHEN the adapter decodes the output
- THEN it MUST preserve typed image or artifact behavior using bytes from the descriptor-confined handle

#### Scenario: Escaping or replaced handoff fails closed
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read.scenario.rejected]
- GIVEN a handoff is outside the command working directory or an intermediate path is replaced with an escaping symlink
- WHEN the adapter opens the handoff
- THEN it MUST fail with a node-scoped diagnostic without returning outside bytes
