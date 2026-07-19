# Workflow Execution Specification

## Purpose

Defines the `workflow-execution` capability.

## Requirements

### Requirement: Shell pipeline stdin propagation
r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin] The system MUST pass a connected shell node output value into a downstream shell node's stdin input.

#### Scenario: Stdout reaches downstream stdin
r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin.connected]
- GIVEN a workflow connects one shell node's stdout port to another shell node's stdin port
- WHEN the workflow runs
- THEN the downstream shell command receives the upstream stdout bytes on standard input

### Requirement: Real graph output port values
r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs] The system MUST store completed shell outputs under the shell node's real graph output port IDs.

#### Scenario: Completed shell outputs use real ports
r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs.completed]
- GIVEN a shell node has stdout, stderr, and exit output ports
- WHEN that shell node completes
- THEN the completed output map uses those real port IDs for stdout, stderr, and exit values

### Requirement: Cache output port identity
r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports] The system SHOULD treat cached shell outputs as values keyed by real graph output port IDs.

#### Scenario: Cached outputs remain routable
r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports.routable]
- GIVEN a cached shell output is used during a workflow run
- WHEN a downstream edge reads the cached upstream output
- THEN the downstream input lookup uses the same real output port ID as non-cached execution

### Requirement: Shell cache keys include executable configuration
r[n8nclone.workflow_execution.cache_key.shell_config] The system MUST include shell node executable configuration in the cache key used to reuse completed shell outputs.

#### Scenario: Command edits do not reuse stale output
r[n8nclone.workflow_execution.cache_key.shell_config.command_edit]
- GIVEN a shell node previously cached output for one command
- WHEN the same node inputs are run with a different command
- THEN the executor runs the new command instead of reusing the previous command's cached output

### Requirement: Headless workflow execution
r[n8nclone.workflow_execution.headless.run] The system MUST let operators run a saved workflow without starting the TUI.

#### Scenario: Run saved workflow
r[n8nclone.workflow_execution.headless.run.saved]
- GIVEN a workflow exists in the configured store
- WHEN the operator runs it through the headless command
- THEN the system loads the workflow, executes its nodes, and reports the completed run

#### Scenario: Missing workflow
r[n8nclone.workflow_execution.headless.run.missing]
- GIVEN no saved workflow matches the requested name
- WHEN the operator runs it through the headless command
- THEN the system reports a clear load error without starting the TUI

### Requirement: Headless exit status
r[n8nclone.workflow_execution.headless.exit_status] The system MUST return a deterministic process result for headless workflow runs.

#### Scenario: Successful workflow exits successfully
r[n8nclone.workflow_execution.headless.exit_status.success]
- GIVEN every node in the workflow completes successfully
- WHEN the headless run finishes
- THEN the command exits successfully

#### Scenario: Failed workflow exits with failure
r[n8nclone.workflow_execution.headless.exit_status.failure]
- GIVEN at least one node in the workflow fails
- WHEN the headless run finishes
- THEN the command exits with failure and identifies the failed node

### Requirement: Headless run summary
r[n8nclone.workflow_execution.headless.summary] The system SHOULD expose headless run summaries in human-readable and machine-readable forms.

#### Scenario: Text summary
r[n8nclone.workflow_execution.headless.summary.text]
- GIVEN a headless run reaches a terminal state
- WHEN the operator requests default output
- THEN the system prints a concise text summary with run status, node count, failures, and cache hits

#### Scenario: JSON summary
r[n8nclone.workflow_execution.headless.summary.json]
- GIVEN a headless run reaches a terminal state
- WHEN the operator requests JSON output
- THEN the system prints a structured summary that automation can parse deterministically

### Requirement: Headless run history
r[n8nclone.workflow_execution.headless.history] The system SHOULD persist headless run records using the same history model as TUI runs.

#### Scenario: Persist headless run
r[n8nclone.workflow_execution.headless.history.persist]
- GIVEN a headless run reaches a terminal state
- WHEN persistence is available
- THEN the system stores a run record that appears in recent run history

### Requirement: Headless error handling
r[n8nclone.workflow_execution.headless.errors] The system MUST handle headless load and execution errors without corrupting persisted workflow state.

#### Scenario: Load failure preserves store
r[n8nclone.workflow_execution.headless.errors.load_failure]
- GIVEN the requested workflow cannot be loaded
- WHEN the headless command fails
- THEN the system leaves saved workflows unchanged and reports the error

