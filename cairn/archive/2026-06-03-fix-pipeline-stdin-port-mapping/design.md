## Context

`execute_shell` builds an output map with sentinel `PortId(u64::MAX - n)` keys. `Executor::run` stores those keys in `port_values`, but `gather_inputs` looks up upstream values by each edge's real `edge.source` port ID. That mismatch means downstream stdin cannot find the upstream stdout value even though the graph edge is correct.

## Decisions

### 1. Map shell outputs at the node boundary

**Choice:** Let `execute_shell` return positional shell outputs, then have `execute_node` map them onto the current node's real `stdout`, `stderr`, and `exit` output ports.

**Rationale:** The shell runner should not need graph knowledge, while the executor has the current node and can deterministically map positional outputs to port IDs.

### 2. Remove sentinel dependence from tests

**Choice:** Update executor tests and fake cache helpers to use actual graph output port IDs instead of sentinel IDs.

**Rationale:** Tests should verify public workflow semantics and catch future real-port mapping regressions.

## Risks / Trade-offs

- Existing caches keyed by sentinel output IDs would not feed downstream edges correctly; using real output IDs is the correct persisted shape going forward.
- This assumes shell nodes keep their existing output order: stdout, stderr, exit.
