## Phase 1: Implementation

- [x] [serial] 1.1 Pin the `cairn` flake input to `317d3ff648f01a5ea1572e6126f4dd71bc7afa99` with the HTTPS URL and `inputs.artifact` override, and lock only that input (`flake.nix`, `flake.lock`). Evidence: `evidence/verification.json` (`lock_delta`). r[lattice.repository_validation.cairn_producer_alignment]
- [x] [serial] 1.2 Refresh the upstream policy snapshots from the same revision and regenerate the policy (`cairn-policy/{contracts.ncl,upstream-default.ncl,generated/cairn-policy.json,PROVENANCE.md}`). Evidence: `evidence/verification.json` (`policy`). r[lattice.repository_validation.cairn_producer_alignment.scenario.policy_accepted]
- [x] [serial] 1.3 Document the fail-closed gate list and the new pin (`docs/release-readiness-integration.md`, `README.md`, `CHANGELOG.md`). Evidence: `evidence/verification.json` (`release_readiness`). r[lattice.repository_validation.cairn_producer_alignment.scenario.gates_selected]
- [x] [serial] 1.4 Pass each Cairn subcommand only the options it accepts: the fixture's per-subcommand table (`tests/support/cairn.rs`), the direct `status` call (`tests/test_repository_validation.rs`), the dev-shell wrapper (`flake.nix`) and the documented release commands. Evidence: `evidence/verification.json` (`tests`). r[lattice.repository_validation.cairn_producer_alignment]

## Phase 2: Verification

- [x] [serial] 2.1 Run `cairn validate`, `cairn doctor`, `cairn traceability coverage`, `cairn implementation-ready`, every active change's gates, the release-readiness receipt and the Cairn-dependent integration tests with the new binary, and record the results in `evidence/verification.json`. r[lattice.repository_validation.cairn_producer_alignment]
- [ ] [serial] 2.2 Run the full `scripts/check.rs` and `nix flake check` in the integration batch with Lattice #31. r[lattice.repository_validation.cairn_producer_alignment]
