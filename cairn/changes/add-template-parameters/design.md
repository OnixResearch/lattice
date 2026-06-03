## Context

The config layer loads Nickel node templates with fixed commands and ports. The TUI add-node flow can create nodes from templates, but it does not collect template-specific values.

## Decisions

### 1. Parameters are part of the Nickel template contract

**Choice:** Add typed parameter declarations with labels, descriptions, defaults, required markers, and validation constraints.

**Rationale:** Template authors can define the input shape close to the reusable template.

### 2. Rendering is pure and explicit

**Choice:** Convert a template plus parameter map into a node config using a pure renderer that fails on missing or invalid values.

**Rationale:** Tests can verify rendering without opening the TUI or evaluating side effects.

### 3. TUI owns prompting, not rendering rules

**Choice:** The add-node flow collects parameter drafts and displays validation messages, then calls the renderer with typed values.

**Rationale:** UI remains an imperative shell over reusable template logic.

### 4. FCIS boundary keeps Nickel and filesystem at the edge

**Choice:** Nickel evaluation and template-directory reads stay in configuration loading. The core receives already-decoded template and parameter data.

**Rationale:** Template behavior remains deterministic after loading.
