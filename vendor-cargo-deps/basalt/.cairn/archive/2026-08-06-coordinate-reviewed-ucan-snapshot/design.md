# Design: Admit exact reviewed UCAN snapshot revisions

## Context

`tools/sync-vendored-ucan.rs` archives `SOURCE_REVISION` from an explicit source repository. The fixed value protects the reviewed snapshot but prevents a BLAKE3-bound external plan from supplying its exact candidate.

The current reviewed revision is not an ancestor of the local UCAN `main`. The pilot must retain the reviewed revision and reject the diverged branch.

## Decisions

### Decision: Accept only full Git object IDs

**Choice:** Add `--check-source-revision SOURCE REVISION` and `--import-revision SOURCE REVISION`.

The pure argument core accepts full lowercase SHA-1 or SHA-256 Git object IDs. It rejects branch names, abbreviated IDs, uppercase IDs, and malformed values.

**Rationale:** An exact object ID can bind a reviewed external plan. A moving reference cannot provide that binding.

### Decision: Keep compatibility commands

**Choice:** Keep `--check-source SOURCE` and `--import SOURCE` bound to the current `SOURCE_REVISION` constant.

**Rationale:** Existing operator procedures remain valid while the coordinator uses the new exact forms.

### Decision: Make snapshot checks use the recorded revision

**Choice:** `--check` reads and validates `vendor/ucan-revision.txt`. Manifest reconstruction uses that recorded revision.

**Rationale:** A successful exact import must remain checkable without rebuilding the importer with a new constant.

## Functional core and imperative shell

The pure model parses modes and validates exact revision text. The shell reads files and runs Git, tar, and BLAKE3 commands.

## Risks / Trade-offs

- A full object ID identifies source bytes but does not approve them.
- The importer can change many vendored files. The coordinator must restrict changed paths and run consumer checks.
- Git SHA-1 remains on this boundary because the existing Git repository uses SHA-1 object IDs. BLAKE3 binds plans, manifests, and receipts.
