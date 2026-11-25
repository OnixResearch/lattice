## ADDED Requirements

### Requirement: Explicit consumer release selection
r[lattice.release_integration.selection] Lattice MUST select its own typed Cairn release inputs, actual maintained external gates and executable consumer replay cases under an admitted immutable producer cohort without weakening obligations.

#### Scenario: Truthful maintained gates
r[lattice.release_integration.selection.gates]
- GIVEN Lattice release policy
- WHEN readiness reports required external commands
- THEN it MUST list the maintained Lattice checks rather than foreign producer-only gates

### Requirement: Real bounded release inputs
r[lattice.release_integration.inputs] Lattice MUST produce MCP smoke inputs from observed native validation and execute canonicalization freshness on maintained consumer-owned vectors, retaining truthful non-passing observations.

#### Scenario: Real MCP observation
r[lattice.release_integration.inputs.mcp]
- GIVEN exact native Cairn and explicit policy
- WHEN release inputs are produced
- THEN status and evidence MUST derive from the actual validation result and selected policy

#### Scenario: Stale corpus rejected
r[lattice.release_integration.inputs.corpus]
- GIVEN a consumer canonicalization vector with stale expected hash
- WHEN freshness executes
- THEN it MUST fail without mutating the vector
