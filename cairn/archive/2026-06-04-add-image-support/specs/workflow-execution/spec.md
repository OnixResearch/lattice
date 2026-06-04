## ADDED Requirements

### Requirement: Image value model
r[n8nclone.workflow_execution.image_values.model] The system MUST represent image payloads as first-class typed values with binary content, media type, stable content hash, and optional metadata.

#### Scenario: Store decoded image value
r[n8nclone.workflow_execution.image_values.model.store]
- GIVEN an image payload has valid bytes, a supported media type, and decoded metadata
- WHEN the workflow value is created
- THEN the system stores the bytes, media type, metadata, and a deterministic content hash without converting the image to text

#### Scenario: Route image ports
r[n8nclone.workflow_execution.image_values.model.route]
- GIVEN an upstream node exposes an image output port and a downstream node exposes an image input port
- WHEN the workflow validates or runs
- THEN the image value is considered compatible and routable between those ports

#### Scenario: Reject incompatible image wiring
r[n8nclone.workflow_execution.image_values.model.incompatible]
- GIVEN an upstream node exposes an image output port and a downstream node requires a string input port
- WHEN workflow validation checks the edge
- THEN the system reports a type compatibility diagnostic before execution starts

### Requirement: Image input node loading
r[n8nclone.workflow_execution.image_input_nodes.load] The system MUST model image input nodes that load and validate images through explicit image I/O and decoder interfaces.

#### Scenario: Load valid local image
r[n8nclone.workflow_execution.image_input_nodes.load.local]
- GIVEN an image input node targets an allowed local PNG or JPEG file
- WHEN the workflow runs
- THEN the node completes with an image value and image metadata on its real output port IDs

#### Scenario: Missing image file
r[n8nclone.workflow_execution.image_input_nodes.load.missing]
- GIVEN an image input node targets a missing local file
- WHEN the workflow runs
- THEN the system marks the node failed with a clear missing-image diagnostic

#### Scenario: Unsupported image format
r[n8nclone.workflow_execution.image_input_nodes.load.unsupported]
- GIVEN an image input node receives bytes that do not match a supported image media type
- WHEN validation or execution planning inspects the payload
- THEN the system rejects the image before routing it to downstream nodes

#### Scenario: Oversized image input
r[n8nclone.workflow_execution.image_input_nodes.load.oversized]
- GIVEN an image input node receives bytes larger than the configured image byte budget
- WHEN validation or execution planning inspects the payload
- THEN the system rejects the image without invoking downstream adapters

### Requirement: Generic agent node configuration
r[n8nclone.workflow_execution.agent_nodes.config] The system MUST model all prompt, AI, or LLM-style workflow steps as generic agent nodes with typed executable configuration and adapter-independent ports.

#### Scenario: Create Pi-backed agent node
r[n8nclone.workflow_execution.agent_nodes.config.pi]
- GIVEN a workflow author adds a generic agent node targeting Pi
- WHEN the node is stored in the workflow
- THEN the system records the target runtime, prompt source, ordered text/image/JSON inputs, output mode, working directory, context references, tool policy, model hints, credential references, session references, and output ports as typed configuration

#### Scenario: Create Clankers-backed agent node
r[n8nclone.workflow_execution.agent_nodes.config.clankers]
- GIVEN a workflow author adds a generic agent node targeting Clankers
- WHEN the node is stored in the workflow
- THEN the system records the same adapter-neutral agent request fields without requiring a Clankers-specific graph node kind

#### Scenario: Reject invalid generic agent configuration
r[n8nclone.workflow_execution.agent_nodes.config.invalid]
- GIVEN a generic agent node is missing a required prompt source or declares an unsupported target runtime
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic for that node before execution starts

### Requirement: Generic agent node execution
r[n8nclone.workflow_execution.agent_nodes.execute] The system MUST execute generic agent nodes through adapter traits and map adapter results onto typed graph output values.

#### Scenario: Generate image through generic agent node
r[n8nclone.workflow_execution.agent_nodes.execute.image]
- GIVEN a workflow routes a user selfie and several inspiration images into a generic agent node with image output mode
- WHEN the configured adapter returns a generated image
- THEN the node completes with the generated image and adapter metadata under the node's real output port IDs

#### Scenario: Produce text or JSON through generic agent node
r[n8nclone.workflow_execution.agent_nodes.execute.text_json]
- GIVEN a generic agent node is configured for text or JSON output mode
- WHEN the configured adapter returns a valid response for that mode
- THEN the node completes with typed text or JSON values under the node's real output port IDs

