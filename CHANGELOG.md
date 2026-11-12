# Changelog

All notable changes to Lattice are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
Semantic Versioning. This project is pre-1.0, so minor releases may change
interfaces.

## [0.1.0] - 2026-11-12

Initial public release candidate. The runtime, CLI, TUI, durable store, scheduler,
approvals, agent adapters, workflow contract, and local workflow exchange are
present but remain pre-1.0.

### Added

- Public release documentation: security policy, contribution guide, server
  deployment guide, release process, documentation index, and launch material.
- A tagged release workflow that runs the maintained checks, builds the Nix
  package, verifies `lattice --version`, and publishes a binary archive with
  checksums.
- `lattice --version` and public Cargo package metadata.

### Changed

- The optional Tile compose integration now lives in the separate `compose/`
  workspace. The default runtime, public checks, and release binary have no Tile
  dependency and no authenticated source requirement.
- Public build inputs use anonymous HTTPS. The transitive SSH-only sources for
  Artifact and Trellis are patched to their exact public revisions, and the Cairn
  and Octet flake inputs use HTTPS overrides.
- The release package no longer embeds the Cairn lifecycle toolchain; only the
  test rail sets `LATTICE_CAIRN_BIN`.

### Known limitations

- The workflow exchange service is a local Unix-socket service. It has no TLS,
  remote authentication, network listener, or multi-tenant isolation.
- Workflows run with host authority. Lattice is not a sandbox for untrusted
  workflows.
- The compose workspace requires authorized Tile source access and is not part of
  the public build closure.
- Three TUI authoring PTY regressions remain tracked by
  `complete-tui-nickel-authoring-parity` and are ignored by the release gate.
  The server release does not claim TUI authoring-parity completion; see
  [the release audit](.cairn/changes/prepare-public-release/evidence/release-audit.md).
- No hosted service, support SLA, or production-readiness claim is included in
  this release.
