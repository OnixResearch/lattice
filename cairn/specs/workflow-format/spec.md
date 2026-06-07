# Workflow Format Specification

## Purpose

Defines the `workflow-format` capability.

## Requirements

### Requirement: Nickel workflow export
r[n8nclone.workflow_format.nickel.export] The system MUST export saved workflows as typed Nickel workflow files.

#### Scenario: Export saved workflow
r[n8nclone.workflow_format.nickel.export.saved]
- GIVEN a workflow exists in the configured store
- WHEN the operator exports it to Nickel
- THEN the system writes a Nickel file containing workflow metadata, graph topology, ports, and node configuration

#### Scenario: Export missing workflow
r[n8nclone.workflow_format.nickel.export.missing]
- GIVEN no saved workflow matches the requested name
- WHEN the operator requests export
- THEN the system reports a clear error and writes no workflow file

### Requirement: Nickel workflow import
r[n8nclone.workflow_format.nickel.import] The system MUST import typed Nickel workflow files only after successful validation.

#### Scenario: Import valid workflow
r[n8nclone.workflow_format.nickel.import.valid]
- GIVEN a Nickel workflow file satisfies the workflow contract
- WHEN the operator imports it
- THEN the system saves the resulting workflow and makes it available to load or run

#### Scenario: Reject invalid workflow
r[n8nclone.workflow_format.nickel.import.invalid]
- GIVEN a Nickel workflow file is malformed or violates the workflow contract
- WHEN the operator imports it
- THEN the system rejects the import and preserves existing saved workflows

### Requirement: Nickel workflow round trip
r[n8nclone.workflow_format.nickel.round_trip] The system SHOULD preserve workflow meaning across Nickel export and import.

#### Scenario: Export then import preserves workflow
r[n8nclone.workflow_format.nickel.round_trip.preserve]
- GIVEN a saved workflow contains nodes, edges, ports, metadata, and shell configuration
- WHEN the workflow is exported to Nickel and imported again
- THEN the imported workflow has equivalent runnable topology and configuration

### Requirement: Complex workflow examples
r[n8nclone.workflow_examples.complex] The system MUST keep complex Nickel workflow examples importable and runnable.

#### Scenario: Run complex workflow examples headlessly
r[n8nclone.workflow_examples.headless]
- GIVEN complex Nickel workflow examples exist in the repository
- WHEN each example is imported and run headlessly
- THEN the system records successful runs with every node executed

### Requirement: Invalid workflow fixtures
r[n8nclone.workflow_format.nickel.invalid] The system MUST cover invalid Nickel workflow inputs with deterministic diagnostics.

#### Scenario: Missing required field
r[n8nclone.workflow_format.nickel.invalid.missing_field]
- GIVEN a Nickel workflow file omits a required workflow field
- WHEN validation runs
- THEN the system reports the missing field and refuses to save the workflow

### Requirement: Nickel generic agent node import and export
r[n8nclone.workflow_format.agent_nodes.nickel] The system MUST preserve image input and generic agent nodes in typed Nickel workflow import and export.

#### Scenario: Export generic agent workflow
r[n8nclone.workflow_format.agent_nodes.nickel.export]
- GIVEN a saved workflow contains image input nodes, generic agent nodes, image ports, prompt sources, Pi or Clankers target settings, tool policy, context references, and output modes
- WHEN the operator exports it to Nickel
- THEN the system writes a Nickel workflow file that contains the same image and generic agent topology and configuration fields

#### Scenario: Import generic agent workflow
r[n8nclone.workflow_format.agent_nodes.nickel.import]
- GIVEN a Nickel workflow file declares valid image input nodes and generic agent nodes targeting Pi or Clankers
- WHEN the operator imports it
- THEN the system saves a workflow with equivalent image ports, generic agent node configuration, and runnable topology

#### Scenario: Round trip preserves generic agent workflow meaning
r[n8nclone.workflow_format.agent_nodes.nickel.round_trip]
- GIVEN a saved workflow routes uploaded and inspiration images into a generic agent node with a prompt source and image output mode
- WHEN the workflow is exported to Nickel and imported again
- THEN the imported workflow preserves the image routes, prompt source, target runtime, model hints, tool policy, context references, and output mode

### Requirement: Invalid generic agent workflow fixtures
r[n8nclone.workflow_format.agent_nodes.invalid] The system MUST reject malformed image and generic agent workflow inputs with deterministic diagnostics.

#### Scenario: Missing generic agent prompt field
r[n8nclone.workflow_format.agent_nodes.invalid.missing_prompt]
- GIVEN a Nickel workflow file omits a required generic agent prompt field
- WHEN validation runs during import
- THEN the system reports the missing field and refuses to save the workflow

#### Scenario: Unsupported generic agent target
r[n8nclone.workflow_format.agent_nodes.invalid.target]
- GIVEN a Nickel workflow file declares a generic agent node target other than Pi or Clankers
- WHEN validation runs during import
- THEN the system reports the unsupported target with the affected node identity

#### Scenario: Invalid image port type
r[n8nclone.workflow_format.agent_nodes.invalid.port_type]
- GIVEN a Nickel workflow file wires an image output to an incompatible non-image input
- WHEN validation runs during import
- THEN the system reports the incompatible port type and preserves existing saved workflows

#### Scenario: Unsupported image input format
r[n8nclone.workflow_format.agent_nodes.invalid.image_format]
- GIVEN a Nickel workflow file declares an image input format outside the supported media-type set
- WHEN validation runs during import
- THEN the system reports the unsupported image format with the affected node identity

### Requirement: Image workflow examples
r[n8nclone.workflow_examples.image] The system MUST keep complex image workflow examples importable and runnable headlessly with deterministic fake Pi and Clankers adapters.

#### Scenario: Run selfie-to-fine-art example headlessly
r[n8nclone.workflow_examples.image.selfie_to_fine_art]
- GIVEN a complex Nickel workflow example loads a selfie image, loads inspiration images, and invokes a generic agent node through a fake Pi or Clankers adapter
- WHEN the example is imported and run headlessly
- THEN the system records a successful run with the generated image output available for downstream nodes

#### Scenario: Invalid image example remains rejected
r[n8nclone.workflow_examples.image.invalid]
- GIVEN an image workflow example fixture intentionally references an unsupported image format
- WHEN the fixture is imported or run in negative tests
- THEN the system reports the expected diagnostic and does not create partial workflow state

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
