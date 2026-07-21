## Phase 1: Baseline and policy ownership

- [x] [serial] Capture the zero-reference fallback receipt and verify the configured roots do not match Lattice's owned source layout. r[lattice.repository_validation.traceability_sources]
- [x] [serial] Add typed Nickel policy ownership and bounded Lattice requirement/evidence roots. r[lattice.repository_validation.traceability_sources.scenario.discovery]
- [x] [serial] Regenerate the checked policy JSON and prove a second export is fresh. r[lattice.repository_validation.traceability_sources.scenario.discovery]
- [x] [serial] Retain the checked Nickel policy and negative fixture in the isolated Nix source closure. r[lattice.repository_validation.traceability_sources.scenario.discovery]

## Phase 2: Positive and negative verification

- [x] [serial] Add a positive test for the exact default profile roots and source kinds. r[lattice.repository_validation.traceability_sources.scenario.discovery]
- [x] [serial] Add negative checks that exclude stale fallback roots, preserve marker verbs, and reject an escaping source root. r[lattice.repository_validation.traceability_sources.scenario.unsafe-root]
- [x] [serial] Run corrected source discovery and retain missing/dangling findings as failures rather than suppressing them. r[lattice.repository_validation.traceability_sources.scenario.fail-closed]

## Phase 3: Validation and lifecycle

- [x] [serial] Run repository checks, Cairn gates/validation, policy freshness, and bounded Nix checks; record evidence. r[lattice.repository_validation.traceability_sources]
- [x] [serial] Sync the accepted requirement, archive the completed change, and commit incrementally. r[lattice.repository_validation.traceability_sources]
