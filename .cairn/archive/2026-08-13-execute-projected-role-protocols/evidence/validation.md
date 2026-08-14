# Projected role-protocol validation

## Implementation result

Lattice consumes the frozen Choregraph role-projection cohort at revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`. Strict Lattice DTOs admit the exact global artifact, ordered local set, projection receipt, schemas, proof reference, adapter identity, and source revision.

The pure core owns placement admission, session planning, cursors, envelopes, transitions, ownership, choices, uncertainty recovery, and replay. The shell owns redb persistence, current effect authorization, bounded adapters, operator commands, and safe evidence.

Dispatch persists `Prepared`, exact authorization references, and `DispatchedUnknown` before an adapter starts. Unknown outcomes do not retry automatically. Retry abandons the old attempt and prepares a fresh identity after explicit acknowledgement.

## Checks

The following checks passed on 2026-08-13:

- `cargo test --workspace --all-targets --no-fail-fast`;
- focused protocol execution, CLI, authority, persistence, shell, and transport tests;
- `cargo check --package lattice --all-targets`;
- `cargo clippy --package lattice --all-targets --no-deps -- -D warnings`;
- `cargo -q -Zscript scripts/check-role-protocol-boundary.rs .`;
- `cargo -q -Zscript scripts/check.rs`;
- top-level and `lattice-core` Octet migration positive and negative tests;
- `nix develop -c octet-precommit-check`;
- Cairn validation and proposal, design, and tasks gates; and
- `nix flake check -L --builders ''`.

The final top-level Octet result was `warning-only`. It matched all 864 accepted findings, with zero new findings and zero stale findings. The separate `lattice-core` profile matched all 24 accepted findings, with zero new findings and zero stale findings.

The Nix gate uses a self-contained Cargo vendor tree. It restores the frozen Choregraph schemas and Kamacite codec needed by compile-time includes. The Octet sandbox copies this tree outside the workspace, so dependency paths cannot become product findings.

## Adversarial result

Maintained negative tests reject incomplete or mixed placements, forged session plans, stale receipts, cross-session envelopes, wrong steps, stale labels, duplicate or reordered input, stale ownership, missing current authority, replay dispatch, hidden retry, unsafe diagnostics, and authority overclaims.

Crash-prefix tests cover persistence failure, prepared attempts, dispatch uncertainty, receiver observation, and commit publication. Replay remains pure and starts no adapter.

## Claim boundary

This evidence establishes bounded intake, placement, local transition, persistence-order, authorization-linkage, recovery, replay, and diagnostic facts for the selected cohort. It does not prove transport confidentiality, endpoint identity truth, remote durability, external-role correctness, exactly-once delivery, global progress, deadlock freedom under faults, production readiness, or release eligibility.
