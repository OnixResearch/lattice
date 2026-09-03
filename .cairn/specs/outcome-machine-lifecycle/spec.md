# Outcome Machine Lifecycle Specification

## Purpose

Defines the `outcome-machine-lifecycle` capability.

## Requirements

### Requirement: Lattice owns outcome-machine lifecycle admission

r[lattice.lifecycle.outcome_machine_profile] Lattice MUST pin the published `outcome-machine` Cairn revision. Profile-selected changes MUST retain BLAKE3-bound rejection and completion evidence without transferring orchestration, effect, evidence, or release authority.

#### Scenario: Planned workflow change attempts sync

- GIVEN a Lattice change has a complete delivery plan
- AND repository completion evidence remains planned
- WHEN the repository-owned Cairn runner evaluates sync
- THEN sync MUST remain blocked
- AND agent output or task activity MUST NOT override that result

#### Scenario: Repository evidence completes admission

- GIVEN the selected Lattice checks pass
- AND the change binds that observation with fresh-state evidence
- WHEN Cairn reevaluates the change
- THEN every BLAKE3 evidence identity MUST match
- AND Cairn MUST NOT claim Lattice release eligibility
