# Repository Validation Specification

## ADDED Requirements

### Requirement: Aligned Cairn producer
r[lattice.repository_validation.cairn_producer_alignment] Lattice MUST pin one exact public Cairn revision for its native binary and its upstream policy snapshots. Lattice's exported policy MUST stay accepted by that revision's native binary without weakening any Lattice-owned policy.

#### Scenario: Policy accepted by the pinned producer
r[lattice.repository_validation.cairn_producer_alignment.scenario.policy_accepted]
- GIVEN the pinned native Cairn binary and Lattice's Nickel policy source
- WHEN `policy export --check` runs
- THEN the generated policy MUST be fresh and the binary MUST load it

#### Scenario: Lattice's release gate list is selected
r[lattice.repository_validation.cairn_producer_alignment.scenario.gates_selected]
- GIVEN Lattice's declared `required_external_commands`
- WHEN the pinned binary assesses release readiness
- THEN the receipt MUST report `consumer-selected-gates` with exactly that list, not the producer's maintained gates
