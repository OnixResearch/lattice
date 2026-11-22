# Export repaired candidates and report repair dispositions in inventory

Tracking: [Lattice #6](https://github.com/OnixResearch/lattice/issues/6), the local item "repaired candidates are not exported or shown in inventory" left open by `bound-same-attempt-repair`. Consumer contract (read-only): Kiln [`4899d149`](https://github.com/OnixResearch/kiln/tree/4899d149/crates/kiln-adapter-lattice/src/worker_inventory.rs) `kiln-adapter-lattice::worker_inventory`, the `lattice.worker-inventory.v3` reader. Owner-authorized round 2026-11-21.

## Why

Bounded same-attempt repair retains step intents, step records and `repair/disposition.json`. A `correct_candidate` or `repaired_candidate` disposition names a host-captured revision bound to attempt, fence, iteration and parent, and every declared check ran against a read-only copy of it. But `export-candidate` reads only the native attempt result, which a repair attempt never publishes, so a passing repaired candidate cannot be exported. `inspect` reads the same missing result, so every repair attempt stays `dispatch_recorded_outcome_unknown` with the `dispatch_outcome_unknown` wait reason even after its disposition is retained.

## What Changes

- `export-candidate` of a request with a repair policy exports the exact final candidate revision of a retained passing disposition, after the single-shot export's request, intent, output and revision-seam checks; a revoked managed revision is still `export_revoked`. The named revision must be the last recorded one at the iteration its kind implies, every recorded revision must re-derive from this attempt and fence through its parent chain, the record must name this envelope and execution binding, the final receipt must bind this work, attempt, fence, workspace and base, its step record must record a pass, and the workspace must still capture to the receipt. r[lattice.implementation_attempt.repair_export] r[lattice.implementation_attempt.repair_export.scenario.passing]
- Three typed refusals with `dispatched:true`: `repair_outcome_unknown` (no retained disposition, including a crash between a step record and the disposition; never guessed), `repair_not_candidate` (any other disposition) and `repair_disposition_invalid` (malformed, crossed or contradicted records). Single-shot attempts export exactly as before. r[lattice.implementation_attempt.repair_export.scenario.refused]
- `inspect` reports a retained repair disposition as `terminal_observed` with the disposition's `kind`, within `lattice.worker-inventory.v3`; without one the instance stays `dispatch_recorded_outcome_unknown`, and a malformed or crossed record is an error. r[lattice.implementation_attempt.repair_export.scenario.inventory]

## Impact

- **Pure core:** new `src/implementation_attempt/repair/exportable.rs` (the passing revision of a disposition) and its tests; `repair/revision.rs` gains `Record`, the recorded revision the shell previously declared privately in `cli/repair/steps.rs`.
- **Shell:** new `src/implementation_attempt/cli/export/repaired.rs`; `cli/export.rs` (branch on the repair policy), `cli/export/admission.rs` (request identity), `cli/export/content.rs` (two helpers shared); `cli/repair/{mod,records}.rs` (typed, read-only disposition and step-record reads, the inventory label); `cli/repair/steps.rs` (uses `revision::Record`); `cli/inspect/mod.rs` (repair branch); `cli/refusal.rs` (three reasons; its unit tests move unchanged to `cli/refusal/tests.rs` to keep the file under 300 lines).
- **Tests:** `src/implementation_attempt/repair/exportable/tests.rs`, `src/implementation_attempt/cli/tests/repair/export.rs`; the inventory expectation in `cli/tests/repair.rs` changes from unknown to `terminal_observed`.
- **Docs:** `docs/candidate-worker.md`, `docs/worker-inventory.md`, changelog.
- **Compatibility:** request, intent, disposition and export bytes are unchanged; the refusal schema gains three reasons, which Kiln `4899d149` retains like any reason outside its no-start list. Inventory keeps its schema and enumerations; only `disposition`, an open token in Kiln's reader, takes the repair kinds.

## Dependencies

- Local: `bound-same-attempt-repair` (repair records), `publish-worker-inventory-v3` (the inventory contract).
- Consumer: Kiln `4899d149` reads `terminal_observed.disposition` as an unvalidated string, requires `exited` liveness and no wait reasons for it, and retains refusals other than its four no-start reasons.

## Out of Scope

- A `lattice.worker-inventory.v4`: v3 carries the disposition kind faithfully; the revision, repair count, bound, cause and defect stay in the retained record that `recover` prints.
- Kiln-side changes, an OMP-backed repair worker port and live qualification.
