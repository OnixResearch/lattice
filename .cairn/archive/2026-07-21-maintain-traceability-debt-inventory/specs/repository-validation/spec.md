## ADDED Requirements

### Requirement: Maintained traceability debt inventory
r[lattice.repository_validation.traceability_debt_inventory] Lattice MUST derive a deterministic, BLAKE3-bound inventory from the exact Cairn missing-requirement set and an explicitly reviewed typed policy without promoting inventory rows into requirement evidence.

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
