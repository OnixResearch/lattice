## Phase 1: Lifecycle and baseline

- [x] [serial] Record the overwrite failure, bounded create-only contract, and implementation design. r[lattice.config.init_no_clobber]
- [x] [serial] Run the focused initialization suite before changing core behavior and record the environment result. r[lattice.config.init_no_clobber]

## Phase 2: Implementation

- [x] [serial] Replace truncating config initialization with exclusive create-new file creation and explicit conflict diagnostics. r[lattice.config.init_no_clobber]
- [x] [serial] Document that `--init` creates a fresh target and refuses an existing config. r[lattice.config.init_no_clobber]

## Phase 3: Verification and closeout

- [x] [serial] Add positive and negative CLI tests proving fresh initialization succeeds while repeated initialization preserves config and template bytes. r[lattice.config.init_no_clobber]
- [x] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix validation; sync the accepted requirement and archive the evidence. r[lattice.config.init_no_clobber]

## Verification evidence

- Post-change `nix develop -c cargo test --test test_template_parameters` passed 12 tests, including fresh creation and exact-byte preservation on rejection.
- `scripts/check.rs` passed the default-member test suite, strict project Clippy, and Cairn spec references.
- `octet-precommit-check` retained 919 accepted/current findings with zero new and zero stale findings; baseline hash remained `b3:b7b84338e5e0a9ba1d6ff41108e196fcb82345a2940a232b70f0a8d5d9aa034e`.
- Cairn validation plus proposal, design, and tasks gates passed through the canonical sibling Cairn CLI.
- `nix flake check -L --option secret-key-files ''` passed all bounded checks on `x86_64-linux`.
- Direct `cargo fmt --check` and broad host-style Clippy traverse vendored Rats sources and report pre-existing formatting/wildcard-import debt; repository-owned formatting, Clippy, Octet, and Nix rails passed.

## Baseline evidence

- Direct host Cargo was unavailable (`cargo: command not found`), so the repository dev shell was used as required by local guidance.
- `nix develop -c cargo test --test test_template_parameters` passed all 11 pre-change tests.
