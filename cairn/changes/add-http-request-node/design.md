## Context

The workflow model currently has shell node configuration only. The executor can be extended with additional node config variants, but direct network I/O must not leak into pure validation or planning code.

## Decisions

### 1. HTTP execution is planned before effects

**Choice:** Convert an HTTP node and resolved inputs into an `HttpRequestPlan` containing method, URL, headers, body, timeout, and output port mapping.

**Rationale:** Request construction can be tested deterministically before any network call occurs.

### 2. Network I/O uses an interface

**Choice:** Execute requests through an `HttpClient` trait implemented by the imperative shell or executor adapter.

**Rationale:** Tests can use an in-memory client and the functional core never performs network access.

### 3. Cache keys include executable request configuration

**Choice:** Include method, URL, headers, body, timeout, and non-secret credential references in the cache key.

**Rationale:** Edited HTTP behavior must not reuse stale outputs.

### 4. FCIS boundary covers validation and redaction

**Choice:** Keep URL/header validation, output mapping, and redaction decisions pure; keep DNS, TLS, sockets, timeouts, and process reporting in shell adapters.

**Rationale:** Side effects remain replaceable and bounded.
