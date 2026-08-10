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

- migrated baseline file BLAKE3: `df8498f2e46e82d91d9d3af7d62150f5fa6ce59fe76fd237c1723c7726ae37bf`
- migrated baseline identity: `b3:437767e8ce3cd6ab1687a52dd8310143763970c67ade6b6ff30cd1142fc5e2fd`
- migration script BLAKE3: `7d3cdbf5dba15f6a0da09861d0f20fa8de5b9069359334e47598ca09a00f7899`

## Octet result

The final gate reported `warning-only` with 1,141 warning occurrences. It reported 890 accepted and current identities, zero new identities, and zero stale identities.

## Validation status

Focused expression, orchestration, TUI, and Nickel adapter tests passed before and after source changes. The orchestration test intentionally starts child test processes. One child rejects its negative phase while the parent test passes.

`scripts/check.rs` passed after these inherited validation repairs:

- the Execution Graph source checker now validates Choregraph's compatibility crate;
- the Choregraph negative fixture now reaches its intended contract rejection;
- the credential log-capture test uses a deterministic current-thread subscriber;
- parameterized Tokio test attributes retain local traceability binding;
- the scheduler crash fixture has a bounded 15-second wait; and
- stale reviewed hashes now bind the accepted Cargo and documentation bytes.

`cargo clippy --package lattice --all-targets --no-deps -- -D warnings` passed. The direct Octet gate passed with 1,141 findings, 890 accepted identities, zero new identities, and zero stale identities.

`nix flake check -L --option secret-key-files ''` passed all host checks. Nix omitted the three incompatible non-host systems.

Native Cairn validation remains blocked because Lattice still uses `cairn/` instead of `.cairn/`. The reported blocker is `cairn.lifecycle_directory.migration_required`. This update does not claim that the lifecycle-store migration is complete.

This evidence does not prove Octet correctness, Lattice correctness, semantic compatibility, release readiness, or whole-stack correctness.
