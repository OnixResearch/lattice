# Tasks: Adopt Nickel 1.17 for policy authoring

## Pin and freshness

- [x] [serial] Add an exact Nickel `1.17.0` source input at commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`. r[basalt.nickel_toolchain.cohort]
- [x] [serial] Replace policy, fixture, and developer-tool uses of the older package-set Nickel. r[basalt.nickel_toolchain.cohort]
- [x] [serial] Regenerate `flake.lock` only with a Nix lock command. r[basalt.nickel_toolchain.cohort]
- [x] [parallel] Add a guard for older, floating, ambient, or mixed evaluators. r[basalt.nickel_toolchain.cohort]
- [x] [serial] Regenerate checked-in policy artifacts and run the freshness rail. r[basalt.nickel_toolchain.freshness]

## Compatibility and authority boundary

- [x] [parallel] Run valid policy, contract, merge, import, and export fixtures. r[basalt.nickel_toolchain.compatibility]
- [x] [parallel] Add unknown, duplicate, malformed, missing, type-mismatch, import, and contract negative fixtures. r[basalt.nickel_toolchain.compatibility]
- [x] [parallel] Prove successful export does not bypass Rust authority enforcement. r[basalt.nickel_toolchain.boundary]

## Validation

- [x] [serial] Record the exact evaluator identity in policy release evidence. r[basalt.nickel_toolchain.validation]
- [x] [serial] Run policy freshness, focused tests, formatting, Clippy, release verification, Cairn gates, and relevant Nix checks. r[basalt.nickel_toolchain.validation]
