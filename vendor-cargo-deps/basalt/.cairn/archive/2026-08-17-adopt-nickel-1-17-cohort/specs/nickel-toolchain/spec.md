# Basalt Nickel Toolchain Delta

## ADDED Requirements

### Requirement: Basalt pins Nickel 1.17 exactly

r[basalt.nickel_toolchain.cohort] Basalt MUST use Nickel `1.17.0` from upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426` for policy authoring and export.

#### Scenario: Policy tools resolve

- GIVEN Basalt builds policy checks and developer tools
- WHEN cohort admission runs
- THEN every Nickel command MUST use the exact reviewed evaluator

#### Scenario: An older or floating evaluator resolves

- GIVEN an ambient package, branch, or older pin supplies Nickel
- WHEN admission runs
- THEN admission MUST fail

### Requirement: Policy exports remain fresh

r[basalt.nickel_toolchain.freshness] Basalt MUST regenerate checked-in policy artifacts with the new evaluator and MUST pass the existing freshness rail.

#### Scenario: Source and generated policy match

- GIVEN Nickel source and generated policy are current
- WHEN the freshness rail runs
- THEN it MUST pass

#### Scenario: Generated policy is stale

- GIVEN the checked-in export differs from the new evaluator output
- WHEN the freshness rail runs
- THEN it MUST fail

### Requirement: Policy compatibility includes negative fixtures

r[basalt.nickel_toolchain.compatibility] Tests MUST cover valid policy exports and rejected unknown, duplicate, malformed, missing, type-mismatched, import, and contract cases.

#### Scenario: The fixture matrix runs

- GIVEN valid and invalid policy fixtures
- WHEN Nickel `1.17.0` evaluates them
- THEN valid fixtures MUST export expected policy
- AND invalid fixtures MUST fail closed

### Requirement: Authority ownership does not change

r[basalt.nickel_toolchain.boundary] Nickel MUST validate policy shape only. Basalt Rust MUST retain runtime enforcement and authority decisions.

#### Scenario: Policy export succeeds

- GIVEN policy exports successfully
- WHEN a runtime request arrives
- THEN Basalt MUST still perform its complete authority checks

### Requirement: The repository validation rail passes

r[basalt.nickel_toolchain.validation] The change MUST pass policy freshness, positive and negative fixtures, formatting, Clippy, release verification, Cairn gates, and relevant Nix checks.

#### Scenario: Validation completes

- GIVEN the pin, exports, fixtures, and evidence are current
- WHEN validation runs
- THEN every required check MUST pass or report one exact blocker