### Requirement: HTTP node configuration
r[n8nclone.workflow_execution.http_node.config] The system MUST model HTTP request nodes with typed executable configuration.

#### Scenario: Create HTTP request node
r[n8nclone.workflow_execution.http_node.config.create]
- GIVEN a workflow author adds an HTTP request node
- WHEN the node is stored in the workflow
- THEN the system records method, URL, headers, body source, timeout, and output ports as typed configuration

#### Scenario: Invalid HTTP configuration
r[n8nclone.workflow_execution.http_node.config.invalid]
- GIVEN an HTTP request node has an invalid method or URL
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic for that node

### Requirement: HTTP node execution
r[n8nclone.workflow_execution.http_node.execute] The system MUST execute HTTP request nodes through an HTTP client interface.

#### Scenario: Successful HTTP request
r[n8nclone.workflow_execution.http_node.execute.success]
- GIVEN an HTTP request node has valid configuration
- WHEN the workflow runs and the HTTP client returns a response
- THEN the system completes the node and stores response values under the node's real output port IDs

#### Scenario: HTTP client failure
r[n8nclone.workflow_execution.http_node.execute.failure]
- GIVEN the HTTP client reports a transport failure
- WHEN the workflow runs
- THEN the system marks the HTTP node failed and reports the failure without freezing other UI or CLI behavior

### Requirement: HTTP node outputs
r[n8nclone.workflow_execution.http_node.outputs] The system MUST expose HTTP response status, headers, and body as routable graph outputs.

#### Scenario: Route HTTP body downstream
r[n8nclone.workflow_execution.http_node.outputs.body]
- GIVEN an HTTP node's body output is connected to a downstream input
- WHEN the HTTP node completes successfully
- THEN the downstream node receives the response body value from the HTTP node's real body output port

### Requirement: HTTP node error handling
r[n8nclone.workflow_execution.http_node.errors] The system MUST report invalid request, timeout, and transport failures as node failures.

#### Scenario: Request timeout
r[n8nclone.workflow_execution.http_node.errors.timeout]
- GIVEN an HTTP request exceeds the configured timeout
- WHEN the workflow runs
- THEN the system fails the HTTP node with a timeout error and continues reporting run status

### Requirement: HTTP cache identity
r[n8nclone.workflow_execution.http_node.cache_key] The system MUST include executable HTTP request configuration in HTTP node cache keys.

#### Scenario: URL edit avoids stale cache
r[n8nclone.workflow_execution.http_node.cache_key.url_edit]
- GIVEN an HTTP node previously cached output for one URL
- WHEN the node is run with a different URL
- THEN the executor performs the new request instead of reusing the previous response

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

### Requirement: Expression syntax
r[n8nclone.workflow_execution.expression_mapping.syntax] The system MUST define a deterministic expression syntax for referencing workflow data.

#### Scenario: Parse output reference
r[n8nclone.workflow_execution.expression_mapping.syntax.reference]
- GIVEN an expression references an upstream node output by stable node and port identity
- WHEN the expression parser runs
- THEN the system produces an expression AST that identifies the referenced value

#### Scenario: Reject invalid syntax
r[n8nclone.workflow_execution.expression_mapping.syntax.invalid]
- GIVEN an expression contains invalid syntax
- WHEN workflow validation runs
- THEN the system reports a stable expression diagnostic before execution starts

### Requirement: Expression evaluation
r[n8nclone.workflow_execution.expression_mapping.evaluate] The system MUST evaluate expressions against completed upstream workflow values before executing dependent nodes.

#### Scenario: Interpolate upstream stdout
r[n8nclone.workflow_execution.expression_mapping.evaluate.stdout]
- GIVEN an expression interpolates an upstream shell node's stdout
- WHEN the upstream node completes and the dependent node is prepared
- THEN the system evaluates the expression to the upstream stdout value

#### Scenario: Evaluate bounded transform
r[n8nclone.workflow_execution.expression_mapping.evaluate.transform]
- GIVEN an expression applies a supported bounded transform to an upstream value
- WHEN expression evaluation runs
- THEN the system returns the transformed value without performing I/O or process execution

### Requirement: Expression-backed node configuration
r[n8nclone.workflow_execution.expression_mapping.config] The system SHOULD allow expression values in supported node configuration fields.

#### Scenario: Resolve command argument
r[n8nclone.workflow_execution.expression_mapping.config.command]
- GIVEN a shell node command field contains an expression
- WHEN the node is prepared after its dependencies complete
- THEN the system resolves the expression before invoking the shell adapter

