# Workflow-execution evidence completion

- audited rows: 83
- exact rows after completion: 83
- Tracey missing before/after: 181 / 181
- dangling markers: 0
- reviewed inventory BLAKE3: `26773e8f3ffa0635c9c45cfc9e4a3a843172c807d7f46cdaad2696e1b80ff170`

The pinned Rats snapshot now contains the previously omitted nested manual assertions module and all source-referenced generated snapshots. The repository check rail performs a focused `cargo check --package lattice --all-targets`, which compiles Lattice against that source closure without requiring the unrelated upstream Rats workspace fetch.

Subworkflow nodes now dispatch the named saved child through a bounded runner shell. Pure validation binds named inputs only to unconnected child ports, maps declared child outputs onto the parent graph's real output port IDs, and rejects missing outputs, identity drift, recursion, unavailable runners, and event-correlation drift without manufacturing output. The headless production shell builds an owned workflow catalog before execution, so child lookup has no ambient store authority.

No selected workflow-execution marker is promoted in this completion tranche.
