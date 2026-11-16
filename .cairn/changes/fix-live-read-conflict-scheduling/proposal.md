## Why

Issue #19 identifies that live `File(Read)` and `ImageInput` nodes were classified as pure. With two or more slots, the product planner could dispatch these observations of mutable host state alongside an unresolved shell mutation. Read-only authority is not evidence that effects commute.

## What Changes

- Classify live file/image reads as unresolved conflict participants by default, retaining their existing resource lanes.
- Retain pure `Condition` and `Merge` computation, and the existing explicitly admitted resource-alias refinement.
- Add product-candidate and active-reservation regressions in both orderings, plus FIFO-barrier executor fixtures delegating to real capability-rooted file/image adapters.
- Cover admitted aliases, expression dependencies, cancellation, and existing capacity/lane accounting without changing authority or lifecycle behavior.

## Impact

- **Files:** `src/engine/frontier.rs`, the ready-frontier integration test and its live-read fixture, `docs/bounded-ready-frontier.md`, and this native Cairn change.
- **Scope:** ordering effects within one admitted workflow; no exclusion of unrelated external writers and no immutable snapshot claim.
- **Non-goals:** path-derived independence, a new scheduler or graph IR, authority changes, lifecycle changes, retries, external-effect guarantees, acceptance, or archive.
- **Verification:** regression code is supplied; no build, test, formatter, lint, or Cairn gate execution is claimed by this implementation handoff.
