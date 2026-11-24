## Phase 1: Source and hygiene corrections

- [x] [serial] Remove the tracked editor backup from generated lifecycle policy output, keeping the downstream runner's Cargo patch key equal to the basalt source consumers declare. r[basalt.public_release.anonymous_sources] r[basalt.public_release.credential_boundary]
- [x] [serial] Remove the internal scratch-path reference from the research report, deduplicate the ignore file, and use the sandbox temporary directory in the policy-authoring check. r[basalt.public_release.credential_boundary]

## Phase 2: Public documentation and reporting

- [x] [serial] Rewrite README requirements, quick start, maturity, non-claims, security, and optional downstream guidance for a clean anonymous clone. r[basalt.public_release.public_documentation]
- [x] [serial] Add SECURITY.md with GitHub private vulnerability reporting and the Basalt scope boundary. r[basalt.public_release.public_documentation]

## Phase 3: CI safety

- [x] [serial] Free unused runner disk before Nix installation while preserving read-only permissions and the fork-safe pull-request model. r[basalt.public_release.release_gate]

## Phase 4: Verification

- [x] [serial] Scan history and tip for credentials, triage every finding, and scan active files for private references, recording immutable residuals. r[basalt.public_release.credential_boundary]
- [x] [serial] Prove anonymous build inputs from a fresh clone with an empty Cargo home and cleared SSH configuration, including the pinned Git dependencies. r[basalt.public_release.anonymous_sources]
- [x] [serial] Run the direct release verifier, Cargo tests, strict Clippy, the vendored-UCAN check, and the sandbox-safe flake check; classify any failure against the unmodified base. r[basalt.public_release.release_gate]