#### Scenario: Resolve HTTP body
r[n8nclone.workflow_execution.expression_mapping.config.http_body]
- GIVEN an HTTP node body field contains an expression
- WHEN the node is prepared after its dependencies complete
- THEN the system resolves the expression before invoking the HTTP adapter

### Requirement: Expression error handling
r[n8nclone.workflow_execution.expression_mapping.errors] The system MUST fail dependent node preparation when expression references are missing, mistyped, or unavailable.

#### Scenario: Missing reference
r[n8nclone.workflow_execution.expression_mapping.errors.missing]
- GIVEN an expression references a node or port that is not available in the evaluation context
- WHEN the dependent node is prepared
- THEN the system fails that node before executing any side-effecting adapter

#### Scenario: Type mismatch
r[n8nclone.workflow_execution.expression_mapping.errors.type]
- GIVEN an expression result cannot be converted to the target field type
- WHEN the dependent node is prepared
- THEN the system reports a type diagnostic and does not execute the node

### Requirement: Expression redaction
r[n8nclone.workflow_execution.expression_mapping.redaction] The system MUST preserve sensitivity metadata when expressions derive values from secrets.

#### Scenario: Secret-derived expression output
r[n8nclone.workflow_execution.expression_mapping.redaction.secret]
- GIVEN an expression combines a secret-derived value with non-secret text
- WHEN the expression result is rendered in logs, TUI, history, or JSON summaries
- THEN the system redacts the secret-derived portion according to shared redaction rules

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

### Requirement: Production agent runtime adapters
r[n8nclone.workflow_execution.agent_nodes.production_adapters] The system MUST run generic agent nodes through production Pi and Clankers runtime adapters in default TUI and headless execution while preserving injectable adapter seams for tests.

#### Scenario: Production runtime adapter configuration
r[n8nclone.workflow_execution.agent_nodes.production_adapters.config]
- GIVEN default runtime configuration is loaded
- WHEN a generic agent node targets Pi or Clankers
- THEN the system SHOULD have deterministic command defaults and MAY accept explicit command overrides for each target

#### Scenario: Target-specific production routing
r[n8nclone.workflow_execution.agent_nodes.production_adapters.route]
- GIVEN a generic agent node targets Pi or Clankers
- WHEN production execution prepares the adapter request
- THEN the system MUST route the request only to the matching runtime adapter and MUST NOT silently retry through another target

#### Scenario: Command adapter text and JSON execution
r[n8nclone.workflow_execution.agent_nodes.production_adapters.command]
- GIVEN a generic agent node requests text or JSON output through a configured production command
- WHEN the command exits successfully with compatible stdout
- THEN the system MUST map stdout into the requested typed agent output and include safe adapter metadata

#### Scenario: Production adapter errors remain node-scoped
r[n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
- GIVEN a production agent command is missing, exits non-zero, returns invalid JSON, or receives an unsupported image or artifact output request
- WHEN the workflow runs that generic agent node
- THEN the system MUST fail only that node with a clear adapter diagnostic and MUST NOT corrupt saved workflow state

#### Scenario: Default executor uses production agent routing
r[n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
- GIVEN a workflow contains a Pi-backed or Clankers-backed generic agent node
- WHEN the workflow runs through `Executor::new()`
- THEN the default executor MUST use production agent routing instead of the placeholder no-op adapter

### Requirement: Generic command agent runtimes
r[n8nclone.workflow_execution.agent_nodes.command_runtimes] The system MUST execute production generic agent nodes through configurable command runtime entries rather than hard-coded Pi or Clankers command branches.

#### Scenario: Runtime map configuration
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.config]
- GIVEN default application configuration is created or loaded
- WHEN production agent runtime settings are inspected
- THEN the system MUST expose deterministic command runtime presets for Pi and Clankers in a generic runtime map

#### Scenario: Runtime-map command planning
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.planning]
- GIVEN a generic agent node targets a configured runtime entry
- WHEN production command planning runs
- THEN the system MUST select the matching runtime entry and render configured argument placeholders without target-specific command branches

#### Scenario: Runtime configuration errors
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.errors]
- GIVEN a generic agent node targets a missing runtime entry or a runtime entry with an empty command
- WHEN production command planning runs
- THEN the system MUST reject execution before spawning a process with a clear node-scoped diagnostic

