# Choregraph Verified Machine Adoption

## MODIFIED Requirements

### Requirement: Accepted prerequisite baselines
r[lattice.choregraph_adoption.prerequisites] Lattice MUST complete the selected immutable execution-graph and Nickel evaluator baselines before Choregraph cutover.

#### Scenario: Baseline change remains active
r[lattice.choregraph_adoption.prerequisites.scenario.active]
- GIVEN an unresolved prerequisite source migration
- WHEN Choregraph cutover readiness runs
- THEN cutover MUST remain blocked.

### Requirement: Complete call-site inventory
r[lattice.choregraph_adoption.inventory] The migration MUST inventory every graph, readiness, lifecycle, resume, replay, persisted-state, event, and effect-intent call site.

#### Scenario: Production call site is missing
r[lattice.choregraph_adoption.inventory.scenario.missing]
- GIVEN one current semantic call site without a migration case
- WHEN readiness is checked
- THEN cutover MUST fail.

### Requirement: Exact Choregraph source admission
r[lattice.choregraph_adoption.source] Lattice MUST accept only an exact Choregraph source with compatible license, release bundle, proof, trust, canonical artifact, frontend, and conformance evidence.

#### Scenario: Proof identity differs from source
r[lattice.choregraph_adoption.source.scenario.identity]
- GIVEN Choregraph proof evidence for another source identity
- WHEN dependency admission runs
- THEN Choregraph MUST be rejected.

### Requirement: Cargo and Nix source alignment
r[lattice.choregraph_adoption.source_alignment] Cargo and Nix MUST resolve the same exact Choregraph revision without floating, GitHub, sibling, vendor, or runtime fallback.

#### Scenario: Source revisions differ
r[lattice.choregraph_adoption.source_alignment.scenario.drift]
- GIVEN different Cargo and Nix revisions
- WHEN source checks run
- THEN validation MUST fail.

### Requirement: Pure lowering integration
r[lattice.choregraph_adoption.lowering] Lattice MUST pass in-memory evaluated values to Choregraph's pure lowerer and MUST retain source discovery, evaluator, import, contract, fallback, and safe-diagnostic policy.

#### Scenario: Lowerer reads Lattice files
r[lattice.choregraph_adoption.lowering.scenario.filesystem]
- GIVEN a lowering path that performs filesystem I/O
- WHEN architecture checks run
- THEN the integration MUST fail.

### Requirement: Conversion-only machine adapter
r[lattice.choregraph_adoption.adapter] The Lattice adapter MUST delegate graph and transition semantics to Choregraph without independent algorithms.

#### Scenario: Adapter computes lifecycle transition
r[lattice.choregraph_adoption.adapter.scenario.duplicate]
- GIVEN independent lifecycle logic in the adapter
- WHEN duplicate checks run
- THEN cutover MUST fail.

### Requirement: Complete behavior parity
r[lattice.choregraph_adoption.parity] Current and Choregraph paths MUST dual-run admission, ordering, dependency, group, readiness, lifecycle, terminal, resume, replay, and intent behavior.

#### Scenario: One decision differs
r[lattice.choregraph_adoption.parity.scenario.mismatch]
- GIVEN equal supported inputs and unequal decisions
- WHEN parity runs
- THEN migration MUST remain blocked unless a semantic change is separately accepted.

### Requirement: Persisted compatibility
r[lattice.choregraph_adoption.persistence] Persisted state and event migration MUST bind old schema, new schema, workflow revision, and machine identity and MUST fail closed on unsupported records.

#### Scenario: Persisted record is stale
r[lattice.choregraph_adoption.persistence.scenario.stale]
- GIVEN a record bound to another workflow or machine identity
- WHEN migration runs
- THEN the record MUST NOT create current execution state.

### Requirement: Replay has no dispatch authority
r[lattice.choregraph_adoption.replay] Replaying stored events through Choregraph MUST compute state without dispatching historical effect intents.

#### Scenario: Historical intent is replayed
r[lattice.choregraph_adoption.replay.scenario.intent]
- GIVEN a stored event that once requested an effect
- WHEN replay runs
- THEN no fresh runtime invocation MUST occur.

### Requirement: One production transition path
r[lattice.choregraph_adoption.single_implementation] After cutover, Lattice MUST use Choregraph as the sole graph and machine transition implementation and MUST remove temporary duplicate paths.

#### Scenario: Legacy path remains callable
r[lattice.choregraph_adoption.single_implementation.scenario.legacy]
- GIVEN completed cutover with a callable legacy decision path
- WHEN duplicate checks run
- THEN release readiness MUST fail.

### Requirement: Product authority remains in Lattice
r[lattice.choregraph_adoption.authority] Lattice MUST retain product contracts, runtime selection, values, ports, credentials, effects, persistence, retries, schedules, triggers, budgets, diagnostics, UI, and evidence authority.

#### Scenario: Choregraph accepts a transition
r[lattice.choregraph_adoption.authority.scenario.effect]
- GIVEN an accepted pure machine decision
- WHEN Lattice handles its intents
- THEN acceptance MUST NOT imply runtime authorization or effect success.

### Requirement: Tested rollback
r[lattice.choregraph_adoption.rollback] A reviewed VCS rollback MUST restore dependencies, generated locks, adapters, and prior behavior without hidden fallback.

#### Scenario: Adoption is reverted
r[lattice.choregraph_adoption.rollback.scenario.baseline]
- GIVEN the recorded pre-cutover revision
- WHEN rollback runs
- THEN the prior Lattice path MUST build and pass its baseline fixtures.

### Requirement: Maintained adoption validation
r[lattice.choregraph_adoption.validation] Maintained checks MUST include parity, persistence, replay, source, duplicate, rollback, Octet, Cairn, and Nix evidence.

#### Scenario: Persisted migration checks are skipped
r[lattice.choregraph_adoption.validation.scenario.persistence]
- GIVEN passing in-memory parity without persisted compatibility evidence
- WHEN readiness runs
- THEN cutover MUST fail.
