# Crate Consumability Delta

## ADDED Requirements

### Requirement: The repo MUST provide a checked-in default CI release-readiness workflow.

r[crate_consumability.ci_release_workflow.default_flake_ci] The repo MUST provide a checked-in default CI release-readiness workflow.

#### Scenario: Default flake CI

r[crate_consumability.ci_release_workflow.default_flake_ci.workflow_runs_flake_check]

- GIVEN a maintainer opens a change against the crate
- WHEN the default CI workflow runs without private dependency credentials
- THEN it runs the sandbox-safe flake release gate and fails if the flake verifier or policy freshness check fails

### Requirement: The default CI workflow MUST be safe to run without repository secrets.

r[crate_consumability.ci_release_workflow.secret_safe_default] The default CI workflow MUST be safe to run without repository secrets.

#### Scenario: No default secrets

r[crate_consumability.ci_release_workflow.secret_safe_default.no_secret_required]

- GIVEN the default CI workflow is reviewed
- WHEN no private SSH key, deploy token, or credential secret is configured
- THEN the workflow still runs its public/sandbox-safe checks and does not embed credential values in repository content

### Requirement: The full direct release verifier path MUST be documented for CI environments with private dependency access.

r[crate_consumability.ci_release_workflow.full_verifier_private_access] The full direct release verifier path MUST be documented for CI environments with private dependency access.

#### Scenario: Private access verifier

r[crate_consumability.ci_release_workflow.full_verifier_private_access.documented_full_mode]

- GIVEN a maintainer provisions private UCAN dependency access in CI
- WHEN they configure the full release verifier job
- THEN the docs identify the direct verifier command and the private dependency access boundary without committing credentials

### Requirement: CI release verification SHOULD retain inspectable verifier receipts.

r[crate_consumability.ci_release_workflow.receipt_artifacts] CI release verification SHOULD retain inspectable verifier receipts.

#### Scenario: Receipt artifacts

r[crate_consumability.ci_release_workflow.receipt_artifacts.inspectable_receipts]

- GIVEN CI runs the release verifier or flake release gate
- WHEN verifier receipts are produced
- THEN JSON and Markdown summaries are retained as logs or artifacts for reviewer inspection
