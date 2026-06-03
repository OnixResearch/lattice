## Context

Shell nodes can manipulate files, but the workflow model cannot inspect or validate those side effects. File nodes should make common file operations explicit while respecting the functional-core / imperative-shell split.

## Decisions

### 1. File nodes produce operation plans

**Choice:** Convert file node config and inputs into typed read, write, or append operation plans before execution.

**Rationale:** Path validation and output mapping can be tested without touching the filesystem.

### 2. Filesystem effects use an interface

**Choice:** Execute plans through a `FileSystem` interface implemented by the imperative shell.

**Rationale:** Tests can use an in-memory filesystem and the core never performs direct file I/O.

### 3. Path policy is explicit

**Choice:** Validate file paths against configured roots, relative-path rules, and overwrite settings before executing writes.

**Rationale:** File operations are useful only if unsafe paths and accidental overwrites are handled predictably.

### 4. FCIS boundary keeps side effects outside core

**Choice:** The core owns operation planning and diagnostics; the shell owns reading, writing, appending, permission errors, and OS-specific behavior.

**Rationale:** This keeps file nodes deterministic until the boundary where effects are intended.
