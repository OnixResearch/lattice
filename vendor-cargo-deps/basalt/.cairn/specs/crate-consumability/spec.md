# Crate Consumability Specification

## Purpose

Defines the `crate-consumability` capability.

## Requirements

### Requirement: The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

r[crate_consumability.use_remote_ucan_dependency.dependency_source] The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

#### Scenario: Remote resolution

r[crate_consumability.use_remote_ucan_dependency.dependency_source.remote_resolution]

- GIVEN an application adds this crate as a dependency

- WHEN Cargo resolves the dependency graph

- THEN UCAN is fetched from the configured remote source or an explicit consumer override, not from this repo's parent directory

#### Scenario: Local override

r[crate_consumability.use_remote_ucan_dependency.dependency_source.local_override]

- GIVEN a maintainer wants to test against a local UCAN checkout

- WHEN they use the documented local override path

- THEN the public manifest remains valid for remote consumers

### Requirement: The crate MUST package successfully with the remote dependency contract.

r[crate_consumability.use_remote_ucan_dependency.package_verification] The crate MUST package successfully with the remote dependency contract.

#### Scenario: Cargo package

r[crate_consumability.use_remote_ucan_dependency.package_verification.cargo_package]

- GIVEN release verification runs

- WHEN `cargo package` executes

- THEN the package manifest passes dependency-source validation and does not fail because UCAN is path-only

### Requirement: The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

r[crate_consumability.package_metadata.metadata_complete] The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

#### Scenario: Cargo metadata

r[crate_consumability.package_metadata.metadata_complete.cargo_metadata]

- GIVEN a consumer or registry reads package metadata

- WHEN Cargo emits metadata for the crate

- THEN repository/readme/license/description/discoverability fields are present and accurate

### Requirement: The release rail MUST detect missing or invalid public package metadata before publication.

r[crate_consumability.package_metadata.metadata_verification] The release rail MUST detect missing or invalid public package metadata before publication.

#### Scenario: Package warning regression

r[crate_consumability.package_metadata.metadata_verification.package_warning_regression]

- GIVEN a maintainer prepares a package

- WHEN package metadata checks run

- THEN missing repository/documentation/homepage-style warnings are fixed or explicitly justified

### Requirement: The library crate MUST be consumable without enabling CLI-only dependencies.

r[crate_consumability.cli_feature_split.library_without_cli_deps] The library crate MUST be consumable without enabling CLI-only dependencies.

#### Scenario: Consumer library build

r[crate_consumability.cli_feature_split.library_without_cli_deps.consumer_library_build]

- GIVEN a downstream app depends on the crate for enforcement APIs only

- WHEN Cargo builds the dependency graph without the CLI feature

- THEN CLI parser dependencies are not required for the library path

### Requirement: The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary] The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

#### Scenario: Cli enabled build

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary.cli_enabled_build]

- GIVEN an operator enables the CLI feature

- WHEN the binary target builds and the enforce command is smoke-tested

- THEN existing CLI behavior and JSON receipt shape are preserved

### Requirement: The crate MUST document the supported public API that downstream applications should call.

r[crate_consumability.stabilize_public_api.documented_api] The crate MUST document the supported public API that downstream applications should call.

#### Scenario: Docs example

r[crate_consumability.stabilize_public_api.documented_api.docs_example]

- GIVEN a downstream developer opens the crate docs or README

- WHEN they follow the library example

- THEN they can parse policy JSON, construct a request, enforce it, and inspect the receipt without using private knowledge

### Requirement: The crate MUST have tests that lock intended public allow/deny/error behavior.

r[crate_consumability.stabilize_public_api.api_regression_tests] The crate MUST have tests that lock intended public allow/deny/error behavior.

#### Scenario: Public behavior tests

r[crate_consumability.stabilize_public_api.api_regression_tests.public_behavior_tests]

- GIVEN public API tests run

- WHEN positive and negative enforcement scenarios execute

- THEN expected receipts and errors remain stable

### Requirement: The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

