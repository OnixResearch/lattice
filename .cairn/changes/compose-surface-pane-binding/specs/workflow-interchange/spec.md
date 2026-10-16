# Lattice Compose Surface Binding

## ADDED Requirements

### Requirement: Binding identity

r[lattice.compose_surface.identity] Every binding MUST use a BLAKE3 digest over a canonical projection of observation-source identity, workflow identity, trigger shape, and capability context. The projection MUST exclude ambient state that does not change the binding.

#### Scenario: Same inputs project to the same binding key

r[lattice.compose_surface.identity.scenario.equal]
- GIVEN two binding requests with equal source, workflow, trigger, and context
- WHEN the core projects both keys
- THEN the keys MUST be equal

#### Scenario: Distinct inputs project to distinct binding keys

r[lattice.compose_surface.identity.scenario.distinct]
- GIVEN two binding requests whose source, workflow, trigger, or context differs
- WHEN the core projects both keys
- THEN the keys MUST differ

### Requirement: Observation admission

r[lattice.compose_surface.admission] The core MUST admit pane-observation payloads against an explicit schema. The core MUST reject missing fields, unknown fields, and payloads outside explicit bounds.

#### Scenario: Well-formed observation admitted

r[lattice.compose_surface.admission.scenario.valid]
- GIVEN a payload that matches the explicit schema
- WHEN the core admits the payload
- THEN the core MUST return a validated binding input

#### Scenario: Malformed observation rejected

r[lattice.compose_surface.admission.scenario.malformed]
- GIVEN a payload with a missing field, an unknown field, or an out-of-bounds value
- WHEN the core admits the payload
- THEN the core MUST reject it with a structured error

### Requirement: Deterministic binding plan

r[lattice.compose_surface.plan] The core MUST decide the binding plan from supplied values only. The core MUST read no clock, environment, or product state. The plan MUST name effects and MUST NOT execute them.

#### Scenario: Equal inputs yield equal plans

r[lattice.compose_surface.plan.scenario.deterministic]
- GIVEN two equal binding inputs
- WHEN the core produces plans
- THEN the plans MUST be equal

#### Scenario: Plan is effect intent only

r[lattice.compose_surface.plan.scenario.intent]
- GIVEN a valid binding input and a bound workflow
- WHEN the core produces a plan
- THEN the plan MUST name effect intents only

### Requirement: Receipt emission

r[lattice.compose_surface.receipt] Each accepted binding activation MUST produce a metadata-only receipt with a BLAKE3 identity. The receipt MUST claim only the observed binding facts and MUST NOT claim observation truth, workflow correctness, or effect success.

#### Scenario: Activation produces a receipt

r[lattice.compose_surface.receipt.scenario.issued]
- GIVEN an accepted binding activation with exact source and workflow identities
- WHEN the shell publishes the receipt
- THEN the receipt MUST bind those exact identities

#### Scenario: Receipt over-claim fails

r[lattice.compose_surface.receipt.scenario.overclaim]
- GIVEN a receipt that claims workflow correctness
- WHEN the claim boundary is verified
- THEN the claim MUST fail unless the workflow owner makes it

### Requirement: Cross-repo contract

r[lattice.compose_surface.dependency] The change MUST record the exact Tile observation-contract revision it consumes. The claim boundary MUST state that Lattice does not own observation truth.

#### Scenario: Missing source revision

r[lattice.compose_surface.dependency.scenario.missing]
- GIVEN an adapter with no recorded source revision
- WHEN the dependency contract is checked
- THEN the check MUST fail

#### Scenario: Recorded source revision

r[lattice.compose_surface.dependency.scenario.recorded]
- GIVEN an adapter with an exact recorded revision
- WHEN the dependency contract is checked
- THEN the check MUST pass

### Requirement: Fixture coverage

r[lattice.compose_surface.verification] Positive, negative, and boundary fixtures MUST cover binding identity, admission, plan determinism, receipt claims, and the cross-repo contract.

#### Scenario: Complete focused matrix passes

r[lattice.compose_surface.verification.scenario.matrix]
- GIVEN decisions, fixtures, documentation, and dependency evidence are complete
- WHEN focused workspace, Clippy, Cairn, and Nix verification runs
- THEN in-bounds inputs MUST behave as declared
- AND every missing-field, unknown-field, over-claim, or unrevisioned-source input MUST fail as declared
