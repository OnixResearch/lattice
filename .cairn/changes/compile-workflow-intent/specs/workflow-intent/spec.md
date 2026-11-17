## ADDED Requirements

### Requirement: Finite inert intent planning
r[lattice.workflow_intent.planning] Lattice MUST normalize the supported host-supplied WorkflowIntent fragment into one bounded Choregraph-admitted dependency plan without executing a workload, opening credentials or granting authority.

#### Scenario: Verify, dependency, review and completion contracts
r[lattice.workflow_intent.planning.scenario.finite]
- GIVEN finite A-to-B dependencies, independent C, required review and completion-only D
- WHEN the supported complete fragment is compiled
- THEN the plan MUST preserve exact obligation, guidance, remaining-condition, action/precondition and required-observation references
- AND review and native completion MUST remain owner obligations rather than inferred successful execution

#### Scenario: Unsupported and incomplete input
r[lattice.workflow_intent.planning.scenario.denial]
- GIVEN missing check contracts, cycles, invalid identities, incomplete/restricted context, unavailable compatibility or exhausted bounds
- WHEN compilation is requested
- THEN it MUST return a bounded nonexecuting diagnostic rather than a success plan

### Requirement: Exact deterministic plan binding
r[lattice.workflow_intent.bindings] Lattice MUST bind canonical intent, selected implementation alternative, compiler/schema, producer/source/context/policy/dependency and runtime-profile facts independently of graph presentation.

#### Scenario: Ordering and meaningful drift
r[lattice.workflow_intent.bindings.scenario.drift]
- GIVEN semantically unordered intent facts
- WHEN only their ordering changes
- THEN the plan identity MUST remain unchanged
- BUT meaningful intent, policy, compiler-artifact, profile or selected implementation change MUST alter the applicable plan binding

### Requirement: Workflow proposals remain data
r[lattice.workflow_intent.proposals] Lattice MUST keep proposed investigation or workflow/evidence changes separate from frozen execution and require owner readmission for changed plans.

#### Scenario: No self-approval or evidence weakening
r[lattice.workflow_intent.proposals.scenario.readmission]
- GIVEN a worker proposes an extra step or changes a required evidence predicate
- WHEN the proposal is compared with a frozen plan
- THEN the comparison MUST expose the exact added, removed and changed obligations without changing the frozen plan or granting effects

### Requirement: Producer and execution non-claims
r[lattice.workflow_intent.boundary] Lattice MUST distinguish its host-normalized input and deterministic compile evidence from an adopted Cairn producer snapshot, current execution authority, native acceptance or live worker qualification.

#### Scenario: Absent native producer remains unqualified
r[lattice.workflow_intent.boundary.scenario.producer]
- GIVEN a host-normalized compile fixture but no adopted Cairn typed selected-work producer or live execution composition
- WHEN the compiler emits an inert dependency plan
- THEN it MUST NOT claim authenticated producer coverage, native obligation discharge, current execution authority or live worker qualification

