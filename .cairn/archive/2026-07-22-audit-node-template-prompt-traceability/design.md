# Design: audit node-template prompt traceability

## Success contract

The inventory MUST classify exactly all three node-template rows, retain all 327 missing IDs, report zero dangling markers, remain `valid: false`, and add no selected marker.

False completion includes test names, implementation shape, archived tasks, generated inventory, generic add-node cancellation, and unasserted panel rendering.

## Portfolio review

Three correlated repository lenses cover accepted-clause matching, maintained TUI test discovery, and adversarial marker/provenance exclusion. Retrieval is bounded to the accepted node-template spec, current inventory, `src/tui/app.rs`, `src/tui/panels.rs`, and repository-owned template tests.

The parent is exact because direct state-machine assertions observe selected-template parameter collection and rendered command creation. Cancellation is exact because the test enters parameter state, cancels, and directly asserts zero nodes, closed flow, and unchanged dirty state. Required prompting is partial: app state and panel implementation show the required parameter step, but no maintained assertion inspects the user-visible required prompt, so implementation shape cannot complete the scenario.

## Validation

A pure validator checks the exact three-row table, current hashes, selected-marker absence, unchanged coverage, and deterministic negative mutations.

## Non-claims

Partial evidence is not near-complete correctness. Inventory rows create no Tracey evidence and prove neither requirement satisfaction nor release readiness.
