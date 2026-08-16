# Tasks: Adopt the Nickel 1.17 workflow evaluator cohort

## Producer and dependency pins

- [ ] [serial] Verify the archived `nickel-eval` cohort change and record its immutable Radicle object. r[lattice.nickel_toolchain.cohort]
- [ ] [depends:lattice.nickel_toolchain.cohort] [serial] Update the `nickel-eval` dependency to that exact object. r[lattice.nickel_toolchain.cohort]
- [ ] [serial] Pin direct fixtures to `nickel-lang-core 0.18.0`. r[lattice.nickel_toolchain.fixture_cohort]
- [ ] [serial] Add an exact Nickel CLI `1.17.0` source input and regenerate lockfiles only with Cargo and Nix. r[lattice.nickel_toolchain.fixture_cohort]
- [ ] [parallel] Add a guard for old, mixed, floating, sibling-path, or fallback evaluator sources. r[lattice.nickel_toolchain.fixture_cohort]

## Workflow boundary and compatibility

- [ ] [serial] Adapt evaluator results at the Lattice workflow import boundary. r[lattice.nickel_toolchain.boundary]
- [ ] [parallel] Prove evaluation and decoding complete before Choregraph admission and effects. r[lattice.nickel_toolchain.boundary]
- [ ] [parallel] Run valid workflow, import, example, and Choregraph handoff fixtures. r[lattice.nickel_toolchain.compatibility]
- [ ] [parallel] Add malformed, missing-import, contract, unknown-field, excessive-bound, and cohort-substitution fixtures. r[lattice.nickel_toolchain.compatibility]

## Validation

- [ ] [serial] Update evaluator source documentation and bounded evidence. r[lattice.nickel_toolchain.validation]
- [ ] [serial] Run focused import tests, operator checks, formatting, Clippy, Octet, Cairn gates, and relevant Nix checks. r[lattice.nickel_toolchain.validation]
