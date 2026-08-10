# Tasks: Maintain the project identity audit

## 1. Establish bounded evidence

- [x] [serial] Capture the fail-closed baseline and confirm the exact two missing project-identity scenarios. r[lattice.project_identity.canonical.scenario.stale-branding] r[lattice.project_identity.stable-identities.scenario.cairn-history]
- [x] [serial] Audit the archived checker and reject broad file exemptions and unverified Cairn-history claims. r[lattice.project_identity.stable-identities.scenario.cairn-history]

## 2. Implement maintained checks

- [x] [serial] Add a pure stale-branding classifier and deterministic BLAKE3 manifest core. r[lattice.project_identity.canonical.scenario.stale-branding]
- [x] [serial] Add a thin sorted filesystem shell over bounded public and historical roots. r[lattice.project_identity.stable-identities.scenario.cairn-history]
- [x] [serial] Add live positive tests and negative branding, marker-bypass, accepted-ID mutation, and archive-mutation tests. r[lattice.project_identity.canonical.scenario.stale-branding] r[lattice.project_identity.stable-identities.scenario.cairn-history]
- [x] [serial] Retain bounded public audit inputs and the exact rename archive in the isolated Nix source closure. r[lattice.project_identity.stable-identities.scenario.cairn-history]

## 3. Validate and archive

- [x] [serial] Prove the exact non-target missing set is unchanged, target missing IDs close, dangling remains zero, and coverage stays unsuccessful. r[lattice.project_identity.canonical.scenario.stale-branding] r[lattice.project_identity.stable-identities.scenario.cairn-history]
- [x] [serial] Run focused tests, policy freshness, repository checks, bounded Nix checks, and Cairn gates. r[lattice.project_identity.canonical]
- [x] [serial] Sync strengthened scenarios, archive the completed change, and commit incrementally. r[lattice.project_identity.stable-identities]
