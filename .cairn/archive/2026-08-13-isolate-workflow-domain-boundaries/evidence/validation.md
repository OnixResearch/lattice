# Workflow domain boundary validation

## Implementation result

Lattice now owns workflow graph identities, port declarations, graph mutations, plans, and domain errors. The functional core receives explicit `WorkflowTimestamp` values and does not read ambient time.

Subwayrat graph values remain under `src/tui/`. The adapter in `src/tui/graph.rs` converts between Subwayrat canvas values and Lattice workflow values. Failed conversion preserves the prior workflow or canvas state.

The Choregraph adapter remains the graph-admission boundary. Nickel, JSON, and redb paths reconstruct Lattice values without running effects. Stored graphs with exhausted `u64::MAX` node or port identities fail restoration instead of wrapping to zero.

## Checks

The following checks passed on 2026-08-13:

- `scripts/check.rs`;
- `cargo clippy --package lattice --all-targets --no-deps -- -D warnings`;
- `octet-precommit-check`;
- `nix flake check -L --option secret-key-files '' --builders ''`;
- Cairn validation;
- Cairn proposal, design, and tasks gates;
- the Octet migration positive and negative self-test; and
- focused stored-identity restoration, traceability, scheduler, repository-validation, and workflow tests.

The final Octet gate reported `warning-only`. It reported 1,101 warnings, 866 accepted and current identities, zero new identities, zero stale identities, and zero errors. The reviewed migration preserved lint-category identity totals and removed workflow-core clock identities.

The first Nix attempt reached a remote-store hash mismatch while importing a nightly Rust source path. The local-builder rerun passed all checks.

## Claim boundary

This evidence establishes selected ownership, dependency, explicit-time, conversion, compatibility, and fail-closed restoration facts. It does not prove whole-workflow correctness, provider correctness, external-effect safety, Choregraph correctness, Subwayrat correctness, or release readiness.
