# Validation evidence

## Result

The orchestration identity migration passed all selected checks on 2026-08-14.

## Dependency evidence

Lattice pins reviewed Octet revision `bac0f3220ada38110c03498ccf2beae5ee83e642`. That revision declares the Lattice core and wire compatibility scopes.

The frozen pre-migration redb JSON fixture has BLAKE3 identity `51302c6b18109162868c01205c74a6d3cb378c2551c0b1ff99e78fcbdb89f905`.

The baseline and migrated Nickel orchestration both have BLAKE3 identity `bc051b8538fd8c5355ddc0fd9888a25dfec3384a2132a184e131d2ad3a003dea`.

## Passed checks

These checks passed:

- `cargo fmt --package lattice` and the repository format check;
- focused identity, format, compatibility, core, shell, store, recovery, replay, inspection, scheduler, webhook, and traceability tests;
- compile-pass and compile-fail fixtures for every maintained orchestration identity role;
- `cargo test --workspace --all-targets --no-fail-fast`;
- focused project and budget-ledger Clippy with `-D warnings`;
- `scripts/check.rs`, including operator E2E and the orchestration identity boundary guard;
- `nix develop --builders '' -c octet-precommit-check`;
- Cairn validation and the proposal, design, and tasks gates; and
- `nix flake check -L --option secret-key-files '' --builders ''`.

The final top-level Octet result retained 859 accepted findings with zero new and zero stale findings. Its baseline identity is `b3:06a05f687a1e64bf1e28ddffecc77aa49bc09e901a15e60663701aaf6cc967e1`.

The independent `lattice-core` Octet result retained 24 accepted findings with zero new and zero stale findings. Its baseline identity is `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

## Negative evidence

The selected tests reject malformed identity text, cross-role source substitutions, stale revisions, stale approvals, wrong owners, foreign runs, duplicate trigger admissions, conflicting facts, malformed stored keys, and event-chain tampering.

These failures occur before workflow, provider, credential, scheduler, webhook, or other external effects start.

## Claim boundary

This evidence supports checked local identity syntax, nominal role separation, compatibility for the selected fixtures, and deterministic local replay.

It does not prove provider truth, remote cancellation, external exactly-once effects, complete history, or production readiness.
