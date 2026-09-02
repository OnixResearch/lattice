# Choregraph Effect Authority Specification Delta

## Purpose

Preserve exact Kamacite semantic operation identity across Lattice effect handoff, authority, persistence, dispatch, observation, and replay boundaries.

## ADDED Requirements

### Requirement: Lattice pins one semantic operation cohort

r[lattice.choregraph_effect_authority.semantic_operation_profile] Lattice MUST pin one immutable Kamacite semantic operation, compatibility, codec, and identity-domain cohort for selected cross-stack effect handoffs.

#### Scenario: Exact cohort is admitted

- GIVEN every required revision, schema, codec, and identity-domain value matches the reviewed cohort
- WHEN semantic operation profile admission runs
- THEN Lattice MUST admit the cohort for selected effect families

#### Scenario: Cohort value drifts

- GIVEN any required Kamacite cohort value differs
- WHEN profile admission runs
- THEN Lattice MUST deny the semantic handoff before product dispatch

### Requirement: Explicit registry resolves operation identity

r[lattice.choregraph_effect_authority.semantic_operation_registry] Lattice MUST resolve complete admitted Choregraph operation facts through a reviewed registry to one exact Kamacite `effect-operation` identity.

#### Scenario: Complete registry row matches

- GIVEN one registry row matches the exact Choregraph source revision, machine role, operation facts, and descriptor identity
- WHEN semantic operation resolution runs
- THEN Lattice MUST return one Lattice-owned semantic operation reference

#### Scenario: Name-only mapping is supplied

- GIVEN a registry row uses only an operation name, schema shape, or payload hash
- WHEN registry validation runs
- THEN Lattice MUST reject the row instead of deriving semantic identity

### Requirement: Authority chain binds one semantic identity

r[lattice.choregraph_effect_authority.semantic_operation_authority] The Choregraph intent, Basalt request, selected Basalt rule, Basalt decision, and Lattice dispatch decision MUST bind one exact semantic operation identity.

#### Scenario: Complete authority chain matches

- GIVEN every authority input and decision references the same admitted semantic operation identity
- WHEN pre-effect binding validation runs
- THEN Lattice MAY continue to existing currentness and durable attempt gates

#### Scenario: Basalt decision references another identity

- GIVEN local operation labels match but the Basalt decision identity differs
- WHEN pre-effect binding validation runs
- THEN Lattice MUST deny without invoking the product adapter

### Requirement: Durable attempts retain semantic identity

r[lattice.choregraph_effect_authority.semantic_operation_attempt] Lattice MUST persist the semantic operation identity and any admitted compatibility identity in the bound attempt before product adapter invocation.

#### Scenario: Attempt commit succeeds

- GIVEN authority and currentness pass with one exact semantic identity
- WHEN attempt persistence commits
- THEN the stored attempt MUST contain that identity before dispatch

#### Scenario: Identity persistence fails

- GIVEN the attempt cannot persist the complete semantic identity fields
- WHEN dispatch preparation runs
- THEN no product effect MUST execute

### Requirement: Compatibility is exact and directional

r[lattice.choregraph_effect_authority.semantic_operation_compatibility] Lattice MAY dispatch a different target identity only through an exact Kamacite compatibility artifact for the requested direction and live execution context.

#### Scenario: Exact live compatibility passes

- GIVEN compatibility binds the exact source and target identities in the requested direction and permits the selected live context
- WHEN compatibility admission runs
- THEN Lattice MAY continue subject to all authority and adapter gates

#### Scenario: Replay compatibility attempts live dispatch

- GIVEN compatibility is replay-only, reverse, stale, or for another context
- WHEN live dispatch admission runs
- THEN Lattice MUST deny before persistence or adapter invocation

### Requirement: Terminal observations retain operation identity

r[lattice.choregraph_effect_authority.semantic_operation_observation] Success, failure, unknown, recovery, and abandonment records MUST retain the exact persisted semantic operation and compatibility identities.

#### Scenario: Adapter returns matching observation

- GIVEN the product adapter returns a bounded observation for the persisted operation identity
- WHEN terminal recording runs
- THEN Lattice MUST retain the identity with the observation and execution non-claims

#### Scenario: Adapter observation identity differs

- GIVEN the adapter reports another or missing semantic identity
- WHEN terminal recording runs
- THEN Lattice MUST classify the result as invalid or unknown and MUST NOT report successful matched execution

### Requirement: Replay cannot create fresh semantic dispatch

r[lattice.choregraph_effect_authority.semantic_operation_replay] Replay MUST reconstruct recorded semantic identities without resolving new registry entries, obtaining fresh authority, or invoking product adapters.

#### Scenario: Historical allowed identity is replayed

- GIVEN history contains an allowed semantic operation and terminal observation
- WHEN replay reconstructs the run
- THEN it MUST retain the recorded identities and MUST execute no product effect

### Requirement: Verification covers two real effect families

r[lattice.choregraph_effect_authority.semantic_operation_verification] Verification MUST cover one process effect and one Animus agent effect through registry, Basalt binding, durable attempt, adapter, terminal observation, recovery, and replay.

#### Scenario: End-to-end semantic identity rail runs

- GIVEN current producer cohorts and positive and negative fixtures
- WHEN the focused rail runs
- THEN accepted cases MUST retain one identity and every mismatch case MUST produce no dependent effect
