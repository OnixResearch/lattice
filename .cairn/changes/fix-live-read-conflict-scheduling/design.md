## Classification boundary

`frontier::candidate` and `frontier::reservation` both obtain their default conflict facts from the same product classifier. Only in-memory `Condition` and `Merge` remain `Pure`. Live `File(Read)` and `ImageInput` fall through to `Unresolved`, like other effects that lack admitted conflict keys. Resource lanes are unchanged. The runtime's existing admitted alias override remains the only refinement exercised by this change; a path or a read-only grant is not such a fact.

The pure core planner and production dispatch loop are intentionally unchanged. Their existing capacity, readiness, reservation, cancellation, canonical result presentation, and observed completion policies remain authoritative.

## Deterministic regression boundary

The product regressions construct real `NodeConfig` values and use `frontier::candidate`, `frontier::reservation`, and `frontier::active_reservations`. Each file/image case runs mutation-first and read-first with two slots and every named lane. No hand-supplied alias hides the missing-conflict regression. Separate cases retain pure overlap and isolate lane, total-capacity, and cancellation blockers.

The Unix executor fixture creates a temporary capability root and two named pipes. A real asynchronous shell subprocess writes an intermediate live file, signals barrier entry, and waits for an explicit release. Only then does it install the final bytes. Observing adapter wrappers record starts and bytes while delegating unchanged to `LocalFs` and `LocalImageIo`; they do not emulate file or image I/O. While the mutation is held, the test polls all currently runnable selected work to `Pending`, snapshots whether a read started, then releases the subprocess. Deadlines bound failures only; neither sleeps nor elapsed-time comparisons determine correctness. A drop guard releases blocked children during fixture teardown.

The same fixture separately exercises an admitted same-resource alias and an explicit output-reference dependency. A cancellation case sets the existing cancellation flag while the mutation is held and checks that the pending read never enters its real adapter. Successful cases require final bytes, completed nodes, canonical results, and mutation-before-read observation order. Existing independent-effect ordering tests remain in the parent integration test.

## Non-claims

This correction orders workflow effects, not all host access. Unrelated external processes may still modify files. A successful read is not an immutable admitted snapshot. More precise independence would require admitted evidence covering resolved expressions, aliases, overlapping roots, and relevant host identity rules.

No tests, builds, formatters, linters, or Cairn gates were run by the implementation worker. Integrated verification belongs to the integration owner.
