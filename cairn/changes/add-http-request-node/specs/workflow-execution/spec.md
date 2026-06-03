## ADDED Requirements

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