#### Scenario: Preset behavior remains compatible
r[n8nclone.workflow_execution.agent_nodes.command_runtimes.compatibility]
- GIVEN existing Pi or Clankers generic agent nodes use default runtime presets
- WHEN workflows run through production TUI or headless execution
- THEN text, JSON, image handoff, and artifact handoff behavior MUST remain compatible with the prior production adapter contract

### Requirement: Agent output file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff] The system MUST support bounded local file handoff for production generic agent image and artifact outputs while preserving text and JSON stdout behavior.

#### Scenario: Handoff envelope contract
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
- GIVEN a production generic agent command runs in image or artifact output mode
- WHEN it exits successfully
- THEN stdout MUST be parsed as a JSON envelope with a required local file path and optional media type and metadata

#### Scenario: Image file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
- GIVEN an image-mode production generic agent command returns a valid handoff envelope for a PNG or JPEG file
- WHEN the adapter decodes the command result
- THEN the system MUST return a typed image value with sniffed media type, stable content hash, source metadata, and adapter metadata

#### Scenario: Artifact file handoff
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact]
- GIVEN an artifact-mode production generic agent command returns a valid handoff envelope for a local file
- WHEN the adapter decodes the command result
- THEN the system MUST return the file bytes as the typed artifact output and include safe adapter metadata

#### Scenario: Handoff safety failures are node-scoped
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
- GIVEN a handoff envelope is invalid, missing its file, outside the command working directory, oversized, has mismatched media type, or references unsupported image bytes
- WHEN the workflow runs that generic agent node
- THEN the system MUST fail only that node with a clear adapter diagnostic and MUST NOT corrupt saved workflow state

#### Scenario: Default runtime maps image handoff to workflow ports
r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.default_runtime]
- GIVEN a saved workflow contains an image-mode generic agent node using a configured production command
- WHEN the workflow runs through the default headless runtime path
- THEN the image file handoff MUST be mapped to the node's real image output port and cached as a typed image value

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

### Requirement: Generic pipeline runner core
r[n8nclone.workflow_execution.generic_pipeline_runner] The system MUST provide a generic typed workflow/pipeline runner that separates pure DAG admission and scheduling from side-effecting runtime execution.

#### Scenario: Runner IR plan creation
r[n8nclone.workflow_execution.runner_core.plan_ir]
- GIVEN a persisted workflow graph with typed nodes, ports, edges, node configuration, credential references, and runtime bindings
- WHEN runner planning starts
- THEN the system MUST convert it into a stable in-memory runner plan without performing filesystem, process, network, clock, database, or UI operations

#### Scenario: Trellis-aligned graph admission
r[n8nclone.workflow_execution.runner_core.trellis_graph_admission]
- GIVEN a runner plan has finite nodes, edges, dependencies, and capacity policy
- WHEN pure runner admission evaluates the plan
- THEN the system SHOULD align edge endpoint, dependency readiness, topological, cycle, and capacity decisions with the matching Trellis graph, job DAG, job state, and dependency scheduler primitive semantics where build and license fit

#### Scenario: Step lifecycle admission
r[n8nclone.workflow_execution.runner_core.lifecycle]
- GIVEN a runner step is pending, ready, running, completed, failed, or cancelled
- WHEN the runner applies a lifecycle transition
- THEN the system MUST admit only valid transitions, MUST reject self-transitions and terminal-state mutation, and MUST keep rejected transitions free of side effects

#### Scenario: Compatibility facade
r[n8nclone.workflow_execution.runner_core.compatibility]
- GIVEN existing TUI, headless, scheduler, and test entrypoints invoke workflow execution
- WHEN the generic runner core and shell are introduced
- THEN those entrypoints MUST continue to execute existing shell, HTTP, file, image, and generic agent workflows with equivalent typed outputs and cache behavior

### Requirement: Pluggable node runtime registry
r[n8nclone.workflow_execution.runtime_registry] The system MUST execute node side effects through a runtime registry instead of executor-local node branches while preserving typed node semantics.

#### Scenario: Runtime dispatch
r[n8nclone.workflow_execution.runtime_registry.dispatch]
- GIVEN a runner step references a node kind and runtime binding present in the registry
- WHEN the runner shell invokes the step
- THEN the system MUST dispatch to the matching runtime implementation with a typed invocation plan

#### Scenario: Runtime output contract
r[n8nclone.workflow_execution.runtime_registry.typed_output_contract]
- GIVEN a runtime returns output values for a step
- WHEN the runner admits the runtime result
- THEN every output MUST be keyed by a declared output port id and MUST match that port's declared type tag before values are stored or routed downstream