r[crate_consumability.policy_artifact_consumption.artifact_contract] The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

#### Scenario: Consumer policy setup

r[crate_consumability.policy_artifact_consumption.artifact_contract.consumer_policy_setup]

- GIVEN a downstream app integrates the crate

- WHEN it needs a policy input

- THEN the docs state whether to include, generate, or supply policy JSON and how to verify freshness

### Requirement: The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

r[crate_consumability.policy_artifact_consumption.freshness_gate] The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

#### Scenario: Policy export check

r[crate_consumability.policy_artifact_consumption.freshness_gate.policy_export_check]

- GIVEN policy source changes

- WHEN the freshness check runs

- THEN stale generated JSON or package artifacts cause verification failure

### Requirement: The crate MUST include integration tests that use only the public consumer API.

r[crate_consumability.consumer_integration_tests.public_surface_tests] The crate MUST include integration tests that use only the public consumer API.

#### Scenario: External style test

r[crate_consumability.consumer_integration_tests.public_surface_tests.external_style_test]

- GIVEN integration tests run

- WHEN they import the crate like a downstream app

- THEN policy parsing, request construction, enforcement, and receipt inspection work through public APIs

### Requirement: The integration suite MUST prove fail-closed behavior for common consumer mistakes.

r[crate_consumability.consumer_integration_tests.negative_coverage] The integration suite MUST prove fail-closed behavior for common consumer mistakes.

#### Scenario: Consumer misconfigurations

r[crate_consumability.consumer_integration_tests.negative_coverage.consumer_misconfigurations]

- GIVEN tests exercise malformed policy, unsupported schema, wrong resource, wrong ability, and missing capability

- WHEN enforcement runs

- THEN the crate returns deterministic errors or denied receipts without panics

### Requirement: The repo MUST expose one documented release-readiness rail for crate consumers.

r[crate_consumability.release_verification_rail.single_gate] The repo MUST expose one documented release-readiness rail for crate consumers.

#### Scenario: Maintainer release check

r[crate_consumability.release_verification_rail.single_gate.maintainer_release_check]

- GIVEN a maintainer prepares the crate for downstream use

- WHEN they run the documented rail

- THEN fmt, tests, clippy, policy freshness, feature builds, and package validation are covered

### Requirement: The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

r[crate_consumability.release_verification_rail.target_hygiene] The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

#### Scenario: Clean environment

r[crate_consumability.release_verification_rail.target_hygiene.clean_environment]

- GIVEN verification runs in a fresh or repo-local target directory

- WHEN Cargo builds dependencies

- THEN missing `.rlib`/`.rmeta` artifacts from prior stale state do not affect the result

### Requirement: The repo MUST expose a flake release check that delegates to the canonical Rust release verifier.

r[crate_consumability.harden_test_harness.flake_release_check] The repo MUST expose a flake release check that delegates to the canonical Rust release verifier and records any sandbox-only deferrals in the verifier receipt.

#### Scenario: Flake release check

r[crate_consumability.harden_test_harness.flake_release_check.default_flake_gate]

- GIVEN a maintainer runs the default flake checks
- WHEN the release check executes in a sandbox that cannot fetch the private UCAN Git dependency
- THEN it compiles and runs the Rust release verifier with policy freshness prevalidated as a Nix input and records Cargo-dependent rails as deferred in the receipt

### Requirement: The release verifier MUST emit machine-readable and human-readable receipts.

r[crate_consumability.harden_test_harness.release_receipts] The release verifier MUST emit machine-readable and human-readable receipts.

#### Scenario: Receipt materialization

r[crate_consumability.harden_test_harness.release_receipts.receipt_materialization]

- GIVEN the release verifier runs
- WHEN each step completes or a step fails
- THEN JSON and Markdown summaries record step names, commands, statuses, elapsed time, and bounded output tails

### Requirement: The release rail MUST run integration tests through nextest while retaining doctest coverage.

r[crate_consumability.harden_test_harness.nextest_rail] The release rail MUST run integration tests through nextest while retaining doctest coverage.

