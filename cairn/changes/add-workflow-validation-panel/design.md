## Context

The workflow model combines graph topology with per-node execution configuration. The TUI mutates both and the executor assumes runnable structure when a run starts.

## Decisions

### 1. Validation is a pure functional core

**Choice:** Add a validator that accepts a `Workflow` snapshot and returns ordered diagnostics with severity, node id, optional port id, and stable code.

**Rationale:** Validation can be tested without TUI, store, or executor side effects.

### 2. The TUI renders diagnostics without owning rules

**Choice:** The validation panel displays diagnostics and navigation affordances, but rule decisions stay in the core validator.

**Rationale:** UI rendering should not duplicate correctness logic.

### 3. Run gating distinguishes fatal errors from warnings

**Choice:** Fatal diagnostics block execution; warnings remain visible but do not prevent runs.

**Rationale:** Users get safety without losing flexibility for non-fatal authoring choices.

### 4. FCIS boundary stays intact

**Choice:** The shell triggers validation before save/run/open actions and passes immutable workflow snapshots to the validator.

**Rationale:** The validator remains deterministic while panels and commands own user interaction.
