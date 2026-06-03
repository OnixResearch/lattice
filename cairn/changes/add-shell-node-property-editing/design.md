## Context

`NodeConfig::Shell` already stores command, cwd, env, timeout, and pipe-stdin values. `render_properties` shows these fields, while `handle_properties_key` is currently a placeholder.

## Decisions

### 1. Use a focused edit model

**Choice:** Add explicit property edit state to `App` for the active field, draft text, and original value.

**Rationale:** A small edit model avoids hidden terminal input behavior and keeps save/cancel transitions deterministic.

### 2. Validate in a pure core helper

**Choice:** Parse and validate editable property values in pure helper functions before mutating workflow state.

**Rationale:** Invalid timeout, cwd, or environment edits can be tested without terminal setup or a database.

### 3. Apply changes through workflow APIs

**Choice:** Property saves update `Workflow` and graph labels through explicit methods, then sync graph state.

**Rationale:** This keeps workflow metadata, node config, and rendered graph consistent.
