## Phase 1: Canonical identity

- [x] [serial] Rename Cargo, crate, executable, Nix, scripts, current documentation, examples, tests, and user-visible runtime surfaces to Lattice, with positive command/package assertions and a negative stale-brand audit. r[lattice.project_identity.canonical]

## Phase 2: Compatibility core

- [x] [serial] Add pure canonical/legacy data-directory and store-file selection with shell-owned environment/filesystem facts and positive/negative precedence tests. r[lattice.project_identity.compatibility]
- [x] [serial] Preserve the versioned webhook domain, existing Cairn IDs/evidence markers, and historical archives; record an allowlisted residual-identity audit. r[lattice.project_identity.stable-identities]

## Phase 3: Verification and lifecycle

- [x] [serial] Run repository checks, strict Clippy, Cairn gates/validation/sync, and bounded Nix validation; record evidence and archive only after every gate passes. r[lattice.project_identity.canonical] r[lattice.project_identity.compatibility] r[lattice.project_identity.stable-identities]