#### Scenario: Runtime errors remain node-scoped
r[n8nclone.workflow_execution.runtime_registry.errors]
- GIVEN a runtime binding is missing, a runtime returns an undeclared output port, a runtime returns a value with an incompatible type, or a runtime fails
- WHEN the workflow runs
- THEN the system MUST fail only the affected step with a clear diagnostic and MUST NOT corrupt workflow configuration or unrelated completed values

#### Scenario: Generic command runtime
r[n8nclone.workflow_execution.runtime_registry.command_runtime]
- GIVEN a shell-like or agent-like node is backed by a configured command runtime
- WHEN the runner dispatches the node
- THEN the system MUST use the generic command runtime contract for command, arguments, working directory, stdin, stdout/stderr, output envelope decoding, timeout, and result admission

### Requirement: Unified runner events
r[n8nclone.workflow_execution.runner_events] The system MUST expose workflow execution progress as a unified ordered event stream usable by TUI, headless, scheduler, and tests.

#### Scenario: Event stream lifecycle
r[n8nclone.workflow_execution.runner_events.stream]
- GIVEN a workflow run starts and nodes become ready, start, use cache, produce artifacts, complete, fail, or cancel
- WHEN the runner shell processes those decisions
- THEN the system MUST emit deterministic runner events with stable run id, node id, optional port id, event kind, sequence ordering, and safe summaries

#### Scenario: Event redaction
r[n8nclone.workflow_execution.runner_events.redaction]
- GIVEN runtime outputs or inputs contain sensitive fragments or secret-derived values
- WHEN runner events, TUI output, headless summaries, or persisted run details render those values
- THEN the system MUST preserve redaction metadata and MUST NOT expose sensitive fragments in unsafe summaries

### Requirement: Typed artifact and durable references
r[n8nclone.workflow_execution.artifact_references] The system MUST represent large or durable workflow values as typed content-addressed references while preserving inline scalar values.

#### Scenario: Artifact reference model
r[n8nclone.workflow_execution.artifact_references.model]
- GIVEN a node produces bytes, images, command-output files, or generated artifacts that exceed inline policy or require durable storage
- WHEN the runner stores the output
- THEN the system MUST create a typed reference containing BLAKE3 content hash, type or media metadata, byte length, backend reference, sensitivity metadata, and safe display summary

#### Scenario: Trellis-aligned reference identity
r[n8nclone.workflow_execution.artifact_references.trellis_identity]
- GIVEN artifact references, schema migrations, or runtime closure identities are admitted by the runner
- WHEN pure admission evaluates those references
- THEN the system SHOULD align typed-reference, closure-availability, and explicit-migration decisions with Trellis typed durable reference and content-addressed code primitives where build and license fit

### Requirement: Durable runner admission
r[n8nclone.workflow_execution.durable_runner] The system SHOULD admit retry, resume, and idempotency decisions through a deterministic durable runner core.

#### Scenario: Resume and idempotency admission
r[n8nclone.workflow_execution.durable_runner.resume]
- GIVEN a workflow run has prior events, checkpoints, consumed execution tokens, retry policy, timeout policy, and completed durable effects
- WHEN the runner decides whether to execute or skip a step during retry or resume
- THEN the system SHOULD use deterministic durable-admission logic aligned with Trellis durable job, durable resume, durable replay, idempotency, timeout, retry, and job result primitives where build and license fit

### Requirement: Trait-driven workflow execution boundaries
r[n8nclone.workflow_execution.trait_boundaries] The system MUST keep persisted workflow configuration as serializable data while routing side-effecting workflow behavior through narrow trait-backed runtime, process, storage, and event ports.

#### Scenario: Runtime dispatch through registered runtimes
r[n8nclone.workflow_execution.trait_boundaries.runtime_dispatch]
- GIVEN a runner step has a node kind, runtime binding, declared ports, resolved settings, credentials, and input values
- WHEN the runner shell executes the step
- THEN the system MUST dispatch to a registered node runtime implementation for that binding instead of using executor-local node branches for supported built-in nodes

#### Scenario: Shared process runner
r[n8nclone.workflow_execution.trait_boundaries.process_runner]
- GIVEN a shell-like, generic command, or command-backed agent runtime needs to run an external process
- WHEN the runtime prepares execution
- THEN the system MUST route spawning through a shared process-runner interface that receives explicit program, args, cwd, env, stdin, timeout, and output-limit inputs

