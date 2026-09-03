# Linear Process Permit Delta

## ADDED Requirements

### Requirement: Lattice binds the published producer chain

r[lattice.linear_process_permit.dependencies] Lattice MUST bind the exact Choregraph resource-flow handoff and its linked Trellis and Kamacite profile identities.

#### Scenario: The complete producer chain matches

- GIVEN the handoff binds accepted immutable producer revisions and identities
- WHEN pilot intake runs
- THEN Lattice MUST admit the producer chain

#### Scenario: Any producer identity is stale or ambient

- GIVEN a required source, profile, proof, schema, graph, machine, or transition identity differs or lacks publication
- WHEN pilot intake runs
- THEN Lattice MUST reject before authorization or execution

### Requirement: The process pilot is explicit and narrow

r[lattice.linear_process_permit.profile] Only an explicit supported local shell-node profile MUST enable process-permit behavior.

#### Scenario: A shell node selects the pilot

- GIVEN the workflow and Choregraph handoff select the supported pilot profile
- WHEN workflow admission runs
- THEN Lattice MUST apply the process-permit contract

#### Scenario: A legacy or other effect omits the pilot

- GIVEN a workflow uses a legacy, file, network, agent, provider, credential, or remote effect path
- WHEN workflow admission runs
- THEN existing behavior and claim scope MUST remain unchanged

### Requirement: Lattice does not repeat graph admission

r[lattice.linear_process_permit.intake] Lattice MUST validate exact closed-row and transition identities. It MUST NOT run generic row unification, fork partition, or join admission.

#### Scenario: The Choregraph handoff is exact

- GIVEN the handoff contains supported closed rows and transition identities
- WHEN Lattice intake runs
- THEN Lattice MUST preserve those identities without semantic reinterpretation

#### Scenario: The handoff contains an unresolved row

- GIVEN a runtime handoff contains a variable or unsupported graph construct
- WHEN Lattice intake runs
- THEN Lattice MUST reject the handoff

### Requirement: The core models explicit permit states

r[lattice.linear_process_permit.state] The pure core MUST model `Available`, `Owned`, `Settled`, and `Unknown` states with exact linked identities.

#### Scenario: A permit is available

- GIVEN an admitted unused permit occurrence
- WHEN the core inspects its state
- THEN the state MUST contain no attempt, observation, or uncertainty identity

#### Scenario: A permit is owned

- GIVEN durable preparation succeeded for one exact attempt
- WHEN the core applies the preparation fact
- THEN the state MUST become `Owned` for that attempt

### Requirement: Permit transitions are deterministic

r[lattice.linear_process_permit.transition] Every permit transition MUST depend only on supplied state, intent, authorization, persistence, observation, and recovery facts.

#### Scenario: Equal inputs repeat

- GIVEN the same admitted state and facts
- WHEN the core evaluates the transition twice
- THEN both decisions and resulting states MUST be equal

#### Scenario: A transition lacks a required fact

- GIVEN preparation, settlement, uncertainty, or recovery lacks an exact required identity
- WHEN the core evaluates the transition
- THEN it MUST reject and preserve the original state

### Requirement: One permit occurrence is single-use

r[lattice.linear_process_permit.single_use] One permit occurrence MUST bind at most one durable attempt. `Settled` and `Unknown` MUST be terminal for that occurrence.

#### Scenario: One available permit binds one attempt

- GIVEN a permit is `Available`
- AND the exact preparation facts pass
- WHEN ownership admission runs
- THEN the permit MUST become `Owned` by one attempt

#### Scenario: A second attempt uses the same permit

- GIVEN a permit is already `Owned`, `Settled`, or `Unknown`
- WHEN another attempt claims the same occurrence
- THEN ownership admission MUST reject

#### Scenario: An unknown permit is restored automatically

- GIVEN a permit is `Unknown`
- WHEN automatic runtime logic requests `Available`
- THEN the transition MUST reject

### Requirement: Authorization precedes process preparation

r[lattice.linear_process_permit.authorization] Lattice MUST require accepted Choregraph intake and accepted Basalt authorization before durable process preparation.

#### Scenario: Intake and authorization pass

- GIVEN the handoff is exact
- AND Basalt allows the supplied actor, ability, resource, request, and scope facts
- WHEN preparation begins
- THEN the preparation decision MUST permit the durable `Owned` commit

#### Scenario: Authorization denies

- GIVEN Choregraph intake passes
- AND Basalt denies the effect
- WHEN Lattice evaluates dispatch
- THEN Lattice MUST start no process
- AND it MUST not create an owned process attempt

### Requirement: Durable ownership precedes execution

r[lattice.linear_process_permit.ordering] Lattice MUST commit exact `Owned` permit and attempt state before it invokes Bounded Exec.

#### Scenario: The ownership commit succeeds

- GIVEN intake and authorization pass
- WHEN storage commits exact ownership
- THEN the dispatch decision MUST permit the selected Bounded Exec plan

#### Scenario: The ownership commit fails

- GIVEN storage rejects the ownership commit
- WHEN shell orchestration handles the result
- THEN Lattice MUST not invoke Bounded Exec

#### Scenario: The ownership commit is uncertain

- GIVEN storage cannot confirm whether ownership committed
- WHEN shell orchestration handles the result
- THEN Lattice MUST reconcile storage before any process execution

