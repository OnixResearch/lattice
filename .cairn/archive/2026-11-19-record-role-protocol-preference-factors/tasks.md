## 1. Producer re-pin

- [x] [serial] Re-pin the five Choregraph crates, the `choregraph` flake input, `choregraphCargoSource` and the source-admission assertions to `fcbae0aa17bc0eab1345529adb9ff104a46540ef`; update `Cargo.lock` with `cargo update` and `flake.lock` with `nix flake update choregraph`. r[lattice.role_protocol.preference_factors.producer]
- [x] [serial] Recompute the source archive BLAKE3 (`715d11b6e73ae618f778aaf0a951fb003b5745f54356b6d60e393f0cb20627fc`) and the flake narHash (`sha256-ZByP+RlUb2Gj2M6K28mNXUDdk/ukmjEBSCs8boJn9i0=`, matched by `nix hash path` over the extracted archive). Update the admission receipt, test constants, `FROZEN_CHOREGRAPH_REVISION` and the documents that name the revision. Confirm that no new above-root restore is needed. r[lattice.role_protocol.preference_factors.producer]

## 2. Implementation

- [x] [serial] Add the default-off `PreferenceProfile`, `PreferenceRecord` and `PreferenceReport` DTOs and the optional, skip-when-absent `Cohort::preference`. r[lattice.role_protocol.preference_factors]
- [x] [serial] Add `admit_annotated_projection`. It admits the unchanged v1 base, verifies the producer receipt and annotated local set, checks their linkage, and records factors only under `record-declared-factors`. r[lattice.role_protocol.preference_factors.identity]
- [x] [serial] Validate recorded factors in `plan_session` against the cohort roles, local artifacts and exact `InternalChoice` labels. Expose them through session inspection only when present. r[lattice.role_protocol.preference_factors.boundary]
- [x] [serial] Add positive, default-off, baseline-comparison, crossed-cohort and forged-record fixtures in `tests/test_role_protocol_preference.rs`. r[lattice.role_protocol.preference_factors.recorded] r[lattice.role_protocol.preference_factors.default_off] r[lattice.role_protocol.preference_factors.baseline_comparison] r[lattice.role_protocol.preference_factors.identity.crossed] r[lattice.role_protocol.preference_factors.boundary.forged]
- [x] [serial] Replace the producer-prerequisite blocker in `docs/role-protocol-execution.md`, add the new pure modules to the role-protocol boundary guard, and record the baseline comparison in `evidence/baseline-comparison.json`. r[lattice.role_protocol.preference_factors]

## 3. Verification

- [x] [serial] Run the focused role-protocol and source-admission targets, Clippy, formatting, native Cairn validation and the Octet semantic comparison. r[lattice.role_protocol.preference_factors] r[lattice.role_protocol.preference_factors.producer]

  These checks passed on 2026-11-19 on the integrated tree, which is Lattice
  `661499d282c2e79ebefbd0f9b35beb5f8d7795cd` with this change applied.
  `scripts/check.rs` passed all 36 steps, including the all-features and default
  workspace tests, Clippy with `-D warnings`, formatting, and the role-protocol
  boundary guard. Within those runs the focused targets passed 49 tests:
  preference 8, execution 15, uncertainty 9, shell 5, persistence 3, authority 2,
  CLI 2, transport 2 and source admission 3.

  The Octet semantic comparison against the previous committed baselines showed
  0 new and 0 resolved findings for root (1,052) and core (25). Every finding ID
  is re-keyed because the root `Cargo.toml` changed, so both baselines were
  rewritten with `cargo-octet check --write-baseline`. `octet-precommit-check`
  then passed with 0 new and 0 stale findings in both scopes. After sync and
  archive, native Cairn validation passed 55 specifications with no issues.
- [x] [serial] Build `checks.x86_64-linux.choregraph-source` and `checks.x86_64-linux.nextest` from a committed tree through `git+file`. r[lattice.role_protocol.preference_factors.producer]

  Both checks passed on 2026-11-19 through `git+file`. They used the same Rust,
  Cargo and Nix inputs as the integrated tree; only the Octet baselines and these
  lifecycle files differ. `choregraph-source` admitted the re-pinned source and
  refused its floating-source negative fixture. Nextest ran 1,569 tests and all
  passed (2 slow); 63 were skipped. The full `nix flake check` over the
  integrated tree is the integration gate, and its result is recorded with the
  integration commit. None of these observations covers other platforms, remote
  receivers or a release.
