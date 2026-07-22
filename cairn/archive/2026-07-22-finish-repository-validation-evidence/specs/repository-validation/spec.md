## ADDED Requirements

### Requirement: Repository-check evidence completion
r[lattice.repository_validation.repository_checks_completion] Lattice MUST maintain direct adversarial evidence for all 11 reviewed repository-validation rows without promoting selected markers or changing unrelated linkage.

#### Scenario: Check-scoped signing remains external
r[lattice.repository_validation.repository_checks_completion.scenario.signing]
- GIVEN a host-level missing signing key and the maintained full-check command
- WHEN validation runs with the command-scoped empty `secret-key-files` override
- THEN validation MUST proceed without a placeholder key and the flake MUST contain no signing-policy override

#### Scenario: Nested and non-Rust sources remain available
r[lattice.repository_validation.repository_checks_completion.scenario.source_closure]
- GIVEN a configured test path dependency with nested output-like Rust names and a tracked JSON fixture
- WHEN Git ignore classification and filtered Nix Nextest or Clippy compilation run
- THEN the nested Rust source and JSON fixture MUST remain admitted, compiled, and directly observed

#### Scenario: Formatter and Clippy ownership remain bounded
r[lattice.repository_validation.repository_checks_completion.scenario.ownership]
- GIVEN the compiled path dependency is intentionally nonconforming to project formatting and Clippy policy
- WHEN flake format and Clippy checks run
- THEN they MUST check package `lattice`, Clippy MUST deny repository-owned warnings, and dependency lint admission MUST remain disabled

#### Scenario: Exactly 11 rows become exact
r[lattice.repository_validation.repository_checks_completion.scenario.exact]
- GIVEN the reviewed 315-item inventory and the 11-ID repository-validation tranche
- WHEN completion validation runs
- THEN exactly those 11 rows MUST be exact, BLAKE3-bound to current safe evidence, and marker-free while every unrelated row remains unchanged

#### Scenario: Repository-check completion drift fails closed
r[lattice.repository_validation.repository_checks_completion.scenario.fail_closed]
- GIVEN command, signing, source-filter, ignore, fixture, formatter, Clippy, classification, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
