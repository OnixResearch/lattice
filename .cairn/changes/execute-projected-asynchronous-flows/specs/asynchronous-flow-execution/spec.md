# Asynchronous Flow Execution Specification Delta

## ADDED Requirements

### Requirement: Asynchronous flow intake is exact

r[lattice.async_flow.intake] Lattice MUST consume one immutable Choregraph global flow, complete ordered local-flow set, projection receipt, schema cohort, assumption set, and Trellis law-evidence cohort through strict Lattice-owned adapters.

#### Scenario: Current complete artifacts are admitted

r[lattice.async_flow.intake.valid]
- GIVEN all producer and proof artifacts have exact supported identities and contain one local flow for every declared role
- WHEN Lattice evaluates intake
- THEN it MUST produce one strict internal flow cohort
- AND it MUST NOT recompile or infer missing producer semantics.

#### Scenario: One producer artifact is stale

r[lattice.async_flow.intake.stale]
- GIVEN one schema, local flow, projection, proof, or assumption identity differs from the selected cohort
- WHEN Lattice evaluates intake
- THEN it MUST reject the complete intake before session persistence.

### Requirement: Flow sessions are complete and bounded

r[lattice.async_flow.session] A flow session MUST bind every declared role, local flow, operator, edge, adapter, and named runtime limit before execution. Limit exhaustion MUST produce a typed bounded outcome and MUST NOT silently drop input.

#### Scenario: Complete bounded plan is created

r[lattice.async_flow.session.valid]
- GIVEN an admitted flow cohort, complete role placement, supported adapters, and valid named limits
- WHEN session planning runs
- THEN Lattice MUST produce one deterministic session identity and initial state.

#### Scenario: Placement or a runtime limit is invalid

r[lattice.async_flow.session.invalid]
- GIVEN one role is missing, duplicated, or mapped through an unsupported adapter, or one named limit is invalid
- WHEN session planning runs
- THEN Lattice MUST reject the plan before persistence or dispatch.

### Requirement: Edge semantics govern admission

r[lattice.async_flow.edge_semantics] Lattice MUST preserve each edge closure, logical-ordering, repetition, completeness, source, destination, payload-schema, and failure-model declaration. It MUST reject an operator that lacks the required compatibility evidence.

#### Scenario: Unordered input reaches an order-insensitive operator

r[lattice.async_flow.edge_semantics.unordered]
- GIVEN an unordered edge and exact order-insensitivity evidence for the selected operator and domain
- WHEN the next item is admitted
- THEN Lattice MAY process that item without inventing a total order.

#### Scenario: Unordered input reaches concatenation

r[lattice.async_flow.edge_semantics.order_sensitive]
- GIVEN an unordered edge and an order-sensitive concatenation operator
- WHEN compatibility admission runs
- THEN Lattice MUST reject the flow before execution.

### Requirement: Algebraic evidence is exact

r[lattice.async_flow.law_evidence] Every algebraic law reference MUST bind the selected operation implementation, admitted domain, property, proof source, verifier cohort, assumptions, and evidence identity. Empty marker traits and unbound assertions MUST NOT satisfy admission.

#### Scenario: Duplicate-bearing set union has exact evidence

r[lattice.async_flow.law_evidence.valid]
- GIVEN a duplicate-bearing edge and a set-union operator with exact associative, commutative, and idempotent evidence
- WHEN law admission runs
- THEN Lattice MAY admit the operator for the covered domain.

#### Scenario: Evidence covers another implementation

r[lattice.async_flow.law_evidence.mismatch]
- GIVEN a law reference names a different operation identity, domain, verifier, assumption set, or source revision
- WHEN law admission runs
- THEN Lattice MUST reject the reference as stale or incompatible.

### Requirement: Closure is distinct from resource bounds

r[lattice.async_flow.closure] Lattice MUST represent `Closed` and `Live` logical inputs separately from runtime resource limits. A completion-dependent operator on live input MUST require an explicit bounded window or checkpoint closure.

#### Scenario: Closed input completes a final fold

r[lattice.async_flow.closure.closed]
- GIVEN an admitted closed edge with one valid closure marker and complete prior input accounting
- WHEN a completion-dependent operator evaluates readiness
- THEN Lattice MAY release that operator.

#### Scenario: A live edge reaches its queue limit

r[lattice.async_flow.closure.limit]
- GIVEN a live edge reaches a named queue or retained-state limit
- WHEN limit handling runs
- THEN Lattice MUST report a bounded blocked or failed outcome
- AND it MUST NOT treat the edge as logically closed.

### Requirement: Flow state is persisted before dispatch

r[lattice.async_flow.persistence] Lattice MUST persist the exact session, edge, item, position, operator, attempt, adapter, and authority references before any transport or protected effect starts.

#### Scenario: Persistence succeeds before dispatch

r[lattice.async_flow.persistence.valid]
- GIVEN a pure decision emits one admitted dispatch plan
- WHEN the shell prepares the action
- THEN it MUST persist the complete prepared record before calling the selected adapter.

#### Scenario: Persistence fails

r[lattice.async_flow.persistence.failure]
- GIVEN the prepared record cannot be persisted completely
- WHEN the shell handles the failure
- THEN it MUST NOT call transport or execute the protected effect.

### Requirement: Dispatch uncertainty never authorizes hidden retry

