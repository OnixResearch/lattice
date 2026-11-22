# Export repaired candidates and report repair dispositions in inventory

Tracking: [Lattice #6](https://github.com/OnixResearch/lattice/issues/6), the local item "repaired candidates are not exported or shown in inventory" left open by `bound-same-attempt-repair`. Consumer contract (read-only): Kiln [`4899d149`](https://github.com/OnixResearch/kiln/tree/4899d149/crates/kiln-adapter-lattice/src/worker_inventory.rs) `kiln-adapter-lattice::worker_inventory`, the `lattice.worker-inventory.v3` reader. Owner-authorized round 2026-11-21; revised after the independent review of `2dff32d1` (`RepairExportReview`, seven findings).

## Why

Bounded same-attempt repair retains step intents, step records and `repair/disposition.json`. A `correct_candidate` or `repaired_candidate` disposition names a host-captured revision bound to attempt, fence, iteration and parent, and every declared check ran against a read-only copy of it. But `export-candidate` reads only the native attempt result, which a repair attempt never publishes, so a passing repaired candidate cannot be exported. `inspect` reads the same missing result, so every repair attempt stays `dispatch_recorded_outcome_unknown` with the `dispatch_outcome_unknown` wait reason even after its disposition is retained.

The review of `2dff32d1` found that the first version trusted single recorded fields: a step's `verdict`, a disposition's `kind`, a receipt's own `changed_paths`, a byte-only candidate identity, and a partial disposition. Export and terminal status must instead be derived from the complete, request-bound retained evidence, re-derived and cross-checked.

## What Changes

- The repair records become typed and complete: `lattice.implementation-attempt-repair-observation.v2` and `lattice.implementation-attempt-repair-step.v2` are written from and read back into `repair::retained` types that refuse unknown, missing and left-out nullable fields. A step record retains the integrity of every check's read-only copy; a revision retains its execute bits, which its identity (`lattice.implementation-attempt-candidate-revision.v2`) now binds, and the base digest of every changed path. Recovery still prints a v1 disposition; export and inventory never act on one. r[lattice.implementation_attempt.repair_export]
- `export-candidate` of a request with a repair policy exports the exact final candidate revision of a retained passing disposition, after the single-shot export's request, intent, output and revision-seam checks; a revoked managed revision is still `export_revoked`. It reads only the complete record set bound to this request's bytes, envelope, persisted execution binding, policy, lineage and dispatch plan, and replays it in the pure core: repairs within the admitted policy and the global bound, every step exited with the loop's parent and failure classes, every revision re-derived with a receipt bound to this work, attempt, fence, workspace and base, and every verdict re-derived from the check observations and copy integrity. The workspace must keep the checked execute bits, and the whole receipt must re-derive from it and the retained changed-path base digests. r[lattice.implementation_attempt.repair_export.scenario.passing]
- Three typed refusals with `dispatched:true`: `repair_outcome_unknown` (no retained disposition, including a crash between a step record and the disposition; never guessed), `repair_not_candidate` (any other disposition) and `repair_disposition_invalid` (malformed, incomplete, first-schema, crossed or contradicted records). Single-shot attempts export exactly as before. r[lattice.implementation_attempt.repair_export.scenario.refused]
- `inspect` reports a repair disposition as `terminal_observed` with its `kind`, within `lattice.worker-inventory.v3`, only when the complete records agree with the kind and show every started worker exited or cancelled. `uncertain` (`worker_outcome`), a step intent that may have run unrecorded, a missing or first-schema disposition stay `dispatch_recorded_outcome_unknown`; a malformed, crossed or contradicted record set is an error. r[lattice.implementation_attempt.repair_export.scenario.inventory]

## Impact

- **Pure core:** new `src/implementation_attempt/repair/{exportable,retained}.rs` and their tests; `repair/revision.rs` gains `Record`, `Executables` and the v2 identity; new `src/candidate/digests.rs` re-derives a produced receipt from file digests, sharing the tree and diff identity formulas in `candidate/snapshots.rs`; `candidate::Bindings` becomes `Copy`.
- **Shell:** new `cli/export/repaired.rs` and `cli/repair/evidence.rs`; `cli/export.rs` (branch on the repair policy), `cli/export/admission.rs` (request identity); `cli/repair/{mod,records,steps}.rs` and `cli/repair/steps/{candidate,checks}.rs` (typed records, copy integrity, execute bits, base digests, the evidence reader, the inventory label); `cli/inspect/mod.rs` (repair branch); `cli/refusal.rs` (three reasons; its unit tests move unchanged to `cli/refusal/tests.rs` to keep the file under 300 lines).
- **Tests:** `src/implementation_attempt/repair/{exportable,retained}/tests.rs`, `src/candidate/digests/tests.rs`, `src/implementation_attempt/cli/tests/repair/{export,forged}.rs`; `cli/tests/repair.rs` expects `terminal_observed` and the v2 schema.
- **Docs:** `docs/candidate-worker.md`, `docs/worker-inventory.md`, changelog.
- **Compatibility:** request and intent bytes and single-shot export are unchanged. The repair disposition and step records move to v2 and revision identities to the v2 domain; recovery still prints v1 dispositions. The refusal schema gains three reasons, which Kiln `4899d149` retains like any reason outside its no-start list. Inventory keeps its schema and enumerations; only `disposition`, an open token in Kiln's reader, takes the repair kinds.

## Dependencies

- Local: `bound-same-attempt-repair` (repair records), `publish-worker-inventory-v3` (the inventory contract).
- Consumer: Kiln `4899d149` reads `terminal_observed.disposition` as an unvalidated string, requires `exited` liveness and no wait reasons for it, and retains refusals other than its four no-start reasons.

## Out of Scope

- A `lattice.worker-inventory.v4`: v3 carries the disposition kind; the revision, repair count, bound, cause and defect stay in the retained record that `recover` prints.
- Protection against an actor who can rewrite the private state directory and forge a fully consistent record set.
- Kiln-side changes, an OMP-backed repair worker port and live qualification.
