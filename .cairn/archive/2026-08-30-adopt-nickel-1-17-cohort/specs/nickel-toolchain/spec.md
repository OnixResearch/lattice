# Lattice Nickel Toolchain Delta

## ADDED Requirements

### Requirement: Lattice consumes the published Nickel 1.17 evaluator

r[lattice.nickel_toolchain.cohort] Lattice MUST pin the immutable `nickel-eval` object for Nickel CLI `1.17.0`, `nickel-lang 2.2.0`, and core `0.18.0`.

#### Scenario: The producer object exists
r[lattice.nickel_toolchain.cohort.scenario.published]
- GIVEN `nickel-eval` publishes a reviewed immutable object
- WHEN Lattice updates its dependency
- THEN the dependency MUST use that exact Radicle object

#### Scenario: The producer object is absent
r[lattice.nickel_toolchain.cohort.scenario.absent]
- GIVEN no reviewed Nickel `1.17.0` producer object exists
- WHEN Lattice adoption runs
- THEN adoption MUST remain blocked

### Requirement: Direct fixture dependencies match the cohort

r[lattice.nickel_toolchain.fixture_cohort] Lattice MUST pin direct Nickel fixtures to `nickel-lang-core 0.18.0` and command-line checks to Nickel `1.17.0`.

#### Scenario: Product and fixture paths resolve
r[lattice.nickel_toolchain.fixture_cohort.scenario.aligned]
- GIVEN Lattice builds all evaluator paths
- WHEN the cohort guard runs
- THEN product, fixture, and command-line paths MUST match the reviewed cohort

#### Scenario: A mixed cohort is present
r[lattice.nickel_toolchain.fixture_cohort.scenario.mixed]
- GIVEN any direct or transitive Nickel path belongs to an older cohort
- WHEN the guard runs
- THEN the guard MUST fail

### Requirement: Workflow meaning remains Lattice-owned

r[lattice.nickel_toolchain.boundary] `nickel-eval` MUST provide bounded evaluation only. Lattice MUST retain workflow decoding, Choregraph admission, effects, persistence, diagnostics, and evidence.

#### Scenario: Evaluation succeeds
r[lattice.nickel_toolchain.boundary.scenario.admitted]
- GIVEN Nickel returns a valid value
- WHEN Lattice imports a workflow
- THEN Lattice MUST decode and admit the workflow before any effect

#### Scenario: Choregraph rejects the workflow
r[lattice.nickel_toolchain.boundary.scenario.rejected]
- GIVEN evaluation succeeds but pure graph admission fails
- WHEN import runs
- THEN Lattice MUST reject the workflow
- AND no runtime effect MUST start

### Requirement: Compatibility tests cover accepted and rejected workflows

r[lattice.nickel_toolchain.compatibility] Tests MUST cover valid workflows plus malformed source, missing imports, contract failures, unknown fields, excessive bounds, and cohort substitution.

#### Scenario: The matrix runs
r[lattice.nickel_toolchain.compatibility.scenario.matrix]
- GIVEN representative valid and invalid workflow fixtures
- WHEN the new cohort evaluates and imports them
- THEN valid workflows MUST retain supported outcomes
- AND invalid workflows MUST fail with stable dispositions

### Requirement: The repository validation rail passes

r[lattice.nickel_toolchain.validation] The change MUST pass focused import tests, operator checks, formatting, Clippy, Octet, Cairn gates, and relevant Nix checks.

#### Scenario: Validation completes
r[lattice.nickel_toolchain.validation.scenario.passed]
- GIVEN the producer pin, direct dependencies, fixtures, and docs are current
- WHEN validation runs
- THEN every required check MUST pass or report one exact blocker
