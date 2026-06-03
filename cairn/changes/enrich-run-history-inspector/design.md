## Context

The store has `RunRecord` summaries and the TUI can list recent runs. Detailed node statuses and outputs are available during execution but are not stored as inspectable history records.

## Decisions

### 1. Store summaries and detail records separately

**Choice:** Keep the existing run summary list lightweight and add detail records keyed by run id.

**Rationale:** Listing recent runs stays fast while inspection can load richer data on demand.

### 2. Details are read-only snapshots

**Choice:** Persist node labels, statuses, cache markers, output snippets, and errors as observed at run time.

**Rationale:** Later workflow edits should not rewrite the historical record.

### 3. FCIS boundary uses pure formatters

**Choice:** Convert stored detail records into timeline rows and output views with pure formatting functions. The store and TUI shells own reads, writes, scrolling, and selection.

**Rationale:** Display logic can be tested over in-memory records.

### 4. Rerun is explicit

**Choice:** Rerun actions require user confirmation and resolve the current saved workflow by name rather than mutating historical records.

**Rationale:** History inspection remains safe and predictable.
