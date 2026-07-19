## ADDED Requirements

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
