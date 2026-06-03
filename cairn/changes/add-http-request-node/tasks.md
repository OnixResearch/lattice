## Phase 1

- [ ] [serial] r[n8nclone.workflow_execution.http_node.config] Add HTTP node configuration, ports, validation, and cache-key material.
- [ ] [serial] r[n8nclone.workflow_execution.http_node.execute] Execute HTTP request plans through an HTTP client interface.
- [ ] [parallel] r[n8nclone.workflow_execution.http_node.outputs] Map status, headers, and body to real graph output ports.
- [ ] [parallel] r[n8nclone.workflow_execution.http_node.errors] Add negative tests for invalid URL, timeout, and transport failure cases.
- [ ] [parallel] r[n8nclone.workflow_execution.http_node.cache_key] Verify edited request configuration does not reuse stale cache entries.
