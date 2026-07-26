## Phase 1: Baseline and source admission

- [ ] [serial] Run focused evaluator/configuration tests before core dependency changes and inventory every Nickel call site, source form, import path, contract, target decode, diagnostic, safe-message boundary, fallback, and store-opening order. r[lattice.nickel_eval_adoption.compatibility]
- [ ] [serial] Verify the shared repository's explicit license, provenance, canonical RID, authority policy, exact Git object, source archive, Nickel cohort, features, and no-fallback receipt. r[lattice.nickel_eval_adoption.source]
- [ ] [parallel] Reject unpublished, floating, sibling, GitHub, incompatible-version, duplicate-evaluator, source-drifted, secret-bearing, or incomplete producer sources. r[lattice.nickel_eval_adoption.source]

## Phase 2: Thin adapter and dual run

- [ ] [serial] Add a thin adapter that supplies explicit Lattice source, import, contract, and decoding facts while leaving path discovery, defaults, warnings, CLI behavior, and mutation order in Lattice. r[lattice.nickel_eval_adoption.boundary]
- [ ] [parallel] Dual-run positive app-config, template, workflow, orchestration definition/proposal, and doctor fixtures through legacy and shared evaluators. r[lattice.nickel_eval_adoption.compatibility]
- [ ] [parallel] Dual-run negative malformed source, missing import, contract violation, invalid export, unsupported version, oversized or sensitive diagnostic, source mismatch, and fallback fixtures. r[lattice.nickel_eval_adoption.compatibility]

## Phase 3: Exact source cutover

- [ ] [serial] Align Cargo and Nix on the same Radicle adapter and immutable revision and regenerate locks with owning tools without manually editing `flake.lock`. r[lattice.nickel_eval_adoption.cutover]
- [ ] [serial] Route production evaluation through the shared shell only after parity while retaining all Lattice product contracts, decoding, diagnostics policy, defaults, and authority. r[lattice.nickel_eval_adoption.boundary]
- [ ] [serial] Remove duplicate program construction and error mapping only after every inventoried call site is covered; record one VCS rollback for source, locks, and adapters. r[lattice.nickel_eval_adoption.cutover]

## Phase 4: Closeout

- [ ] [parallel] Update README references, dependency/source documentation, supported evaluator cohort, outage behavior, rollback, and non-claims. r[lattice.nickel_eval_adoption.source] r[lattice.nickel_eval_adoption.boundary]
- [ ] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, source checks, and bounded `nix flake check` before sync and archive. r[lattice.nickel_eval_adoption.compatibility] r[lattice.nickel_eval_adoption.cutover]