#### Scenario: Nextest and doctests

r[crate_consumability.harden_test_harness.nextest_rail.nextest_and_doctests]

- GIVEN the release rail executes no-default and CLI-feature tests
- WHEN integration tests and doctests are selected
- THEN nextest runs the integration/unit tests and Cargo runs doctests explicitly

### Requirement: Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

r[crate_consumability.harden_test_harness.enforcement_invariants] Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

#### Scenario: Enforcement invariant coverage

r[crate_consumability.harden_test_harness.enforcement_invariants.boundary_and_capabilities]

- GIVEN the enforcement regression suite runs
- WHEN resource prefix siblings, unknown contracts, id mismatches, malformed contract shape, and multiple capabilities are tested
- THEN the crate returns deterministic allowed, denied, or error outcomes without panics

### Requirement: CLI tests MUST cover common negative operator paths.

r[crate_consumability.harden_test_harness.cli_negative_smokes] CLI tests MUST cover common negative operator paths.

#### Scenario: CLI negative coverage

r[crate_consumability.harden_test_harness.cli_negative_smokes.operator_failures]

- GIVEN the CLI feature test rail runs
- WHEN denied requests, malformed policy files, and bad capability syntax are exercised
- THEN the CLI emits the expected JSON receipt or nonzero parse/error result

### Requirement: Public API and package-content guardrails MUST protect consumer-facing surfaces.

r[crate_consumability.harden_test_harness.api_and_package_guards] Public API and package-content guardrails MUST protect consumer-facing surfaces.

#### Scenario: API and package guards

r[crate_consumability.harden_test_harness.api_and_package_guards.consumer_surface]

- GIVEN public API tests and the package-content verifier run
- WHEN DTO accessors/serialization and package file lists are checked
- THEN required public API behavior and packaged source/policy artifacts remain present

### Requirement: Release verification MUST use a pinned Rust toolchain selection.

r[crate_consumability.harden_test_harness.toolchain_pin] Release verification MUST use a pinned Rust toolchain selection.

#### Scenario: Toolchain reproducibility

r[crate_consumability.harden_test_harness.toolchain_pin.explicit_toolchain]

- GIVEN a maintainer enters or runs the repo toolchain
- WHEN Rust tooling is selected
- THEN the toolchain file names an explicit Rust version rather than a moving stable channel

### Requirement: The crate MUST retain `AGPL-3.0-or-later` package license metadata.

r[crate_consumability.preserve_agpl_license.license_preserved] The crate MUST retain `AGPL-3.0-or-later` package license metadata.

#### Scenario: Manifest review

r[crate_consumability.preserve_agpl_license.license_preserved.manifest_review]

- GIVEN package metadata is reviewed

- WHEN Cargo reads the manifest

- THEN the license remains `AGPL-3.0-or-later` and is not silently weakened or removed

### Requirement: Consumer documentation MUST state the license boundary without making legal advice claims.

r[crate_consumability.preserve_agpl_license.license_documented] Consumer documentation MUST state the license boundary without making legal advice claims.

#### Scenario: Consumer reads docs

r[crate_consumability.preserve_agpl_license.license_documented.consumer_reads_docs]

- GIVEN a downstream application author evaluates adoption

- WHEN they read README/package docs

- THEN the AGPL license is visible and no alternate license or compatibility promise is implied

### Requirement: The crate MUST expose `basalt` as the canonical package, library, and CLI identity.

r[crate_consumability.rename_basalt.canonical_name] The crate MUST expose `basalt` as the canonical package, library, and CLI identity.

#### Scenario: Canonical Cargo and CLI names

r[crate_consumability.rename_basalt.canonical_name.cargo_and_cli]

- GIVEN a consumer reads Cargo metadata or enables the CLI feature
- WHEN package, library, and binary names are resolved
- THEN `basalt` is the canonical crate import path and CLI binary name

### Requirement: Documentation and release surfaces MUST use the Basalt identity while preserving former-name discoverability.

