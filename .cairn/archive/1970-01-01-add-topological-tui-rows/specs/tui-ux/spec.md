## ADDED Requirements

### Requirement: Topological rows and branch lanes

r[lattice.tui_ux.rows.topology] The TUI MUST render each workflow node once in deterministic dependency order, using horizontal lane columns and connectors for forks and joins, including unequal-length branches and disconnected roots.

#### Scenario: Workflow topology snapshots

r[lattice.tui_ux.rows.topology.snapshots]
- GIVEN linear, fork-join, and diamond workflow fixtures
- WHEN the rows surface renders
- THEN maintained screen-column snapshots MUST cover their node order and lane connectors
- AND every dependency MUST precede its child without changing graph coordinates.

### Requirement: Live observed metadata

r[lattice.tui_ux.rows.metadata] Rows MUST display state, local observed start time, elapsed or final observed duration, observed dispatch-attempt count, and a bounded meaningful output summary, updating from actual per-node execution observations before whole-run completion. Provider usage MAY occupy additional columns at wider widths; absent observations MUST NOT be invented.

#### Scenario: Completed and non-successful nodes update during a run

r[lattice.tui_ux.rows.metadata.live]
- GIVEN the workflow still has active or pending work
- WHEN a node's final admitted result is observed
- THEN rows MUST display its actual completed, failed, or skipped state and available output/usage without waiting for the final run event
- AND preparation or admission failures MUST NOT be presented as completed
- AND a skipped or cache-only result without a dispatch-start observation MUST NOT invent an attempt
- AND whole-run completion MUST NOT duplicate already-observed output.

### Requirement: Shared selection and viewport

r[lattice.tui_ux.rows.selection] The canvas `v` binding MUST switch graph and rows layouts without changing selected node identity. Row navigation and clicks MUST update the shared selection and keep it visible; row drags and lateral arrows MUST NOT mutate the hidden spatial graph.

#### Scenario: Navigate beyond the visible rows

r[lattice.tui_ux.rows.selection.viewport]
- GIVEN a workflow has more nodes than fit in the canvas
- WHEN the operator uses arrows, page navigation, Home/End, or the mouse wheel
- THEN the selected row MUST remain visible and its properties/output ownership MUST remain the selected node
- AND clicking a scrolled row MUST select that displayed node, not its unscrolled index.

### Requirement: Bounded visible lanes

r[lattice.tui_ux.rows.wide] Visible lane columns MUST be bounded by terminal width and six lanes, with a `+N` overflow marker and a shared-lane marker, while preserving one independently selectable row per node.

#### Scenario: Fan-out exceeds lane capacity

r[lattice.tui_ux.rows.wide.overflow]
- GIVEN a workflow fan-out exceeds available lane columns
- WHEN rows render or the terminal resizes
- THEN connector width MUST remain bounded, overflow MUST be explicit, and every node MUST remain present and navigable.