### Requirement: Bounded Exec retains process mechanics

r[lattice.linear_process_permit.execution] Lattice MUST use the pinned Bounded Exec adapter for child-process execution and teardown.

#### Scenario: An owned attempt starts

- GIVEN exact owned state and an admitted process plan
- WHEN Lattice executes the effect
- THEN it MUST invoke the pinned Bounded Exec adapter

#### Scenario: A code path bypasses Bounded Exec

- GIVEN the pilot attempts direct child-process spawning
- WHEN architecture validation runs
- THEN validation MUST reject the implementation

### Requirement: Settlement requires an exact terminal observation

r[lattice.linear_process_permit.settlement] Lattice MUST settle only an exact owned attempt with a trustworthy terminal observation.

#### Scenario: The process succeeds

- GIVEN Bounded Exec returns a complete terminal success observation for the owned attempt
- WHEN settlement admission runs
- THEN the permit MUST become `Settled` with the exact observation identity

#### Scenario: The process fails terminally

- GIVEN Bounded Exec returns a complete terminal failure observation for the owned attempt
- WHEN settlement admission runs
- THEN the permit MUST become `Settled` with the exact observation identity

#### Scenario: The observation is stale or foreign

- GIVEN the observation names another attempt, intent, transition, or process profile
- WHEN settlement admission runs
- THEN settlement MUST reject and preserve `Owned`

### Requirement: Untrustworthy outcomes become unknown

r[lattice.linear_process_permit.uncertainty] Missing, stale, foreign, conflicting, or untrustworthy terminal evidence MUST produce explicit `Unknown` state for the owned attempt.

#### Scenario: No terminal observation exists

- GIVEN an owned process attempt has no trustworthy terminal observation
- WHEN uncertainty admission runs
- THEN the permit MUST become `Unknown`
- AND Lattice MUST record the exact uncertainty identity

#### Scenario: Runtime restarts with an unknown permit

- GIVEN persistence contains exact `Unknown` state
- WHEN Lattice replays the run
- THEN replay MUST restore `Unknown`
- AND it MUST emit no automatic process intent

### Requirement: Recovery is explicit and history-preserving

r[lattice.linear_process_permit.recovery] Recovery MUST retain the original attempt and unknown evidence. A fresh retry MUST use a new permit and attempt identity.

#### Scenario: A later exact observation arrives

- GIVEN a permit is `Unknown`
- AND a later trustworthy terminal observation matches the original attempt
- WHEN explicit settlement runs
- THEN Lattice MUST record a linked settlement decision
- AND it MUST retain the prior uncertainty record

#### Scenario: An operator abandons uncertainty

- GIVEN a permit is `Unknown`
- WHEN an authorized operator confirms abandonment
- THEN Lattice MUST record abandonment without restoring or reusing the occurrence

#### Scenario: An operator requests a fresh retry

- GIVEN a permit is `Unknown`
- AND an authorized operator acknowledges duplicate risk
- WHEN fresh retry admission runs
- THEN Lattice MUST require new permit and attempt identities

#### Scenario: Fresh retry lacks acknowledgment

- GIVEN a permit is `Unknown`
- WHEN fresh retry lacks duplicate-risk acknowledgment
- THEN Lattice MUST reject the retry

### Requirement: The core remains pure and authority-free

r[lattice.linear_process_permit.boundary] The permit core MUST perform no filesystem, process, storage, network, clock, credential, authorization-service, or presentation effect.

#### Scenario: A direct test calls the core

- GIVEN ordinary in-memory state and admitted facts
- WHEN a test evaluates a permit decision
- THEN no production infrastructure MUST be required

#### Scenario: The core starts a process or reads storage

- GIVEN the permit core directly accesses an external capability
- WHEN architecture validation runs
- THEN validation MUST reject the implementation

### Requirement: Legacy behavior remains unchanged

r[lattice.linear_process_permit.compatibility] Workflows without the explicit pilot profile MUST retain current behavior, persisted shapes, commands, and claim scope.

#### Scenario: A legacy shell workflow runs

- GIVEN the workflow has no process-permit profile
- WHEN Lattice admits and runs the workflow
- THEN its current supported behavior MUST remain unchanged

#### Scenario: A migration infers permits from node type

- GIVEN a legacy shell node has no declared permit artifacts
- WHEN migration runs
- THEN migration MUST NOT infer a permit or linear-resource claim

### Requirement: Evidence covers runtime order and failure

r[lattice.linear_process_permit.evidence] Evidence MUST cover core transitions, shell call order, persistence, process observations, crash windows, recovery, compatibility, and denial paths.

#### Scenario: The pilot evidence is complete

- GIVEN all required positive and negative tests pass
- WHEN readiness review runs
- THEN evidence MUST record exact source, producer, profile, state, adapter, command, and non-claim identities

#### Scenario: A denial can still start a process

- GIVEN any intake, authorization, persistence, state, or recovery decision rejects
- WHEN shell tests inspect port calls
- THEN they MUST observe no protected process effect

#### Scenario: Evidence claims exactly-once execution

- GIVEN evidence proves local single-use permit decisions and bounded observations
- WHEN a report claims external exactly-once execution
- THEN evidence review MUST reject the broader claim