#### Scenario: Storage access through narrow ports
r[n8nclone.workflow_execution.trait_boundaries.storage_ports]
- GIVEN headless execution, scheduling, runner cache use, run-history persistence, or value storage needs persisted state
- WHEN those shells access storage
- THEN the system MUST depend on the narrowest workflow repository, run history, execution cache, value store, or schedule store trait needed by the operation

#### Scenario: Runner events through event sinks
r[n8nclone.workflow_execution.trait_boundaries.event_sink]
- GIVEN a workflow run starts, admits nodes, hits cache, invokes runtimes, produces outputs, fails, cancels, or completes
- WHEN the runner shell observes the lifecycle fact
- THEN the system MUST emit an ordered runner event through an event-sink interface with stable run id, node id, optional port id, event kind, safe summary, and redaction metadata

#### Scenario: Compatibility facade remains stable
r[n8nclone.workflow_execution.trait_boundaries.compatibility]
- GIVEN existing TUI, headless, scheduler, and test entrypoints call the public executor and store APIs
- WHEN trait-backed runtime, process, storage, and event ports are introduced
- THEN those entrypoints MUST preserve existing shell, HTTP, file, image, generic agent, cache, and run-history behavior

### Requirement: Replay event persistence
r[n8nclone.workflow_execution.replay.event_store] The system MUST persist ordered runner events and redaction-safe value summaries for workflow run replay.

#### Scenario: Persist completed run events
r[n8nclone.workflow_execution.replay.event_store.completed]
- GIVEN a workflow run emits runner events and reaches a terminal state
- WHEN run history is persisted
- THEN the system stores the ordered event sequence, node identities, optional port identities, safe summaries, redaction metadata, and artifact references

#### Scenario: Partial run remains replayable
r[n8nclone.workflow_execution.replay.event_store.partial]
- GIVEN a workflow run is interrupted before terminal completion
- WHEN the operator opens replay for that run
- THEN the system reconstructs a partial timeline from the persisted events without requiring side-effect re-execution

### Requirement: Replay core
r[n8nclone.workflow_execution.replay.core] The system MUST reconstruct replay timelines from immutable event records without executing workflow side effects.

#### Scenario: Fold events into node states
r[n8nclone.workflow_execution.replay.core.fold]
- GIVEN a valid ordered event sequence for a run
- WHEN replay reconstruction runs
- THEN the system produces deterministic timeline frames and node status at each cursor position

#### Scenario: Replay does not rerun side effects
r[n8nclone.workflow_execution.replay.core.no_side_effects]
- GIVEN a historical run contains shell, HTTP, file, image, agent, trigger, or subworkflow events
- WHEN replay mode opens that run
- THEN the system displays recorded events and summaries without invoking any runtime or adapter

### Requirement: Replay artifact placeholders
r[n8nclone.workflow_execution.replay.artifact_placeholders] The system SHOULD keep replay usable when artifact payloads are unavailable.

#### Scenario: Missing artifact shows placeholder
r[n8nclone.workflow_execution.replay.artifact_placeholders.missing]
- GIVEN a replay event references an artifact whose payload cannot be loaded
- WHEN the replay view requests artifact details
- THEN the system displays hash, media metadata, and a missing-payload placeholder instead of failing the replay

### Requirement: Conditional control-flow nodes
r[n8nclone.workflow_execution.control_flow.condition] The system MUST route values through conditional branch nodes using deterministic typed predicates evaluated without side effects.

#### Scenario: Matching branch emits output
r[n8nclone.workflow_execution.control_flow.condition.match]
- GIVEN a condition node receives an input value that satisfies one configured branch predicate
- WHEN the workflow runner evaluates the node
- THEN the system emits the input value only on the matching branch output port

#### Scenario: No matching branch is recoverable
r[n8nclone.workflow_execution.control_flow.condition.no_match]
- GIVEN a condition node receives an input value that satisfies no configured branch predicate
- WHEN the workflow runner evaluates the node
- THEN the system completes the node with no branch outputs and records a safe summary explaining that no branch matched

### Requirement: Merge control-flow nodes
r[n8nclone.workflow_execution.control_flow.merge] The system MUST merge multiple upstream values according to an explicit deterministic fan-in policy.

