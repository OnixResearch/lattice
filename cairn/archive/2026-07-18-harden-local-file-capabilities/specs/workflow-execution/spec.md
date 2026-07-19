## ADDED Requirements

### Requirement: Descriptor-confined local file effects
r[n8nclone.workflow_execution.file_nodes.local_capabilities] Production local file effects MUST derive a safe relative target from an explicit allowed root and MUST execute through an open directory capability without reopening the target through the ambient filesystem namespace.

#### Scenario: Nested local operations remain functional
r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.nested-io]
- GIVEN a regular nested file path is contained beneath a configured allowed root
- WHEN `LocalFs` reads, writes, or appends that target
- THEN each operation MUST use the root directory capability and MUST preserve the existing content and byte-count contracts

#### Scenario: Missing authority fails closed
r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.missing-root]
- GIVEN a production local file plan has no allowed roots
- WHEN `LocalFs` admits the operation
- THEN it MUST reject the plan before reading, creating, truncating, or appending any host file

#### Scenario: Lexical traversal fails closed
r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.traversal]
- GIVEN a target is lexically prefixed by an allowed root but its relative components contain parent traversal
- WHEN `LocalFs` admits the operation
- THEN it MUST reject the plan before opening the target

#### Scenario: Symlink escape fails closed
r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.symlink-escape]
- GIVEN a target or intermediate component beneath an allowed root is a symlink that resolves outside that root
- WHEN `LocalFs` attempts a read, write, or append
- THEN capability resolution MUST fail and MUST leave the outside target unchanged

#### Scenario: Concurrent path replacement remains confined
r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.path-swap]
- GIVEN the allowed root has been opened and an intermediate path is replaced with an escaping symlink before the target open
- WHEN the descriptor-relative operation resolves the admitted relative path
- THEN it MUST fail without reading or modifying the outside target
