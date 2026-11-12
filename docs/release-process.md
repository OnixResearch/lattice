# Release process

Lattice is pre-1.0. Releases use Semantic Versioning and the `vMAJOR.MINOR.PATCH`
tag form. A release is a tagged source revision plus a Nix-built binary archive;
it is not a hosted service or a production-readiness claim.

## Preconditions

1. The release commit is on `main` and the `check` workflow passed for that
   commit.
2. `Cargo.toml` and every workspace package that carries a version use the same
   release version.
3. `CHANGELOG.md` has an entry for the version with an accurate date.
4. The default build closure contains no authenticated source. The optional
   compose workspace is separate and is not part of the release artifact.
5. `SECURITY.md`, `CONTRIBUTING.md`, and the deployment guide describe the
   current trust boundary.
6. The release notes state the non-claims: local Unix service, no TLS or remote
   authentication, no sandbox for workflows, no hosted support commitment.

## Build and verify

Run the maintained gates on the exact release commit:

```sh
nix develop
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
```

Then verify the package that will be published:

```sh
nix build .#lattice -o result
./result/bin/lattice --version
./result/bin/lattice --help
```

The binary archive contains the executable, `README.md`, `LICENSE`,
`CHANGELOG.md`, `SECURITY.md`, and `CONTRIBUTING.md`. The workflow also writes a
`SHA256SUMS` file. Nix users should build `.#lattice` from the tagged source
instead of trusting the archive alone.

Release notes are read from `docs/release-notes-<version>.md` when present; the
workflow falls back to generated notes otherwise. The notes must repeat the
scoped limitations, including ignored tests owned by another active change.

The current preparation audit is
[`prepare-public-release/evidence/release-audit.md`](../.cairn/changes/prepare-public-release/evidence/release-audit.md).
It records observed local facts and explicitly pending external actions.

## Tag and publish

```sh
git tag -s "v0.1.0" -m "Lattice v0.1.0"
git push origin "v0.1.0"
```

The tag triggers `.github/workflows/release.yml`. That workflow re-runs the
maintained checks, builds the package, verifies `lattice --version`, creates the
archive and checksums, and publishes a GitHub release with generated notes.

If the workflow fails, fix the commit and create a new tag. Do not move a
published tag or replace a published asset silently. If an archive is wrong and
the tag is already public, publish a new patch release and state the affected
tag and checksum in the notes.

## Version increments

- **Patch**: documentation, diagnostics, or behavior fixes that preserve the
  workflow, contract, and transport interfaces.
- **Minor**: new workflow features, adapters, or contract operations that remain
  backward compatible within the pre-1.0 boundary.
- **Major**: breaking workflow format, contract, transport, authority, or store
  compatibility changes. Pre-1.0 breaking changes may ship in a minor release
  only when the changelog and migration guidance are explicit.

A version bump must update the changelog, package metadata, and any checked-in
release selection that names the version.

## Non-claims

A release proves that the selected source revision built and that the recorded
checks passed for that revision. It does not prove arbitrary workflow
correctness, host confinement, external effect success, credential freshness,
multi-tenant isolation, or production fitness.