#### Scenario: All-input merge waits for dependencies
r[n8nclone.workflow_execution.control_flow.merge.all]
- GIVEN a merge node declares the all-input policy
- WHEN only some upstream dependencies have completed
- THEN the system keeps the merge node pending until every required upstream value is available or a dependency fails

#### Scenario: First-input merge is deterministic
r[n8nclone.workflow_execution.control_flow.merge.first]
- GIVEN a merge node declares the first-input policy and more than one upstream value becomes available
- WHEN the runner admits the merge result
- THEN the system selects the value with the lowest deterministic runner event sequence among available inputs

### Requirement: Subworkflow invocation nodes
r[n8nclone.workflow_execution.subworkflows.invocation] The system MUST invoke saved workflows as typed subworkflow nodes through the runner shell.

#### Scenario: Subworkflow outputs route to parent
r[n8nclone.workflow_execution.subworkflows.invocation.outputs]
- GIVEN a parent workflow contains a subworkflow node with valid input and output mappings
- WHEN the parent workflow runs
- THEN the system executes the referenced child workflow and maps declared child outputs to the parent node's real output port IDs

#### Scenario: Recursive subworkflow is rejected
r[n8nclone.workflow_execution.subworkflows.invocation.recursive]
- GIVEN a subworkflow reference would invoke a workflow already present in the current invocation ancestry
- WHEN runner admission runs
- THEN the system rejects the subworkflow before executing child workflow side effects

### Requirement: Subworkflow runner events
r[n8nclone.workflow_execution.subworkflows.events] The system SHOULD correlate parent and child run events for subworkflow execution.

#### Scenario: Child events keep parent context
r[n8nclone.workflow_execution.subworkflows.events.correlation]
- GIVEN a subworkflow node starts a child workflow run
- WHEN child runner events are emitted
- THEN the system includes parent run identity and subworkflow node identity in safe event metadata

### Requirement: Trigger payload binding
r[n8nclone.workflow_execution.triggers.payload_binding] The system MUST bind admitted trigger payload data to declared workflow input values before execution.

#### Scenario: JSON payload binds to input port
r[n8nclone.workflow_execution.triggers.payload_binding.json]
- GIVEN a webhook trigger maps a JSON body field to a workflow input port
- WHEN a valid request is admitted
- THEN the system supplies the mapped field as a typed input value for the workflow run

#### Scenario: Missing required payload field fails
r[n8nclone.workflow_execution.triggers.payload_binding.missing]
- GIVEN a trigger input binding requires a payload field that is absent
- WHEN trigger admission runs
- THEN the system rejects the event before starting the workflow

#### Scenario: Payload type mismatch fails
r[n8nclone.workflow_execution.triggers.payload_binding.type_mismatch]
- GIVEN a trigger input binding maps a payload value to an incompatible workflow input type tag
- WHEN trigger admission runs
- THEN the system reports a deterministic diagnostic and does not execute workflow side effects

### Requirement: Trigger run history
r[n8nclone.scheduling.triggers.history] The system SHOULD record trigger run outcomes with safe payload summaries.

#### Scenario: Persist trigger outcome
r[n8nclone.scheduling.triggers.history.persist]
- GIVEN a trigger event is accepted and starts a workflow run
- WHEN that run reaches a terminal state
- THEN the system stores trigger id, workflow name, run id, idempotency key, status, and redaction-safe payload summary

### Requirement: Seaglass execution graph core adoption
r[n8nclone.workflow_execution.seaglass_graph_core] The system MUST use the provenance-recorded Seaglass execution-graph core for reusable DAG admission and lifecycle decisions while retaining `n8nclone` runtime, value, policy, event, and artifact ownership.

#### Scenario: Existing runtime behavior remains compatible
r[n8nclone.workflow_execution.seaglass_graph_core.scenario.compatibility]
- GIVEN an existing valid workflow contains shell, HTTP, file, image, condition, merge, subworkflow, Pi, or Clankers nodes
- WHEN runner admission and execution use the shared graph core
- THEN deterministic order and readiness MUST remain compatible and typed runtime dispatch MUST stay in the `n8nclone` shell

#### Scenario: Vendored core records exact provenance
r[n8nclone.workflow_execution.seaglass_graph_core.scenario.provenance]
- GIVEN the Seaglass owner revision is not available through an authorized remote push
- WHEN the core snapshot is synchronized into `n8nclone`
- THEN a repo-owned import tool MUST record the exact source revision and product builds MUST NOT depend on a durable sibling path

