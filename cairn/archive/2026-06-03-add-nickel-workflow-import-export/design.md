## Context

The config layer already evaluates Nickel for app config and node templates. Workflow persistence currently stores serialized `Workflow` values directly in redb.

## Decisions

### 1. Nickel is the portable source format

**Choice:** Define a typed Nickel workflow contract and import/export workflows through that contract.

**Rationale:** Nickel contracts catch malformed definitions before runtime and produce reviewable files.

### 2. Import validates before mutation

**Choice:** Evaluate and validate Nickel into an in-memory workflow candidate, then save only after schema and graph checks pass.

**Rationale:** Bad imports must not overwrite existing saved workflows.

### 3. FCIS boundary separates conversion from I/O

**Choice:** Keep workflow-to-portable-model conversion, graph validation, and round-trip comparison pure. Keep file reads, Nickel evaluation, store writes, and CLI/TUI messages in the shell.

**Rationale:** The conversion core can be tested with fixtures without touching the filesystem or database.
