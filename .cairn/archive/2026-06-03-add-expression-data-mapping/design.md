## Context

Current edges can pass output values to input ports, but node configuration such as commands, URLs, headers, file paths, and environment values cannot be assembled from upstream data in a structured way.

## Decisions

### 1. Expressions are deterministic and non-Turing-complete

**Choice:** Support references, string interpolation, field selection, and bounded built-in transforms only.

**Rationale:** Expressions should map workflow data, not become another arbitrary execution engine.

### 2. Evaluation is a pure functional core

**Choice:** Parse expressions into an AST and evaluate them against an explicit immutable context of node outputs and metadata.

**Rationale:** Missing references, type mismatches, and rendering can be tested with plain in-memory values.

### 3. Node adapters receive resolved config

**Choice:** Resolve expression-bearing config before invoking shell, HTTP, or file adapters.

**Rationale:** Side-effecting adapters should receive concrete values or fail before any external effect begins.

### 4. Interfaces preserve redaction and secrets

**Choice:** Expression values carry sensitivity metadata so renderers and adapters can redact secret-derived strings consistently.

**Rationale:** Expression mapping must compose with credential interfaces without leaking secret material.

### 5. FCIS boundary excludes I/O and process execution

**Choice:** Expressions cannot read files, environment variables, clocks, network resources, or spawn commands.

**Rationale:** The mapping layer stays deterministic and safe.
