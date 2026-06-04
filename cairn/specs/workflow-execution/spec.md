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
