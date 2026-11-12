## Why

Lattice's default source tree still required authenticated SSH access to a
private Tile repository and several public repositories declared over SSH. The
repository also lacked a public security, contribution, deployment, release, and
launch surface. A public release must build anonymously, state its trust
boundary, and give an operator a repeatable installation path without disclosing
private source or claiming production readiness.

## What Changes

- Move the Tile compose integration into a separate `compose/` workspace that is
  excluded from the default workspace, public checks, and release artifact.
- Preserve exact source revisions while acquiring Artifact, Trellis, Valence,
  Cairn, and Octet over anonymous HTTPS, including root Cargo patches and flake
  input overrides.
- Add public Cargo package metadata and `lattice --version`.
- Add the security policy, contribution guide, changelog, documentation index,
  server deployment guide, systemd unit, release process, and tagged release
  workflow.
- Add a launch plan with positioning, proof points, demo script, checklist, and
  explicit non-claims.
- Keep private Tile source out of the public closure and out of the release
  artifact.

## Impact

- **Files**: root manifests and lockfiles, `flake.nix`, `flake.lock`, `src/lib.rs`,
  `src/main.rs`, `compose/`, release and operations documentation, deployment
  assets, release workflow, Cairn policy and release input manifest.
- **Testing**: `cargo metadata --locked`, default `cargo check`, compose workspace
  tests with authorized Tile access, policy export, flake evaluation, CLI
  `--version`/`--help`, and the maintained repository gates.
- **Authority**: this change authorizes local release preparation only. It does
  not publish a tag, publish a release, disclose private Tile source, deploy a
  service, or claim production readiness.
