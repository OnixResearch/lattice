## Phase 1: Intake and baseline

- [ ] [serial] [task:reference-intake] Record the three Jane Street revisions, MIT notices, selected mechanisms, and rejected runtime dependencies in `README.md`. r[lattice.incremental_pilot.boundary]
- [ ] [serial] [task:baseline] Record current full-recomputation and bounded node-result-cache behavior over one identity-bound corpus. r[lattice.incremental_pilot.baseline]
- [ ] [serial] [task:pilot-policy] Add typed Nickel limits and accept, reject, or insufficient-evidence policy. r[lattice.incremental_pilot.measurement]

## Phase 2: Pure pilot oracle

- [ ] [serial] [task:projection] Implement a test-only projection over explicit graph, dependency, operation, input, capability, and policy facts. r[lattice.incremental_pilot.projection]
- [ ] [serial] [task:invalidation] [after:projection] Implement bounded dependency invalidation and affected-node planning. r[lattice.incremental_pilot.candidate]
- [ ] [serial] [task:full-oracle] Implement an independent complete-recomputation comparator. r[lattice.incremental_pilot.oracle]

## Phase 3: Corpus and measurements

- [ ] [parallel] [task:positive-corpus] [after:invalidation] Add chain, diamond, fan-out, fan-in, deep, sparse, multi-source, and no-change cases. r[lattice.incremental_pilot.corpus]
- [ ] [parallel] [task:negative-corpus] [after:full-oracle] Add cycle, missing-node, stale, crossed-authority, hidden-effect, overflow, and divergence cases. r[lattice.incremental_pilot.corpus]
- [ ] [serial] [task:measurement-shell] [after:positive-corpus] Record bounded structural and shell-owned timing or resource observations. r[lattice.incremental_pilot.measurement]

## Phase 4: Decision and gates

- [ ] [serial] [task:pilot-decision] [after:negative-corpus] Apply the typed policy and record adopt-candidate, reject-candidate, or insufficient-evidence. r[lattice.incremental_pilot.decision]
- [ ] [serial] [task:gates] [after:pilot-decision] Run focused tests, Clippy, Octet, Cairn, and Nix gates. r[lattice.incremental_pilot.verification]
- [ ] [serial] [task:follow-up-boundary] Record that production adoption requires a separate accepted Cairn. r[lattice.incremental_pilot.boundary]
