## Phase 1: Lifecycle and baseline

- [x] [serial] Record the overwrite failure, bounded create-only contract, and implementation design. r[lattice.config.init_no_clobber]
- [x] [serial] Run the focused initialization suite before changing core behavior and record the environment result. r[lattice.config.init_no_clobber]

## Phase 2: Implementation

- [x] [serial] Replace truncating config initialization with exclusive create-new file creation and explicit conflict diagnostics. r[lattice.config.init_no_clobber]
- [x] [serial] Document that `--init` creates a fresh target and refuses an existing config. r[lattice.config.init_no_clobber]

## Phase 3: Verification and closeout

- [x] [serial] Add positive and negative CLI tests proving fresh initialization succeeds while repeated initialization preserves config and template bytes. r[lattice.config.init_no_clobber]
- [ ] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix validation; sync the accepted requirement and archive the evidence. r[lattice.config.init_no_clobber]

## Baseline evidence

- Direct host Cargo was unavailable (`cargo: command not found`), so the repository dev shell was used as required by local guidance.
- `nix develop -c cargo test --test test_template_parameters` passed all 11 pre-change tests.
