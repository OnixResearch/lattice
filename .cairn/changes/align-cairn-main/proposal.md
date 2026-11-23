# Align Lattice on Cairn `main`

## Why

Lattice's `cairn` flake input is pinned at `47ea1b867c44b026c030b535f9a1041950c482a0`, a commit only on Cairn's branch `release-selection-20261109`. Kiln #49's scripted-worker VM test needs Kiln and Lattice to share one Cairn revision that has `implementation-ready`. Cairn `main` at `317d3ff648f01a5ea1572e6126f4dd71bc7afa99` has `implementation-ready`, consumer-selected release gates, a fail-closed refusal of a malformed gate list, and a generated workflow catalog that files `cairn readiness` under the `readiness` class.

## What Changes

- Pin the `cairn` flake input to `317d3ff648f01a5ea1572e6126f4dd71bc7afa99`. Keep the anonymous-HTTPS URL and the `inputs.artifact` override, and lock only that input. r[lattice.repository_validation.cairn_producer_alignment]
- Refresh the `cairn-policy/contracts.ncl` and `cairn-policy/upstream-default.ncl` snapshots from the same revision, and regenerate `cairn-policy/generated/cairn-policy.json`. That revision's policy loader rejects the earlier `5e1fe08d` snapshots. Lattice-owned overlays are unchanged. r[lattice.repository_validation.cairn_producer_alignment.scenario.policy_accepted]
- Document that Cairn now refuses a malformed `required_external_commands` list instead of falling back. Lattice's list is accepted and is still reported as `consumer-selected-gates`. r[lattice.repository_validation.cairn_producer_alignment.scenario.gates_selected]
- Pass each Cairn subcommand only the options it accepts. The pinned binary refuses unknown options, where the old one ignored them. This covers the test fixture's per-subcommand table, the dev-shell `cairn` wrapper's `--policy` injection and the documented release commands. r[lattice.repository_validation.cairn_producer_alignment]

## Impact

- **Files**: `flake.nix`, `flake.lock`, `cairn-policy/{contracts.ncl,upstream-default.ncl,generated/cairn-policy.json,PROVENANCE.md}`, `docs/release-readiness-integration.md`, `README.md`, `CHANGELOG.md`, `tests/support/cairn.rs`, `tests/test_repository_validation.rs`, and markers in `scripts/check.rs` and `tests/test_release_readiness_integration.rs`.
- **Compatibility**: The native binary at the new pin rejects a policy exported from the old snapshots, and the old binary is no longer the flake's `LATTICE_CAIRN_BIN`. The generated policy differs from the previous one only by the upstream changes between the two snapshot revisions. Before, `init`, `status` and `preserves-corpus` silently ignored the fixture's options; dropping those options keeps their behavior.
- **Testing**: `cairn policy export --check`, `cairn validate`, `cairn doctor`, `cairn traceability coverage`, `cairn implementation-ready`, every active change's gates, the release-readiness receipt, and the Cairn-dependent integration tests. The full `scripts/check.rs` and `nix flake check` run in the integration batch with Lattice #31.

## Out of Scope

- Kiln's vendored Cairn crates and onix-core pins.
- The pre-existing traceability coverage debt; its missing and dangling sets are identical under both binaries apart from this change's markers.
