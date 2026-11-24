# Public repository release readiness

## ADDED Requirements

### Requirement: Anonymous reproducible sources

r[basalt.public_release.anonymous_sources] Every flake input and Cargo Git dependency MUST resolve from an anonymous public HTTPS URL at the pinned revision. Active repository instructions MUST NOT require sibling checkouts, SSH credentials, `path:` overrides, or private hosts to build or test the crate. A standalone clone SHALL build its dependency graph with an empty Cargo home and cleared SSH agent and terminal prompts.

#### Scenario: Clean anonymous fetch

- GIVEN a fresh clone of the published revision and an empty `CARGO_HOME`
- WHEN dependency fetch runs with SSH disabled and prompts disabled
- THEN every Git dependency SHALL resolve at its pinned revision through HTTPS.

#### Scenario: Credential or sibling fallback

- GIVEN a build input that needs an SSH key, a private host, or a sibling checkout
- WHEN an anonymous consumer follows the documented quick start
- THEN the build SHALL fail rather than silently substituting private access.

### Requirement: Credential and private-reference boundary

r[basalt.public_release.credential_boundary] Tracked files and full history MUST NOT contain real credentials or key material. Active non-immutable files MUST NOT present personal home paths, private network addresses, personal emails, SSH remotes, or internal scratch paths as build inputs or instructions. Detector markers, synthetic identities, and public digests MAY match generic secret rules and SHALL be triaged individually. Immutable archive evidence and reviewed vendored snapshots SHALL be reported rather than edited.

#### Scenario: Secret scan triage

- GIVEN a full-history secret scan of the published revision
- WHEN a finding matches a synthetic DID, public digest, or commit revision
- THEN the finding SHALL be classified as a false positive with its path and purpose recorded.

#### Scenario: Real credential

- GIVEN a real credential in the tip or in history
- WHEN release readiness is evaluated
- THEN publication SHALL stop and the exposure SHALL be reported without attempting a history rewrite.

### Requirement: Public documentation and reporting

r[basalt.public_release.public_documentation] The README MUST describe what the repository is, its maturity, requirements, a clean-clone quick start, check commands, license terms, security reporting, and explicit non-claims. `SECURITY.md` MUST direct reports to GitHub private vulnerability reporting and state the Basalt scope boundary. Package license metadata and the license text SHALL remain consistent, and third-party notices SHALL be retained.

#### Scenario: Stranger arrival

- GIVEN a reader with only anonymous GitHub access
- WHEN they follow the README requirements and quick start
- THEN the documented commands SHALL be complete, portable, and free of workspace-local assumptions.

#### Scenario: Security report

- GIVEN a vulnerability report
- WHEN the reporter follows `SECURITY.md`
- THEN the report SHALL use the private advisory channel rather than a public issue.

### Requirement: Release gate and CI safety

r[basalt.public_release.release_gate] The primary release checks MUST pass after the release-readiness edits, and any failure SHALL be classified as pre-existing or introduced. GitHub Actions workflows MUST NOT check out untrusted pull-request code under `pull_request_target`, run fork pull requests on self-hosted runners, or expose secrets to forks. Infrastructure fixes SHALL NOT change pins, permissions beyond least privilege, or trust boundaries.

#### Scenario: Sandbox-safe gate

- GIVEN the edited revision
- WHEN the repository release verifier and `nix flake check -L` run
- THEN the verifier SHALL pass and the flake checks SHALL complete without private inputs.

#### Scenario: Fork pull request

- GIVEN a pull request from a fork
- WHEN the release workflow runs
- THEN it SHALL use only public inputs and read-only repository permissions.
