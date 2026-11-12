# Contributing

Lattice is a Rust workspace with a pinned Nickel toolchain and a Cairn lifecycle.
Keep changes focused, preserve the existing ownership boundaries, and retain the
evidence that supports a claim.

## Development environment

Use the repository's Nix development shell. It supplies the pinned Rust and
Nickel toolchain, Cairn, Octet, and native build inputs.

```sh
nix develop
cargo run --locked -- --help
```

The default build and the public flake closure use only anonymously acquirable
sources. The optional Tile compose integration lives in the separate `compose/`
workspace and is not built by the default workspace or public checks. Building
it requires authorized access to the exact Tile revision declared in
[`compose/Cargo.toml`](compose/Cargo.toml).

## Checks

Run the maintained gates from the repository root:

```sh
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
```

`scripts/check.rs` runs formatting, boundary guards, Nickel policy checks,
example and builtin typechecks, focused checks, tests, Clippy, and specification
validation. Integration tests that invoke Cairn require `LATTICE_CAIRN_BIN`; the
development shell sets it to the pinned native binary. Running `cargo test`
outside the development shell fails closed instead of using an unrelated Cairn
from `PATH`.

For the optional compose workspace, run its own checks with authorized Tile
source access:

```sh
cargo test --manifest-path compose/Cargo.toml --locked
cargo clippy --manifest-path compose/Cargo.toml --all-targets --locked -- -D warnings
```

## Lifecycle

Accepted requirements and active changes live in [`.cairn/`](.cairn/). Use the
native Cairn lifecycle for behavior changes:

```sh
cairn status --root .
cairn gate proposal <change> --root . --policy cairn-policy/generated/cairn-policy.json
```

Do not edit generated policy or generated inventories by hand. Change the
reviewable Nickel source and regenerate it with the pinned producer.

## Source and evidence

- Preserve functional-core / imperative-shell boundaries. Pure cores must not
  read files, inspect the environment, spawn processes, print, access clocks,
  perform network I/O, or depend on ambient mutable state.
- Keep positive and negative fixtures together for validation, parsing, policy,
  and receipt behavior.
- Do not edit vendored snapshots as ordinary source. Use the repository-owned
  sync procedure and record the exact source revision.
- Do not use commit counts, task closures, archive counts, or generated-code
  volume as completion evidence.
- State the bounded claim and its non-claims. Local checks do not prove remote
  publication, production deployment, external effect success, or arbitrary
  workflow correctness.

## Licensing

New Lattice-owned source is licensed under AGPL-3.0-or-later. By contributing,
you agree that your contribution is provided under the same license. Imported
snapshots, historical MIT notices, and exact old dependency pins retain their
own terms.

## Security

Report vulnerabilities through the private process in [SECURITY.md](SECURITY.md).
Do not include plaintext credentials in Nickel profiles, fixtures, issues, or
pull requests.
