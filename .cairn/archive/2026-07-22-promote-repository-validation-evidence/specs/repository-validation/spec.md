## ADDED Requirements

### Requirement: Repository-check evidence promotion
r[lattice.repository_validation.repository_checks_promotion] Lattice MUST promote only the 11 reviewed exact repository-validation rows through markers attached to maintained assertions and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact 11-ID repository-check delta
r[lattice.repository_validation.repository_checks_promotion.scenario.delta]
- GIVEN the failing 315-item completion receipt and the explicit 11-ID repository-validation allowlist
- WHEN maintained promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 304

#### Scenario: Repository-check markers remain locally bound
r[lattice.repository_validation.repository_checks_promotion.scenario.bindings]
- GIVEN every selected row has exact maintained signing, source-closure, formatter, or Clippy assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on configuration names, file scope, generated inventory, lifecycle tests, or archived evidence

#### Scenario: Repository-check promotion drift fails closed
r[lattice.repository_validation.repository_checks_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing binding, stale reviewed row, unexpected removal or addition, dangling marker, or successful coverage
- WHEN maintained promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion
