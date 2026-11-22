## Why

Basalt is published as a public GitHub repository and is about to be mirrored on
Radicle. The repository must be usable and reviewable by a person arriving with
no workspace access, no credentials, and no sibling checkouts. The current tree
still contains an internal scratch-path reference in a research report, an
editor backup under generated lifecycle policy output, and user-facing
instructions that assume workspace-local consumers. The release
workflow also fails on GitHub-hosted runners because the runner exhausts disk
while building the pinned Nickel toolchain.

## What Changes

- Remove the tracked editor backup from `cairn-policy/generated/` and cover
  editor scratch directories in `.gitignore`.
- Replace the internal scratch-path reference in the holder-observation research
  report with an operator-owned, out-of-repository statement.
- Rewrite the README quick start and requirements for a clean clone inside the
  flake development shell, state maturity and non-claims, document the security
  channel, and describe the optional consumer runner without private sibling
  names or paths.
- Add `SECURITY.md` with GitHub private vulnerability reporting and a Basalt
  scope boundary.
- Free disk space in the release workflow before Nix installation, without
  changing permissions or exposing secrets.

## Impact

- **Files**: `README.md`, `SECURITY.md`, `.github/workflows/release.yml`,
  `.gitignore`, `flake.nix`,
  `research/current/holder-observation/REPORT.md`,
  `cairn-policy/generated/cairn-policy.json.stale-bak` (removed), and this Cairn
  change.
- **Testing**: full-history secret scan, private-reference scan, anonymous
  dependency fetch and remote checks, the direct release verifier, Cargo tests
  and Clippy, the vendored-UCAN `--check`, and `nix flake check -L`.
- **Authority**: no product behavior, policy meaning, dependency revision,
  license term, or package version changes.

## Out of Scope

- Rewriting history or editing immutable `.cairn/archive/**`, evidence,
  receipts, provenance records, or vendored snapshots.
- Changing Cargo or flake pins, registry publication, or the crate's `AGPL` terms.
- Making the optional workspace-local consumer matrix part of the public gate.
- Publishing, tagging, or changing Radicle visibility.
