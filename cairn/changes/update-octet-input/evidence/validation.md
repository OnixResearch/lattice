# Octet input update evidence

## Dependency plan

- relationship: `octet-to-lattice-nix`
- dependency plane: `validation`
- old Octet revision: `493eb43402e5f6ece57cbeec0893c84a0a008200`
- new Octet revision: `aa3fb4a2483439f332418aa1df3cfda04158f8fd`
- plan BLAKE3: `7076e99af7d400cc362bb2a9fc6b749fdb0b5282af9f56889cb6a019733378c7`

The coordinator changed only `flake.lock`. Its adapter and revision postcondition passed. The first Nix check failed before baseline migration, as expected.

Apply receipt BLAKE3: `fc1c2c9ebdc7eedd89fd0b754daa70d6e72ca358d1a127539d627fc37bb11367`.

## Semantic migration

The initial candidate contained 1,156 warning occurrences. The migration reported 39 old groups and 44 candidate groups before the new normalization.

The final source changes removed all semantic occurrence growth. They also moved the Nickel adapter into `src/nickel/mod.rs`, qualified imports, typed expression port identities, and reduced ambiguous facade parameters.

The accepted transition contains:

- 1,140 old occurrences;
- 1,141 candidate occurrences;
- 18 exact old-only semantic groups with 25 occurrences;
- 17 exact candidate-only semantic groups with 26 occurrences; and
- zero semantic occurrence-growth groups.

The old-only numeric groups were false positives over nonnumeric values. The new groups are exact reviewed debt from type-aware `ambiguous_params` and `numeric_units` detection.

- migrated baseline BLAKE3: `80a3b588cc95f3f752fc7fb7cb6cbc99259fa87b9f6ad6ebea449ca51fe9b7e8`
- migration script BLAKE3: `7d3cdbf5dba15f6a0da09861d0f20fa8de5b9069359334e47598ca09a00f7899`

## Octet result

The final gate reported `warning-only` with 1,141 warning occurrences. It reported 890 accepted and current identities, zero new identities, and zero stale identities.

## Validation status

Focused expression, orchestration, TUI, and Nickel adapter tests passed before and after source changes. The orchestration test intentionally starts child test processes. One child rejects its negative phase while the parent test passes.

`scripts/check.rs` reached the full test suite. It stopped at three inherited `test_execution_graph_source_admission` failures. Those tests still require the retired Execution Graph source after Lattice adopted Choregraph.

The full Nix rail reached ten checks. Its first run found the untracked nested Nickel module, which staging corrected. The targeted Nix format check then passed.

The full rail also found an inherited invalid Nickel merge in `negative/choregraph-floating-source.ncl`. The targeted Nix Octet derivation printed 1,141 findings and still failed. The direct gate passed with the same occurrence count, so this sandbox difference remains open.

Native Cairn validation remains blocked because Lattice still uses `cairn/` instead of `.cairn/`. The reported blocker is `cairn.lifecycle_directory.migration_required`. This update does not claim that the lifecycle-store migration is complete.

This evidence does not prove Octet correctness, Lattice correctness, semantic compatibility, release readiness, or whole-stack correctness.
