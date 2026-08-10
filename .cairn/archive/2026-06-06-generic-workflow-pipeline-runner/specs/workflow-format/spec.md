## ADDED Requirements

### Requirement: Generic pipeline Nickel IR
r[n8nclone.workflow_format.generic_pipeline_ir] The system MUST represent generic runner metadata in typed Nickel workflow import/export without losing existing workflow meaning.

#### Scenario: Export runtime bindings
r[n8nclone.workflow_format.generic_pipeline_ir.nickel]
- GIVEN a workflow contains typed nodes with runtime bindings, effect policies, cache policy, retry policy, artifact-reference policy, ports, and edges
- WHEN the operator exports the workflow to Nickel
- THEN the system MUST write a typed Nickel workflow file that preserves those generic pipeline fields alongside existing node configuration

#### Scenario: Import runtime bindings
r[n8nclone.workflow_format.generic_pipeline_ir.import]
- GIVEN a typed Nickel workflow file declares valid runtime bindings and runner policy fields
- WHEN the operator imports it
- THEN the system MUST validate and save a workflow whose runner plan has equivalent typed topology, runtime bindings, and policy metadata

#### Scenario: Backward-compatible round trip
r[n8nclone.workflow_format.generic_pipeline_ir.compatibility]
- GIVEN an existing Nickel workflow omits generic runner fields and uses current shell, HTTP, file, image, or generic agent node configuration
- WHEN the workflow is imported, exported, and run
- THEN the system MUST preserve existing workflow meaning by applying deterministic default runtime bindings and policies
