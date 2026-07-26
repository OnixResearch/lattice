## Phase 1: Baseline and producer admission

- [ ] [serial] Run the smallest relevant graph tests before core dependency changes and record current vendor bytes, Seaglass source revision, sync metadata, adapter entry points, Cargo/Nix sources, and rollback state. r[lattice.execution_graph_external_adoption.compatibility]
- [ ] [serial] Verify the producer's lifecycle, license, canonical RID, authority policy, reviewed Git object, source archive BLAKE3, endpoint, positive acquisition, and negative no-fallback receipt. r[lattice.execution_graph_external_adoption.source]
- [ ] [parallel] Reject unpublished, floating, sibling, GitHub, license-incompatible, source-drifted, duplicate-package, or incomplete producer sources before lock mutation. r[lattice.execution_graph_external_adoption.source]

## Phase 2: Dual-run adapter

- [ ] [serial] Add the exact Radicle-backed package under a temporary compatibility alias while retaining the vendor path for tests only. r[lattice.execution_graph_external_adoption.compatibility]
- [ ] [parallel] Dual-run admission diagnostics, ordering, dependency/dependent queries, parallel groups, readiness, lifecycle, resume, critical path, and bottleneck fixtures. r[lattice.execution_graph_external_adoption.compatibility]
- [ ] [parallel] Add negative malformed graph, cycle, missing endpoint, terminal mutation, source mismatch, revision mismatch, and duplicate-package fixtures. r[lattice.execution_graph_external_adoption.compatibility]

## Phase 3: Source cutover

- [ ] [serial] Align Cargo and Nix on the same Radicle adapter and immutable revision, regenerate locks with owning tools, and verify no GitHub, sibling, or floating fallback exists. r[lattice.execution_graph_external_adoption.cutover]
- [ ] [serial] Route production graph decisions only through the external core while retaining all Lattice product types, policies, effects, persistence, and evidence. r[lattice.execution_graph_external_adoption.authority]
- [ ] [serial] Remove vendor snapshots, source-sync tooling, and temporary dual-run code only after all compatibility checks pass; retain VCS rollback instructions. r[lattice.execution_graph_external_adoption.cutover]

## Phase 4: Closeout

- [ ] [parallel] Update README references, source documentation, dependency policy, and non-claims with the canonical RID and exact revision. r[lattice.execution_graph_external_adoption.source] r[lattice.execution_graph_external_adoption.authority]
- [ ] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, repository source checks, and bounded `nix flake check` before sync and archive. r[lattice.execution_graph_external_adoption.compatibility] r[lattice.execution_graph_external_adoption.cutover]
