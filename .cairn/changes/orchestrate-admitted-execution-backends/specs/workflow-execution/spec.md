# Admitted Execution Backend Orchestration Delta

## Requirements

### Requirement: Workflow operations declare backend-neutral requirements

r[lattice.execution_backend_orchestration.contract] Lattice workflow operations MUST bind typed execution requirements for format, filesystem semantics, network posture, process model, terminal, browser or device needs, resources, persistence, and requested isolation claims. Unknown requirements MUST fail validation.

#### Scenario: Complete requirements are accepted

- GIVEN a workflow operation declares every behavior-relevant execution requirement
- WHEN workflow validation runs
- THEN Lattice records one canonical requirement profile identity

#### Scenario: Unknown requirement fails closed

- GIVEN a workflow operation uses an unsupported requirement
- WHEN workflow validation runs
- THEN Lattice denies the workflow before runtime selection

### Requirement: Animus planner mappings are frozen and revalidated

r[lattice.execution_backend_orchestration.planner_mapping] Lattice MUST consume one exact reviewed Animus planner cohort through compatibility fixtures. A planner result MUST be treated as advice and MUST NOT grant Lattice runtime, effect, credential, or authority admission.

#### Scenario: Compatible plan maps to Lattice

- GIVEN an Animus plan uses the reviewed cohort and every identity maps without loss
- WHEN Lattice validates the plan
- THEN it can continue to product-owned admission

#### Scenario: Mapping drift is rejected

- GIVEN the plan cohort, requirement class, backend class, or identity meaning differs from the reviewed mapping
- WHEN Lattice validates the plan
- THEN it denies before dispatch

### Requirement: Lattice retains final runtime admission

r[lattice.execution_backend_orchestration.admission] Lattice MUST revalidate the selected route against current workflow revision, operation contract, runtime adapter, effect policy, authority, credential references, budgets, persistence, and recovery policy.

#### Scenario: Current admission passes

- GIVEN a compatible plan and all current product checks pass
- WHEN Lattice admits the route
- THEN it produces one Lattice-owned backend attempt plan

#### Scenario: Planner allow cannot override denial

- GIVEN the planner selected a backend but Lattice authority, credentials, budget, or adapter readiness denies it
- WHEN product admission runs
- THEN the final result remains denied

### Requirement: Exact backend identity is persisted before effects

r[lattice.execution_backend_orchestration.persistence] Before dispatch, Lattice MUST persist the workflow revision, node, operation contract, requirement profile, planner cohort, backend class, adapter, backend profile, authority policy, resource bounds, run, and attempt identities.

#### Scenario: Crash before dispatch is distinguishable

- GIVEN the route is persisted but no dispatch observation exists
- WHEN recovery runs
- THEN Lattice can distinguish reserved work from an observed effect attempt

### Requirement: Dispatch and recovery deny silent substitution

r[lattice.execution_backend_orchestration.dispatch] Initial dispatch, retry, resume, and recovery MUST bind the persisted backend route. Backend drift or unavailability MUST deny the attempt. A different backend requires a new explicit plan and new attempt after uncertainty policy admits it.

#### Scenario: Backend changes during retry

- GIVEN a failed or uncertain attempt names one backend identity
- WHEN retry proposes another backend without a new plan
- THEN Lattice denies the retry

#### Scenario: Unknown outcome remains uncertain

- GIVEN dispatch can have reached an external backend but no terminal result exists
- WHEN recovery runs
- THEN Lattice preserves an unknown outcome and does not automatically redispatch elsewhere

### Requirement: Backend inspection is safe and read-only

r[lattice.execution_backend_orchestration.inspection] Lattice CLI and TUI views MUST show safe requirement classes, selected backend class, identities, attempt state, current observations, and denial reasons. They MUST omit commands, prompts, credentials, environment values, provider payloads, and physical paths.

#### Scenario: Operator inspects a route

- GIVEN a persisted backend attempt exists
- WHEN an operator requests its safe view
- THEN Lattice shows planned, admitted, dispatched, and observed distinctions without mutating the run

### Requirement: Backend orchestration has positive and negative evidence

r[lattice.execution_backend_orchestration.verification] The focused rail MUST cover supported backend classes and negative mapping, adapter, authority, credential, budget, drift, fallback, retry, recovery, and unknown-outcome cases.

#### Scenario: Focused rail passes

- GIVEN all required fixtures and crash-boundary cases are present
- WHEN the focused rail runs
- THEN it reports the supported orchestration boundary and non-claims