### Requirement: LLM operating graph
r[n8nclone.workflow_execution.llm_operating_graph] The system MUST support role-based LLM workflows as admitted typed DAGs with explicit runtime, state, effect, and artifact boundaries.

#### Scenario: Role graph fans out and aggregates
r[n8nclone.workflow_execution.llm_operating_graph.scenario.fanout]
- GIVEN a workflow contains Markdown-backed research, build, review, or publishing agent roles with typed dependencies
- WHEN the graph runs through an injected or production agent adapter
- THEN independent ready roles MAY execute as admitted fan-out and downstream aggregation MUST wait for required predecessors

#### Scenario: Feedback remains between runs
r[n8nclone.workflow_execution.llm_operating_graph.scenario.feedback-boundary]
- GIVEN metrics, replies, failures, budget facts, or approval outcomes suggest a graph change
- WHEN the operating graph reorganizes
- THEN the change MUST be represented as explicit next-run state or graph input and MUST NOT bypass cycle admission, authority policy, or durable event recording

#### Scenario: Production file boundaries execute explicitly
r[n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
- GIVEN a role graph reads shared state and writes its final report through file nodes with explicit allowed roots
- WHEN the graph runs through the default headless production shell
- THEN local file operations MUST execute through the filesystem interface, MUST reject paths outside canonical allowed roots, and MUST leave downstream roles and effects blocked after a failed dependency

### Requirement: Rats dependency readiness
r[n8nclone.workflow_execution.rats_dependency] The system MUST consume the newest Rats source that is complete and passes the focused consumer compile checks.

#### Scenario: Source-distribution defect fails visibly
r[n8nclone.workflow_execution.rats_dependency.scenario.source-completeness]
- GIVEN the newest Rats revision references a source file omitted by ignore or archive rules
- WHEN `n8nclone` compiles against that revision
- THEN the update MUST remain blocked until the Rats owner source includes the file and the focused consumer check passes

### Requirement: Native OpenAI-compatible agent runtime
r[n8nclone.workflow_execution.agent_nodes.openai_compatible] The system MUST execute configured generic text and JSON agent roles through a repository-owned bounded OpenAI-compatible chat-completions adapter without requiring an external shell wrapper.

#### Scenario: Runtime route is unambiguous
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.route]
- GIVEN an agent target has exactly one command or OpenAI-compatible runtime entry
- WHEN production adapter admission selects that target
- THEN the adapter MUST select the configured transport deterministically and MUST reject missing or ambiguous routes before process or network I/O

#### Scenario: Request planning is bounded and typed
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan]
- GIVEN an OpenAI-compatible runtime declares a valid HTTP or HTTPS endpoint, model, positive timeout, token bound, request-byte bound, response-byte bound, and optional thinking setting
- WHEN a text or JSON agent request is planned
- THEN the pure planning core MUST produce a non-streaming bounded chat-completions request, MUST declare `json_object` response format and a JSON-only output contract for JSON output, and MUST reject invalid URLs, embedded URL credentials, fragments, unsupported media inputs or output modes, sessions, and invalid numeric bounds

#### Scenario: Bearer credential stays execution-only
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.credentials]
- GIVEN an HTTPS runtime names an API-key environment binding backed by a node credential reference
- WHEN the executor resolves and sends the request
- THEN the key MUST pass only through a non-serializable execution context, MUST NOT appear in Nickel, plans, metadata, or diagnostics, and MUST be required before network I/O

#### Scenario: Successful text and JSON responses decode strictly
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.success]
- GIVEN the endpoint returns a bounded successful chat-completions response with non-empty `choices[0].message.content`
- WHEN the requested output mode is text or JSON
- THEN the adapter MUST return typed text or parsed JSON output plus non-secret provider metadata and MUST NOT substitute hidden reasoning content

#### Scenario: Protocol and transport failures fail closed
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
- GIVEN the endpoint times out, redirects, returns a non-success status, exceeds the response bound, returns invalid UTF-8 or malformed JSON, omits choices or content, returns empty content, or returns malformed requested JSON
- WHEN the adapter executes or decodes the response
- THEN the node MUST fail with a bounded redacted adapter diagnostic and MUST produce no typed agent output

#### Scenario: Real role graph uses native transport
r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.real-graph]
- GIVEN the Markdown-backed role graph is configured for an available OpenAI-compatible model endpoint
- WHEN the headless production executor runs the graph without command runtime entries
- THEN every admitted role and final report effect MUST complete through the native adapter, with the endpoint model and bounded run evidence recorded

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
