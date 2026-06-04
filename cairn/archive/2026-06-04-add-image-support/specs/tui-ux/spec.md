## ADDED Requirements

### Requirement: Generic agent node add flow
r[n8nclone.tui_ux.agent_nodes.add_flow] The system MUST let users create image input nodes and generic Pi or Clankers agent nodes from the TUI add-node flow.

#### Scenario: Create image input node
r[n8nclone.tui_ux.agent_nodes.add_flow.input]
- GIVEN the user starts the add-node flow
- WHEN the user chooses an image input node, enters an image label, and selects a local image path
- THEN the system creates an image input node with image output ports and marks the workflow as unsaved

#### Scenario: Create Pi generic agent node
r[n8nclone.tui_ux.agent_nodes.add_flow.pi]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a generic agent node, selects Pi as the target runtime, and enters required prompt and output fields
- THEN the system creates a generic agent node with Pi target settings, media-aware inputs, and typed outputs

#### Scenario: Create Clankers generic agent node
r[n8nclone.tui_ux.agent_nodes.add_flow.clankers]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a generic agent node, selects Clankers as the target runtime, and enters required prompt and output fields
- THEN the system creates a generic agent node with Clankers target settings, media-aware inputs, and typed outputs

#### Scenario: Cancel generic agent node creation
r[n8nclone.tui_ux.agent_nodes.add_flow.cancel]
- GIVEN image input or generic agent node creation is in progress
- WHEN the user cancels the add-node flow
- THEN the system creates no node and preserves the existing workflow

### Requirement: Generic agent node property editing
r[n8nclone.tui_ux.agent_nodes.properties] The system MUST let users edit selected image input and generic agent node properties from the properties surface.

#### Scenario: Edit generic agent prompt
r[n8nclone.tui_ux.agent_nodes.properties.prompt]
- GIVEN exactly one generic agent node is selected
- WHEN the user edits and saves the prompt field
- THEN the workflow stores the new prompt, marks the workflow as unsaved, and displays the updated value

#### Scenario: Edit generic agent target settings
r[n8nclone.tui_ux.agent_nodes.properties.target]
- GIVEN exactly one generic agent node is selected
- WHEN the user edits target runtime, model hints, tool policy, context references, output mode, credential references, or session references with valid values
- THEN the workflow stores the updated generic agent configuration and keeps the node runnable

#### Scenario: Edit image input path
r[n8nclone.tui_ux.agent_nodes.properties.image_path]
- GIVEN exactly one image input node is selected
- WHEN the user edits the local image path with a valid value
- THEN the workflow stores the updated image input configuration and keeps the node routable to generic agent inputs

#### Scenario: Reject invalid generic agent property
r[n8nclone.tui_ux.agent_nodes.properties.invalid]
- GIVEN exactly one image input or generic agent node is selected
- WHEN the user enters an invalid local path, unsupported output mode, unsupported target runtime, or missing required prompt field
- THEN the system rejects the edit, preserves the previous value, and keeps the edit flow recoverable

### Requirement: Image previews
r[n8nclone.tui_ux.image_nodes.preview] The system SHOULD render bounded image previews or metadata fallbacks for image nodes and generic agent image outputs.

#### Scenario: Canvas shows image thumbnail or fallback
r[n8nclone.tui_ux.image_nodes.preview.canvas]
- GIVEN an image input node or generic agent node has a loaded or generated image value
- WHEN the canvas renders that node
- THEN the system shows a bounded image thumbnail when supported or a compact metadata summary when thumbnail rendering is unavailable

#### Scenario: Output panel shows generated image details
r[n8nclone.tui_ux.image_nodes.preview.output_panel]
- GIVEN the selected run output contains a generated image value from a generic agent node
- WHEN the output panel renders
- THEN the system shows media type, dimensions when available, content hash summary, adapter metadata, and a bounded preview or fallback

#### Scenario: Preview never dumps raw bytes
r[n8nclone.tui_ux.image_nodes.preview.no_raw_bytes]
- GIVEN an image value contains binary content
- WHEN any TUI surface renders the value
- THEN the system does not dump raw image bytes into the terminal

### Requirement: Generic agent error feedback
r[n8nclone.tui_ux.agent_nodes.errors] The system MUST surface image load, validation, preflight, and Pi or Clankers adapter errors without breaking TUI navigation.

#### Scenario: Missing image path feedback
r[n8nclone.tui_ux.agent_nodes.errors.missing_path]
- GIVEN an image input node references a missing local path
- WHEN the workflow validates or runs from the TUI
- THEN the system displays a clear node-scoped error and keeps the canvas interactive

#### Scenario: Unsupported image format feedback
r[n8nclone.tui_ux.agent_nodes.errors.unsupported_format]
- GIVEN an image input node references an unsupported image format
- WHEN validation reports the format error
- THEN the system displays the diagnostic near the node or output surface and keeps editing recoverable

#### Scenario: Invalid generic agent target feedback
r[n8nclone.tui_ux.agent_nodes.errors.invalid_target]
- GIVEN a generic agent node declares an unsupported target runtime
- WHEN validation reports the target error
- THEN the system displays the diagnostic near the node or properties surface and keeps editing recoverable

#### Scenario: Pi or Clankers adapter error feedback
r[n8nclone.tui_ux.agent_nodes.errors.adapter]
- GIVEN a Pi or Clankers adapter fails while running a generic agent node
- WHEN the run state updates in the TUI
- THEN the system marks the node failed, shows the adapter error summary, and continues handling input
