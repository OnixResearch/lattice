## ADDED Requirements

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