r[lattice.async_flow.uncertainty] A dispatch that can have reached an external boundary MUST remain explicit uncertainty until an admitted observation settles it. Lattice MUST NOT resend automatically or report logical delivery from transport success alone.

#### Scenario: Crash follows possible dispatch

r[lattice.async_flow.uncertainty.crash]
- GIVEN the adapter can have started an external action and the process crashes before an admitted terminal observation
- WHEN recovery loads the attempt
- THEN the attempt MUST remain uncertain
- AND no automatic retry MUST occur.

#### Scenario: Transport reports success without receiver evidence

r[lattice.async_flow.uncertainty.transport_success]
- GIVEN a transport adapter reports local success but no required receiver observation exists
- WHEN logical outcome classification runs
- THEN Lattice MUST NOT report logical delivery or exactly-once execution.

### Requirement: Protected effect release needs stronger evidence

r[lattice.async_flow.effect_release] Lattice MUST release a protected effect only after required input closure, selected window closure, or exact prefix-safety evidence. Monotonicity alone MUST NOT grant effect authority.

#### Scenario: Prefix-safe internal result advances

r[lattice.async_flow.effect_release.internal]
- GIVEN a monotone partial result and no protected external effect
- WHEN the operator evaluates the next prefix
- THEN Lattice MAY update internal session knowledge within its bounds.

#### Scenario: Partial result requests an irreversible effect

r[lattice.async_flow.effect_release.denied]
- GIVEN a live partial result lacks exact prefix-safety evidence for its protected effect plan
- WHEN effect release is evaluated
- THEN Lattice MUST deny the effect until an admitted closure or coordination fact exists.

### Requirement: Nondeterminism assumptions remain visible

r[lattice.async_flow.assumptions] Lattice MUST bind each selected nondeterminism assumption to affected operators, edges, expected observational relation, ChaosControl profile, runtime state, diagnostics, and evidence.

#### Scenario: Assumption is admitted and recorded

r[lattice.async_flow.assumptions.valid]
- GIVEN a current Choregraph assumption has a supported expected relation and selected validation profile
- WHEN the flow session is created
- THEN Lattice MUST retain that assumption through execution, inspection, replay, and evidence.

#### Scenario: Assumption is absent or stale

r[lattice.async_flow.assumptions.stale]
- GIVEN an operator requires an assumption that is missing, stale, or incompatible
- WHEN session admission runs
- THEN Lattice MUST reject the flow before execution.

### Requirement: Replay remains effect-free

r[lattice.async_flow.replay] Replay MUST recompute flow state, closure, windows, outcomes, and effect-release decisions from persisted records without transport, retry, storage mutation, credential resolution, or protected effects.

#### Scenario: Flow replay is requested

r[lattice.async_flow.replay.valid]
- GIVEN one admitted persisted flow session
- WHEN replay runs
- THEN it MUST return deterministic state and diagnostics
- AND it MUST emit no dispatchable external plan.

### Requirement: The first pilot uses proof-linked set union

r[lattice.async_flow.pilot] The first maintained pilot MUST reduce bounded item sets over unordered duplicate-bearing input through one exact associative, commutative, and idempotent set-union profile.

#### Scenario: Reordered duplicate inputs converge

r[lattice.async_flow.pilot.valid]
- GIVEN two admitted executions contain the same logical item sets with different order and duplicate observations
- WHEN the set-union pilot closes with complete observation accounting
- THEN both executions MUST produce the same canonical union result.

#### Scenario: Duplicate-sensitive counting is substituted

r[lattice.async_flow.pilot.invalid]
- GIVEN the pilot replaces set union with duplicate-sensitive counting without deduplication evidence
- WHEN compatibility admission runs
- THEN Lattice MUST reject the flow.

### Requirement: Flow evidence stays bounded

r[lattice.async_flow.evidence] Lattice MUST emit domain-separated BLAKE3 identities and bounded evidence for producer artifacts, law evidence, session state, adapters, observations, uncertainty, assumptions, replay, limits, and non-claims.

#### Scenario: Evidence omits a required observation gap

r[lattice.async_flow.evidence.incomplete]
- GIVEN required observation accounting contains a gap, overflow, truncation, or failed final drain
- WHEN evidence classification runs
- THEN the result MUST remain incomplete or invalid
- AND it MUST NOT become a passing flow result.

### Requirement: Runtime authority remains bounded

r[lattice.async_flow.boundary] Passing flow admission or execution MUST NOT claim transport delivery, remote durability, external-role correctness, exactly-once effects, universal determinism, universal convergence, production readiness, or release eligibility.

#### Scenario: Passing pilot is promoted to exactly-once delivery

r[lattice.async_flow.boundary.overclaim]
- GIVEN the bounded set-union pilot and all local checks pass
- WHEN evidence is presented as transport-level exactly-once delivery
- THEN Lattice MUST reject that claim as outside its evidence scope.

### Requirement: Asynchronous flow validation is maintained

r[lattice.async_flow.validation] Maintained validation MUST cover positive and negative intake, edge compatibility, algebra, closure, persistence, uncertainty, effect release, assumptions, pilot, replay, evidence, architecture, formatting, Clippy, Octet, Cairn, and bounded Nix cases.

#### Scenario: One compatibility guard regresses

r[lattice.async_flow.validation.regression]
- GIVEN a source or fixture change admits one previously rejected incompatible flow
- WHEN maintained validation runs
- THEN the validation rail MUST fail before sync or archive.
