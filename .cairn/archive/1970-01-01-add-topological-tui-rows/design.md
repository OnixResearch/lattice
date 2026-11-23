## Layout

Kahn traversal computes dependency depth with stable node identities as ties. Rows are sorted by depth then identity. A pending-node frontier reserves lanes until the corresponding row consumes them. The first new child inherits the consumed lane; further children use free lanes, and an already-reserved join keeps its lane. Up/down/left/right connector masks join the before/after frontier on each row. Disconnected roots reserve independent lanes. Invalid draft graphs retain every node in the fallback display without claiming cycle validity.

Visible lane count is bounded independently from logical frontier width. Overflow maps onto the final visible lane, marks hidden-lane nodes with `◆`, and reports the peak additional lane count as `+N`. This bounds rendering, not workflow cardinality or scheduler concurrency. A fixed header and aggregate footer surround a selection-following vertical viewport. Hit testing applies the same viewport offset and excludes both header and footer.

## Observations

`NodeProgress::Finished` carries the existing real `StepResult`. Producers publish only after product-session finish/admission postprocessing. The result is cloned only when a progress subscriber exists, because the executor's owned final results and the asynchronous observer have separate lifetimes; no additional Arc or allocation wrapper is introduced. The frontier's canonical/observed completion ordering remains unchanged.

The TUI records local start time and monotonic elapsed time on dispatch observation, and applies completed/failed/skipped node results before the run ends. The forwarding task is drained before the final run event to prevent late start events from reopening completed metadata. Final receipt processing persists/cache-updates as before without duplicating live output. New runs clear previous row summaries. Cancelling freezes observed duration at the local cancellation request without asserting remote effect termination.

## Verification boundary

Snapshot fixtures cover actual rendered topology/name columns. Deterministic reducer tests isolate live metadata and final-event deduplication; viewport/input tests cover shared node ownership and hidden-graph isolation. Maintainer verification must run after shared integration. The implementation did not run builds, tests, linters, formatters, or Cairn gates. `xd://lsp` was unavailable (device not mounted); exact-symbol grep located the `NodeProgress` producers and consumers before migration.
