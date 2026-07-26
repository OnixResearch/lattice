## ADDED Requirements

### Requirement: Baselined Octet admission remains fail closed
r[lattice.repository_validation.baselined_octet_gate] Lattice MUST run every configured Octet lint without disabled or allowed entries through a complete warning-level observation and MUST reject every stable finding identity absent from the checked reviewed baseline.

#### Scenario: Reviewed identities remain admitted
r[lattice.repository_validation.baselined_octet_gate.scenario.accepted]
- GIVEN a successful complete Cargo child and a runner-generated baseline for the exact warning-level config identity
- WHEN the canonical Octet wrapper observes only reviewed stable finding identities
- THEN it SHALL report a nonempty current set, zero new identities, zero stale identities, and successful admission without claiming the accepted findings are correct or harmless

#### Scenario: A new finding fails the canonical gate
r[lattice.repository_validation.baselined_octet_gate.scenario.new_finding]
- GIVEN an isolated source fixture introduces a stable finding identity absent from the reviewed baseline
- WHEN the same canonical Octet wrapper evaluates the fixture without changing policy or baseline bytes
- THEN it SHALL exit unsuccessfully and report at least one new identity even though the warning-level Cargo child completes

#### Scenario: Observation and admission authority remain distinct
r[lattice.repository_validation.baselined_octet_gate.scenario.boundary]
- GIVEN direct warning-level Octet output and the baseline-aware canonical wrapper are both available
- WHEN repository validation or release checks run
- THEN only the wrapper SHALL grant finding-set admission, while direct output remains review evidence and MUST NOT imply correctness, complete analysis, traceability satisfaction, or release eligibility

## MODIFIED Requirements

### Requirement: Maintained traceability debt inventory
r[lattice.repository_validation.traceability_debt_inventory] Lattice MUST derive a deterministic, BLAKE3-bound inventory from the exact current Cairn missing-requirement set and an explicitly reviewed typed policy without promoting inventory rows into requirement evidence.

#### Scenario: Complete deterministic partition
r[lattice.repository_validation.traceability_debt_inventory.scenario.complete]
- GIVEN a Cairn coverage receipt with a policy-pinned missing set and no dangling markers
- WHEN the operator generates or checks the debt inventory
- THEN every missing identifier MUST appear exactly once in sorted subsystem and review-state summaries and repeated generation MUST produce the same inventory identity

#### Scenario: Stale or malformed inputs fail closed
r[lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
- GIVEN duplicate missing identifiers, a stale reviewed identifier, an unsafe evidence path, a changed missing-set identity, or a tampered report
- WHEN the inventory core validates or checks the inputs
- THEN it MUST emit bounded deterministic issue codes and MUST NOT report a valid inventory

#### Scenario: Unreviewed debt remains an explicit non-claim
r[lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]
- GIVEN a missing identifier has no explicit reviewed policy decision
- WHEN the inventory is generated
- THEN the row MUST remain `unassessed` and MUST NOT claim evidence absence, requirement satisfaction, semantic correctness, complete traceability, or release readiness

### Requirement: Repository-check evidence completion
r[lattice.repository_validation.repository_checks_completion] Lattice MUST maintain direct adversarial evidence for all 11 reviewed repository-validation rows, including current accepted byte identities, without promoting selected markers or changing unrelated linkage.

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

#### Scenario: Exactly 11 rows remain exact
r[lattice.repository_validation.repository_checks_completion.scenario.exact]
- GIVEN the reviewed 315-item inventory and the 11-ID repository-validation tranche
- WHEN completion validation runs after an accepted source transition
- THEN exactly those 11 rows MUST remain exact, BLAKE3-bound to current safe evidence, and marker-free while every unrelated row remains unchanged

#### Scenario: Repository-check completion drift fails closed
r[lattice.repository_validation.repository_checks_completion.scenario.fail_closed]
- GIVEN command, signing, source-filter, ignore, fixture, formatter, Clippy, classification, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