#### Scenario: Adapter result type mismatch
r[n8nclone.workflow_execution.agent_nodes.execute.type_mismatch]
- GIVEN a generic agent node expects image output mode
- WHEN the configured adapter returns only text output
- THEN the system fails the node with an output type diagnostic instead of writing a mistyped graph value

### Requirement: Pi agent adapter
r[n8nclone.workflow_execution.agent_nodes.pi_adapter] The system MUST execute generic agent nodes targeting Pi through a Pi adapter interface.

#### Scenario: Pi adapter accepts image prompt request
r[n8nclone.workflow_execution.agent_nodes.pi_adapter.image_prompt]
- GIVEN a generic agent node targets Pi and includes prompt text plus image inputs
- WHEN the Pi adapter prepares the request through SDK, print, or RPC integration
- THEN the adapter receives an image-capable prompt request without the core invoking Pi directly

#### Scenario: Pi preflight rejection
r[n8nclone.workflow_execution.agent_nodes.pi_adapter.preflight_reject]
- GIVEN Pi rejects a prompt before acceptance because required configuration or credentials are unavailable
- WHEN the workflow runs the generic agent node
- THEN the system fails that node with the Pi preflight diagnostic and does not retry through another target silently

### Requirement: Clankers agent adapter
r[n8nclone.workflow_execution.agent_nodes.clankers_adapter] The system MUST execute generic agent nodes targeting Clankers through a Clankers adapter interface.

#### Scenario: Clankers adapter accepts image context request
r[n8nclone.workflow_execution.agent_nodes.clankers_adapter.image_context]
- GIVEN a generic agent node targets Clankers and includes prompt text plus image inputs
- WHEN the Clankers adapter prepares the request through one-shot, daemon, MCP, ACP, or context-reference integration
- THEN the adapter receives an image-capable agent request without the core invoking Clankers directly

#### Scenario: Clankers session failure
r[n8nclone.workflow_execution.agent_nodes.clankers_adapter.session_failure]
- GIVEN a generic agent node references a Clankers session that is unavailable
- WHEN the workflow prepares the adapter request
- THEN the system fails that node with a session diagnostic before sending prompt content

### Requirement: Generic agent cache identity
r[n8nclone.workflow_execution.agent_nodes.cache_key] The system MUST include upstream value hashes and executable agent configuration in generic agent node cache keys.

#### Scenario: Prompt edit avoids stale generated output
r[n8nclone.workflow_execution.agent_nodes.cache_key.prompt_edit]
- GIVEN a generic agent node previously cached output for one prompt
- WHEN the same upstream inputs are run with a different prompt
- THEN the executor invokes the configured adapter instead of reusing the previous output

#### Scenario: Target edit avoids stale generated output
r[n8nclone.workflow_execution.agent_nodes.cache_key.target_edit]
- GIVEN a generic agent node previously cached output for the Pi target
- WHEN the node is run with the Clankers target and otherwise matching inputs
- THEN the executor invokes the Clankers adapter instead of reusing the Pi output

#### Scenario: Image edit avoids stale generated output
r[n8nclone.workflow_execution.agent_nodes.cache_key.image_edit]
- GIVEN a generic agent node previously cached output for one upstream image hash
- WHEN the prompt is unchanged but an upstream image hash changes
- THEN the executor invokes the configured adapter instead of reusing the previous output

### Requirement: Generic agent node error handling
r[n8nclone.workflow_execution.agent_nodes.errors] The system MUST report image load, decode, credential, session, preflight, and adapter errors as node failures without corrupting workflow state.

#### Scenario: Decode failure remains local to node
r[n8nclone.workflow_execution.agent_nodes.errors.decode]
- GIVEN an image input node receives corrupt image bytes
- WHEN the decoder interface reports a decode failure
- THEN the system fails that node and leaves saved workflow configuration unchanged

#### Scenario: Credential failure skips adapter call
r[n8nclone.workflow_execution.agent_nodes.errors.credential]
- GIVEN a generic agent node requires a missing or invalid credential reference
- WHEN the workflow prepares the adapter request
- THEN the system fails the node before invoking the Pi or Clankers adapter

#### Scenario: Adapter failure is node-scoped
r[n8nclone.workflow_execution.agent_nodes.errors.adapter]
- GIVEN a Pi or Clankers adapter returns an execution error
- WHEN the workflow run processes that error
- THEN the system marks the generic agent node failed and reports the adapter error without freezing TUI or CLI behavior
