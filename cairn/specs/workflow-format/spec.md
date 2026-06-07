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

### Requirement: n8n JSON import
r[n8nclone.workflow_format.n8n_json.import] The system MUST import a documented subset of n8n workflow JSON into typed n8nclone workflows with deterministic mapping reports.

#### Scenario: Import supported n8n workflow
r[n8nclone.workflow_format.n8n_json.import.supported]
- GIVEN an n8n JSON workflow uses only supported node kinds and connection shapes
- WHEN the operator imports it
- THEN the system creates an equivalent n8nclone workflow and writes a mapping report for every source node

#### Scenario: Import malformed n8n JSON fails
r[n8nclone.workflow_format.n8n_json.import.malformed]
- GIVEN an n8n JSON file is syntactically invalid or missing required workflow fields
- WHEN import runs
- THEN the system rejects the file with deterministic diagnostics and preserves existing saved workflows

### Requirement: n8n JSON import CLI
r[n8nclone.workflow_format.n8n_json.import_cli] The system SHOULD expose n8n JSON import through an operator-facing command that separates workflow creation from mapping evidence.

#### Scenario: Import writes mapping report
r[n8nclone.workflow_format.n8n_json.import_cli.report]
- GIVEN an n8n JSON import succeeds or fails with supported diagnostics
- WHEN the operator requests a mapping report
- THEN the system writes stable report data containing source ids, target ids when created, statuses, diagnostics, and unsupported fields

### Requirement: n8n JSON fixtures
r[n8nclone.workflow_format.n8n_json.fixtures] The system MUST keep positive and negative n8n JSON interoperability fixtures in the repository.

#### Scenario: Supported fixtures import
r[n8nclone.workflow_format.n8n_json.fixtures.supported]
- GIVEN supported n8n JSON fixtures exist for shell and HTTP workflows
- WHEN the fixture tests run
- THEN each fixture imports into an equivalent typed n8nclone workflow

#### Scenario: Unsupported fixtures fail predictably
r[n8nclone.workflow_format.n8n_json.fixtures.unsupported]
- GIVEN negative n8n JSON fixtures exist for unsupported nodes, expressions, malformed connections, and credential data
- WHEN the fixture tests run
- THEN each fixture reports the expected deterministic diagnostic

### Requirement: n8n JSON export
r[n8nclone.workflow_format.n8n_json.export] The system MAY export supported n8nclone workflow subsets as deterministic n8n JSON.

#### Scenario: Export supported subset
r[n8nclone.workflow_format.n8n_json.export.supported]
- GIVEN an n8nclone workflow contains only node kinds and settings with known n8n mappings
- WHEN the operator exports it as n8n JSON
- THEN the system writes deterministic n8n JSON and a mapping report describing exported semantics

#### Scenario: Export unsupported subset is blocked
r[n8nclone.workflow_format.n8n_json.export.unsupported]
- GIVEN an n8nclone workflow contains nodes or settings without known n8n mappings
- WHEN the operator exports it as n8n JSON
- THEN the system reports unsupported features and does not write misleading workflow JSON unless explicit best-effort export is requested

### Requirement: Replay bundle format
r[n8nclone.workflow_format.replay_bundles] The system MUST export and import redaction-safe replay bundles for workflow run debugging.

#### Scenario: Export replay bundle
r[n8nclone.workflow_format.replay_bundles.export]
- GIVEN a persisted run has runner events and safe summaries
- WHEN the operator exports a replay bundle
- THEN the system writes run metadata, workflow snapshot or workflow hash, ordered events, redaction metadata, and artifact reference metadata without plaintext secrets

#### Scenario: Import replay bundle
r[n8nclone.workflow_format.replay_bundles.import]
- GIVEN a replay bundle satisfies the replay bundle contract
- WHEN the operator imports it
- THEN the system stores or opens the bundle as a replayable historical run without executing workflow side effects

#### Scenario: Reject unsafe replay bundle
r[n8nclone.workflow_format.replay_bundles.reject_unsafe]
- GIVEN a replay bundle includes malformed events or fields marked as plaintext secret material
- WHEN import validation runs
- THEN the system rejects the bundle with deterministic diagnostics

### Requirement: Credential provider format
r[n8nclone.workflow_format.credentials.providers] The system MUST import and export credential provider references without plaintext secret material.

#### Scenario: Export provider references
r[n8nclone.workflow_format.credentials.providers.export]
- GIVEN a workflow contains credential references targeting configured providers
- WHEN the workflow is exported
- THEN the system writes provider id, key identity, and optional field identity without resolved secret values

#### Scenario: Import provider references
r[n8nclone.workflow_format.credentials.providers.import]
- GIVEN a workflow file declares credential provider references with valid shapes
- WHEN the workflow is imported
- THEN the system saves the references as opaque credential identities and does not require resolving plaintext secrets during import

#### Scenario: Reject plaintext provider payload
r[n8nclone.workflow_format.credentials.providers.reject_plaintext]
- GIVEN a workflow file attempts to include local vault payloads or resolved secret values inline
- WHEN import validation runs
- THEN the system rejects the file and preserves existing saved workflows

### Requirement: Control-flow Nickel format
r[n8nclone.workflow_format.control_flow] The system MUST preserve condition, merge, and subworkflow node configuration in typed Nickel workflow import and export.

#### Scenario: Export control-flow workflow
r[n8nclone.workflow_format.control_flow.export]
- GIVEN a saved workflow contains condition, merge, or subworkflow nodes
- WHEN the operator exports it to Nickel
- THEN the system writes predicate definitions, branch labels, merge policies, subworkflow references, and interface mappings deterministically

#### Scenario: Import control-flow workflow
r[n8nclone.workflow_format.control_flow.import]
- GIVEN a Nickel workflow file declares valid condition, merge, and subworkflow nodes
- WHEN the operator imports it
- THEN the system saves a workflow with equivalent typed topology and node configuration

### Requirement: Control-flow format compatibility
r[n8nclone.workflow_format.control_flow.compatibility] The system SHOULD keep existing Nickel workflows valid when control-flow fields are absent.

#### Scenario: Existing workflow round trip remains stable
r[n8nclone.workflow_format.control_flow.compatibility.existing]
- GIVEN a Nickel workflow file contains only existing shell, HTTP, file, image, or agent nodes
- WHEN the workflow is imported and exported after control-flow support is added
- THEN the system preserves the original runnable meaning without requiring new control-flow fields

#### Scenario: Malformed control-flow input is rejected
r[n8nclone.workflow_format.control_flow.compatibility.invalid]
- GIVEN a Nickel workflow file declares malformed control-flow configuration
- WHEN import validation runs
- THEN the system rejects the workflow and preserves existing saved workflows