r[crate_consumability.rename_basalt.docs_and_release_surfaces] Documentation and release surfaces MUST use the Basalt identity while preserving former-name discoverability.

#### Scenario: Basalt docs and verifier names

r[crate_consumability.rename_basalt.docs_and_release_surfaces.examples]

- GIVEN a maintainer or consumer follows README examples or release-verifier commands
- WHEN they copy the documented commands
- THEN the commands use `basalt` names and explain that Basalt was formerly `ucan-nickel-contracts`

### Requirement: The rename MUST classify intentional old-name references so compatibility is reviewable.

r[crate_consumability.rename_basalt.compatibility_classification] The rename MUST classify intentional old-name references so compatibility is reviewable.

#### Scenario: Stable policy schema compatibility

r[crate_consumability.rename_basalt.compatibility_classification.policy_schema]

- GIVEN stale-name searches run after the rename
- WHEN `ucan-nickel-contracts` remains in source-controlled files
- THEN each remaining occurrence is either historical Cairn archive content, former-name discoverability text, or an intentionally stable policy schema identifier

### Requirement: Rename verification MUST prove the renamed crate still passes consumer and release gates.

r[crate_consumability.rename_basalt.rename_verification] Rename verification MUST prove the renamed crate still passes consumer and release gates.

#### Scenario: Rename gate suite

r[crate_consumability.rename_basalt.rename_verification.gates]

- GIVEN the Basalt rename is implemented
- WHEN formatting, tests, clippy, policy freshness, Nix flake checks, Cairn validation, and stale-name classification run
- THEN the renamed crate is ready to commit without unreviewed stale identity references

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

### Requirement: The release rail MUST build Clankers as the separate consumer fixture against Basalt's public package surface.

r[crate_consumability.external_consumer_fixture.separate_consumer] The release rail MUST build `../clankers/` as the separate consumer fixture against Basalt's public package surface rather than relying only on in-package integration tests.

#### Scenario: Separate consumer build

r[crate_consumability.external_consumer_fixture.separate_consumer.build]

- GIVEN `../clankers/` depends on Basalt through the public package surface or an explicit sibling-repo override
- WHEN the fixture builds with the default library feature set
- THEN it imports `basalt` through public APIs without requiring CLI-only dependencies, private Basalt internals, or broad Clankers architecture rewrites

### Requirement: The consumer fixture MUST exercise policy consumption and enforcement receipts through public APIs.

r[crate_consumability.external_consumer_fixture.public_api_behavior] The Clankers consumer fixture MUST exercise policy consumption and enforcement receipts through public APIs.

#### Scenario: Consumer policy and receipts

r[crate_consumability.external_consumer_fixture.public_api_behavior.policy_and_receipts]

- GIVEN the Clankers fixture uses packaged default policy and supplied policy JSON
- WHEN it constructs requests, supplies capabilities, and inspects receipts
- THEN allowed, denied, and malformed/unsupported policy cases behave deterministically through public APIs

### Requirement: The release verifier MUST record consumer-fixture evidence.

r[crate_consumability.external_consumer_fixture.release_evidence] The release verifier MUST record Clankers consumer-fixture evidence in its machine-readable and human-readable receipts.

#### Scenario: Fixture receipt

r[crate_consumability.external_consumer_fixture.release_evidence.receipt_step]

- GIVEN release verification runs
- WHEN the Clankers consumer fixture step passes, fails, or is explicitly deferred because `../clankers/` or private dependency access is unavailable
- THEN the verifier receipt records the fixture command, status, bounded output, and deferral reason if any

### Requirement: License consumption contract
r[crate_consumability.license_consumption.contract] Basalt MUST support a receipt contract that records downstream consumer, Basalt artifact identity, license expression, review action, and non-claim boundary for license-sensitive adoption.

#### Scenario: Acknowledged license consumption passes
r[crate_consumability.license_consumption.fixtures.positive]
- GIVEN a consumer receipt names Basalt artifact identity, AGPL license expression, reviewer action, and required non-claims
- WHEN license-consumption validation runs
- THEN validation MUST pass and preserve the consumer identity.

