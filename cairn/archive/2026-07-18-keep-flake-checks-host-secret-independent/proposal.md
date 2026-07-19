## Why

Repository validation currently inherits the host daemon's `secret-key-files` setting. When the configured machine key is unavailable, `nix flake check` fails before evaluating project quality. The same check also formats vendored snapshots and previously omitted required Rats source whose basename matched an unanchored root ignore rule, so local Cargo success did not prove a reproducible Nix source snapshot.

## What Changes

- Document a check-scoped Nix invocation that clears `secret-key-files` without changing normal build or release signing behavior.
- Anchor project output ignores at the repository root so valid vendored source names are not hidden.
- Track the required Rats routing and coverage source files in the vendored provenance snapshot.
- Include tracked non-Rust integration fixtures in the Nix source closure.
- Restrict Crane format and Clippy checks to the repository-owned `n8nclone` package rather than enforcing consumer policy on imported dependency snapshots.

## Scope

This change owns repository validation and source-snapshot reproducibility. It does not modify host NixOS configuration, provision signing keys, disable normal Nix signing, reformat vendored dependencies, or change runtime workflow behavior.
