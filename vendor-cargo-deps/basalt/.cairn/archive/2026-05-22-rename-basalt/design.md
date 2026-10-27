# Design: Basalt rename

## Public identity

`basalt` becomes the canonical package, library crate, and CLI binary name. Documentation should introduce it as:

> Basalt, formerly `ucan-nickel-contracts`, provides hardened UCAN contract boundaries for Nickel and Steel policy backends.

The package description should remain descriptive enough for Cargo/registry tooling, mentioning UCAN, Nickel, Steel, and policy contracts rather than relying on the metaphor alone.

## Compatibility stance

This is a pre-release crate at `0.1.0`, so the rename may update Rust import paths and the CLI binary directly. The policy schema version string remains `ucan-nickel-contracts.policy.v1` in this slice because the JSON policy schema has not changed; changing it would create unnecessary migration work for existing policy artifacts.

The old name may remain only where intentional:

- historical Cairn archives;
- explicit "formerly `ucan-nickel-contracts`" discoverability text;
- policy schema identifiers that are intentionally stable for artifact compatibility.

## Surfaces to update

- `Cargo.toml`: package name, repository/homepage/documentation URLs if they point at the canonical repo name, and binary name.
- `Cargo.lock`: package entry name.
- `src/main.rs`, integration tests, doctests, and README examples: import `basalt` and use the `basalt` binary.
- `flake.nix`: description, check derivation names, compiled verifier path names.
- `tools/release-verify.rs`: temp target directories, receipt wording if name-specific, and package-content checks as needed.
- `README.md`: title, tagline, CLI instructions, target dirs, and former-name note.

## Verification

The rename is complete when formatting, no-default tests, CLI-feature tests, doctests, clippy for both feature sets, policy freshness, `nix flake check -L`, Cairn validation/gates, and stale-name classification all pass.
