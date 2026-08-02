# Repository Validation Delta: Compiler-enforced FCIS boundaries

## ADDED Requirements

### Requirement: Lattice product decisions have a compiler-enforced core

r[lattice.repository_validation.fcis_crate_boundary] Lattice MUST place the selected workflow, scheduler-admission, replay, and orchestration decisions in a host-capability-free core crate whose dependency graph cannot reach store, filesystem, environment, clock, process, network, provider, async-runtime, CLI, or UI authority.

#### Scenario: Core decisions run from supplied facts

r[lattice.repository_validation.fcis_crate_boundary.scenario.pure]
- GIVEN owned workflow, schedule, occurrence, replay, or orchestration state and explicit input facts
- WHEN a selected core decision runs
- THEN it MUST return deterministic data without opening a store, reading a host clock, inspecting environment state, performing I/O, starting a process, or mutating UI state

#### Scenario: Host authority enters the core graph

r[lattice.repository_validation.fcis_crate_boundary.scenario.forbidden]
- GIVEN the core crate or one of its non-adapter dependencies imports a forbidden host capability or shell crate
- WHEN architecture validation runs
- THEN validation MUST fail with a deterministic diagnostic that names the dependency path and forbidden authority class

### Requirement: Temporal and execution observations are explicit core facts

r[lattice.repository_validation.fcis_explicit_facts] Core constructors and transitions whose result depends on time, persistence, provider execution, or process execution MUST receive typed observations as explicit inputs and MUST NOT obtain those observations from ambient services.

#### Scenario: Workflow metadata changes at a supplied time

r[lattice.repository_validation.fcis_explicit_facts.scenario.timestamp]
- GIVEN a caller supplies valid workflow state and an explicit timestamp
- WHEN the core creates or touches workflow metadata
- THEN the resulting created and modified values MUST derive only from the supplied timestamp and deterministic state

#### Scenario: Shell observation is absent or contradictory

r[lattice.repository_validation.fcis_explicit_facts.scenario.invalid]
- GIVEN a transition requires a terminal, persistence, or temporal observation that is absent, stale, foreign, or contradictory
- WHEN core admission runs
- THEN it MUST reject without fabricating an observation or executing a host effect

### Requirement: FCIS extraction preserves accepted behavior and identity

r[lattice.repository_validation.fcis_compatibility] The crate extraction MUST preserve accepted workflow, scheduler occurrence, replay, orchestration, persistence, command, canonical serialization, and BLAKE3 identity behavior unless a separate versioned change approves a difference.

#### Scenario: Accepted records cross the new boundary

r[lattice.repository_validation.fcis_compatibility.scenario.accepted]
- GIVEN an accepted workflow, schedule occurrence, replay bundle, or orchestration fixture
- WHEN legacy shell-facing adapters and the extracted core process the fixture
- THEN public fields, canonical bytes, identities, decisions, diagnostics, and terminal outcomes MUST remain equal

#### Scenario: Invalid records remain rejected

r[lattice.repository_validation.fcis_compatibility.scenario.rejected]
- GIVEN a stale, duplicate, malformed, foreign, over-limit, or contradictory fixture
- WHEN the extracted core and compatibility adapters process it
- THEN they MUST reject without partial persistence, effect dispatch, or identity drift

### Requirement: Architecture policy guards dependency direction

r[lattice.repository_validation.fcis_architecture_guard] Lattice MUST declare and run reviewed core, shell, adapter, forbidden-dependency, and target scopes through its canonical Octet architecture gate, and new core violations MUST NOT enter the accepted baseline.

#### Scenario: Declared topology remains clean

r[lattice.repository_validation.fcis_architecture_guard.scenario.accepted]
- GIVEN the core depends only on reviewed deterministic crates and shells depend inward through explicit adapters
- WHEN canonical Octet and repository checks run on the declared targets
- THEN architecture admission MUST succeed and report the exact evaluated scope

#### Scenario: A new violation is proposed for baseline admission

r[lattice.repository_validation.fcis_architecture_guard.scenario.no-new-debt]
- GIVEN a candidate adds a forbidden core dependency, reverses dependency direction, or exposes a shell-owned type through the core API
- WHEN baseline-aware architecture admission runs
- THEN it MUST fail and MUST NOT accept the violation as new reviewed debt

### Requirement: FCIS evidence retains bounded claims

r[lattice.repository_validation.fcis_claim_boundary] Lattice MUST report FCIS checks as evidence of the evaluated source shape and dependency direction only.

#### Scenario: All architecture checks pass

r[lattice.repository_validation.fcis_claim_boundary.scenario.non-claim]
- GIVEN core tests, shell compatibility tests, and architecture gates pass
- WHEN evidence is summarized
- THEN it MUST NOT claim workflow correctness, exactly-once execution, provider correctness, host sandboxing, semantic determinism outside supplied facts, or release readiness
