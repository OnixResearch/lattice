## ADDED Requirements

### Requirement: Production image input I/O
r[n8nclone.workflow_execution.image_input_nodes.production_io] The system MUST run image input nodes through production local image I/O in default TUI and headless execution while preserving injectable image I/O seams for tests.

#### Scenario: Load local PNG or JPEG through production I/O
r[n8nclone.workflow_execution.image_input_nodes.production_io.local]
- GIVEN an image input node references an allowed local PNG or JPEG file
- WHEN the node executes through production image I/O
- THEN the node completes with a typed image value, source metadata, media type, and stable content hash under its real output port IDs

#### Scenario: Default executor uses local image I/O
r[n8nclone.workflow_execution.image_input_nodes.production_io.default_executor]
- GIVEN a workflow contains an image input node that references a valid local image file
- WHEN the workflow runs through `Executor::new()`
- THEN the default executor loads the file through local image I/O instead of failing with an unconfigured-image-I/O diagnostic

#### Scenario: Headless image workflow can load local file
r[n8nclone.workflow_execution.image_input_nodes.production_io.headless]
- GIVEN a saved or imported workflow contains an image input node referencing a valid local image file
- WHEN the operator runs the workflow through the headless command path
- THEN the run reaches the image node with production image I/O enabled and records the typed image output

#### Scenario: TUI image workflow can load local file
r[n8nclone.workflow_execution.image_input_nodes.production_io.tui]
- GIVEN a user creates an image input node in the TUI and configures a valid local image file
- WHEN the user starts a workflow run from the TUI
- THEN the background executor loads the image file and reports normal node completion instead of a no-op image I/O failure

#### Scenario: Production image read errors are node-scoped
r[n8nclone.workflow_execution.image_input_nodes.production_io.errors]
- GIVEN a production image input node references a missing, unreadable, unsupported, oversized, or outside-root image path
- WHEN the workflow runs
- THEN the system fails only that node with a clear image diagnostic and does not invoke downstream nodes that require its image output

#### Scenario: Static workflow operations do not read image bytes
r[n8nclone.workflow_execution.image_input_nodes.production_io.no_import_io]
- GIVEN a workflow is imported, exported, or statically validated with image input node configuration
- WHEN no workflow execution is requested
- THEN the system preserves image configuration and diagnostics without reading the referenced image file bytes
