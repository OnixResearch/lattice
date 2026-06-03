## ADDED Requirements

### Requirement: File node configuration
r[n8nclone.workflow_execution.file_nodes.config] The system MUST model read, write, and append file nodes with typed executable configuration.

#### Scenario: Create read file node
r[n8nclone.workflow_execution.file_nodes.config.read]
- GIVEN a workflow author adds a read file node
- WHEN the node is stored in the workflow
- THEN the system records the path source, operation kind, and output ports as typed configuration

#### Scenario: Create write file node
r[n8nclone.workflow_execution.file_nodes.config.write]
- GIVEN a workflow author adds a write file node
- WHEN the node is stored in the workflow
- THEN the system records the path source, content input, overwrite policy, and output ports as typed configuration

### Requirement: Filesystem interface
r[n8nclone.workflow_execution.file_nodes.interface] The system MUST execute file node operations through a filesystem interface.

#### Scenario: In-memory filesystem test
r[n8nclone.workflow_execution.file_nodes.interface.memory]
- GIVEN tests provide an in-memory filesystem implementation
- WHEN a file node execution plan runs
- THEN the system uses that interface instead of direct operating-system file I/O

### Requirement: Read file node
r[n8nclone.workflow_execution.file_nodes.read] The system MUST read file contents into routable graph output values.

#### Scenario: Read file content
r[n8nclone.workflow_execution.file_nodes.read.content]
- GIVEN a read file node targets an allowed existing file
- WHEN the workflow runs
- THEN the node completes with file content and metadata on its real output port IDs

#### Scenario: Missing file
r[n8nclone.workflow_execution.file_nodes.read.missing]
- GIVEN a read file node targets a missing file
- WHEN the workflow runs
- THEN the system marks the node failed with a clear file error

### Requirement: Write and append file nodes
r[n8nclone.workflow_execution.file_nodes.write_append] The system MUST write or append input content through explicit file node operations.

#### Scenario: Write file content
r[n8nclone.workflow_execution.file_nodes.write_append.write]
- GIVEN a write file node receives content input and targets an allowed path
- WHEN the workflow runs
- THEN the system writes the content and reports bytes written

#### Scenario: Append file content
r[n8nclone.workflow_execution.file_nodes.write_append.append]
- GIVEN an append file node receives content input and targets an allowed path
- WHEN the workflow runs
- THEN the system appends the content and reports bytes written

### Requirement: File node safety
r[n8nclone.workflow_execution.file_nodes.safety] The system MUST enforce path and permission safety before executing file node side effects.

#### Scenario: Path outside allowed roots
r[n8nclone.workflow_execution.file_nodes.safety.outside_root]
- GIVEN a file node targets a path outside configured allowed roots
- WHEN workflow validation or execution planning runs
- THEN the system rejects the operation before reading or writing the file

#### Scenario: Write without overwrite permission
r[n8nclone.workflow_execution.file_nodes.safety.overwrite]
- GIVEN a write file node targets an existing file while overwrite is disabled
- WHEN the workflow runs
- THEN the system fails the node without modifying the existing file