#### Scenario: Missing license review fails
r[crate_consumability.license_consumption.fixtures.negative]
- GIVEN a consumer receipt omits review action, uses wrong artifact identity, has stale license metadata, or weakens non-claims
- WHEN validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: License validation boundary
r[crate_consumability.license_consumption.validation] Basalt MUST validate license-consumption receipt shape without granting legal advice or alternate license terms.

#### Scenario: Non-claim is visible
r[crate_consumability.license_consumption.docs]
- GIVEN a license-consumption receipt validates
- WHEN the supported claim is rendered
- THEN it MUST state that the receipt records review metadata only and is not legal advice or a license exception.

### Requirement: Final validation
r[crate_consumability.license_consumption.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers license receipt health
r[crate_consumability.license_consumption.final_validation.fixtures]
- GIVEN valid and invalid license-consumption fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

### Requirement: Policy identity maps

r[crate_consumability.iddqd_policy_identity_maps.contract]

Basalt MUST validate policy contracts and backend metadata through identity maps whose keys are derived from the mapped values rather than trusted as duplicated external keys.

#### Scenario: Matching policy identity passes

r[crate_consumability.iddqd_policy_identity_maps.fixtures.positive]

- GIVEN a policy artifact whose contract and backend object keys match each value's stable id
- WHEN Basalt builds the runtime policy index
- THEN validation MUST pass and enforcement MUST resolve contracts and backends by those stable ids.

#### Scenario: Duplicate or mismatched identity fails

r[crate_consumability.iddqd_policy_identity_maps.fixtures.negative]

- GIVEN a policy artifact has duplicate ids, an object key that does not match the value id, an empty id, an unsupported backend id, or malformed policy data
- WHEN Basalt builds the runtime policy index
- THEN validation MUST fail closed with deterministic diagnostics before enforcement.

### Requirement: Public policy JSON compatibility

r[crate_consumability.iddqd_policy_identity_maps.serde_boundary]

Basalt MUST preserve the existing Nickel-generated policy JSON object shape while using iddqd-backed identity maps only after parse-time validation.

#### Scenario: Existing generated policy remains compatible

r[crate_consumability.iddqd_policy_identity_maps.docs]

- GIVEN the checked-in generated policy JSON was exported from reviewed Nickel source
- WHEN downstream callers parse it through Basalt's public API
- THEN the public parse and enforcement APIs MUST remain source-compatible while rejecting identity drift earlier.

### Requirement: Final validation

r[crate_consumability.iddqd_policy_identity_maps.final_validation]

The change MUST include positive and negative identity-map coverage plus focused Basalt validation evidence before archive.

#### Scenario: Identity-map fixtures cover drift

r[crate_consumability.iddqd_policy_identity_maps.final_validation.fixtures]

- GIVEN valid and invalid policy identity fixtures
- WHEN focused policy tests and Cairn validation run
- THEN valid fixtures MUST pass and duplicate or mismatched identities MUST fail closed.

### Requirement: Basalt CLI policy filesystem authority is capability-rooted

r[crate_consumability.cap_std_cli_policy_boundary.root_wrappers]

Basalt MUST represent CLI policy input, generated policy, and receipt/output filesystem authority as typed capability roots opened by CLI-feature code.

#### Scenario: Valid CLI policy path stays under the declared root

r[crate_consumability.cap_std_cli_policy_boundary.tests.positive]

- GIVEN Basalt has opened an operator-declared policy or output root
- WHEN CLI code reads a valid relative policy path, uses packaged policy fallback, or writes a receipt/output through the typed root
- THEN the operation MUST remain confined to that declared root while preserving existing valid-input behavior.

#### Scenario: Invalid CLI policy path cannot escape the root

r[crate_consumability.cap_std_cli_policy_boundary.tests.negative]

- GIVEN input names `../` traversal, an absolute path, a missing capability root, a symlink escape attempt, or an output escape attempt
- WHEN Basalt resolves the path through the typed root
- THEN Basalt MUST fail closed before reading policy bytes or writing receipt/output files outside the declared root.

### Requirement: Capability dependency stays outside no-default library APIs

r[crate_consumability.cap_std_cli_policy_boundary.dependency]

Basalt MUST add `cap-std` only to CLI-feature code that owns filesystem effects and MUST preserve no-default library builds without CLI-only filesystem dependencies.

#### Scenario: Library core remains filesystem-neutral

r[crate_consumability.cap_std_cli_policy_boundary.conversion]

- GIVEN enforcement core, UCAN validation, Nickel/Steel backend DTOs, and packaged policy helpers are reviewed
- WHEN cap-std adoption is complete
- THEN those library surfaces MUST operate on in-memory policy data, DTOs, or validated relative locators and MUST NOT depend on `cap-std` or open ambient filesystem paths directly.

### Requirement: Capability boundary is documented without policy overclaims

r[crate_consumability.cap_std_cli_policy_boundary.docs]

Basalt MUST document that capability roots bound local CLI filesystem authority only and do not prove policy truth, UCAN issuer trust, caveat semantics, or downstream deployment readiness.

#### Scenario: Closeout evidence includes CLI policy boundary validation

r[crate_consumability.cap_std_cli_policy_boundary.validation]

- GIVEN the cap-std CLI-policy change is ready to archive
- WHEN focused CLI, no-default library, Cairn validation, and gate checks run
- THEN the evidence MUST include positive and negative filesystem-boundary coverage and visible policy/UCAN non-claim documentation.

### Requirement: Vendored UCAN composes with reviewed exact BLAKE3 consumers

r[crate_consumability.blake3_alignment] Basalt MUST import a reviewed UCAN revision whose exact peer-authentication and Nostr BLAKE3 pins compose with supported consumer graphs. The source revision, vendored snapshot manifest, and Cargo lock MUST move together through owning tools.

#### Scenario: Molten consumes branch-authority policy

- GIVEN Molten requires the reviewed exact BLAKE3 cohort
- WHEN Cargo resolves Basalt and its vendored UCAN peer-authentication crates
- THEN one compatible BLAKE3 version MUST resolve without a patch override

#### Scenario: Vendored source is edited by hand

- GIVEN a vendored manifest differs from the recorded UCAN source revision
- WHEN the snapshot check runs
- THEN verification MUST fail before publication

### Requirement: Compatibility evidence remains bounded

r[crate_consumability.blake3_alignment.verification] Verification MUST cover snapshot parity, Basalt policy APIs, supported feature sets, and one downstream composition. It MUST NOT claim cryptographic correctness or universal compatibility.

#### Scenario: Compatibility rail passes

- GIVEN the imported revision and lock agree
- WHEN focused, release, Nix, and lifecycle checks run
- THEN the receipt MAY report tested dependency compatibility only

### Requirement: Explicit enforcement-only dependency profile
r[crate_consumability.enforcement_profile] Basalt SHALL preserve its default public modules and verification behavior. Explicit no-default consumers MUST omit OIDC HTTP and durable-state adapter dependencies while retaining exact UCAN enforcement. No dependency downgrade or verification bypass is permitted.

#### Scenario: Default consumer
- GIVEN default features
- WHEN Basalt compiles and tests
- THEN OIDC and durable-authority modules retain their existing behavior

#### Scenario: Minimal consumer
- GIVEN default features disabled
- WHEN enforcement compiles and tests on pinned Rust 1.95
- THEN signed execution and document verification remain available without bounded-http, bounded-http-core, replay-ledger-core or revocation-view-core

### Requirement: Maintained isolation evidence
r[crate_consumability.enforcement_profile_evidence] Release verification MUST test the minimal profile and reject empty or forbidden normal dependency observations. Evidence SHALL distinguish actual compiler results from configured feature intent.

#### Scenario: Invalid dependency observation
- GIVEN empty output or a forbidden package
- WHEN the pure dependency validator runs
- THEN it rejects the observation
