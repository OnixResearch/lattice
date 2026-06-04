## ADDED Requirements

### Requirement: Generic runner admission diagnostics
r[n8nclone.workflow_validation.generic_runner_admission] The system MUST validate generic runner admission before executing side-effecting runtime calls.

#### Scenario: Runner admission blocks invalid graph
r[n8nclone.workflow_validation.generic_runner_admission.graph]
- GIVEN a workflow has invalid edge endpoints, a cycle, incompatible port types, or dependencies that cannot become ready
- WHEN generic runner admission runs
- THEN the system MUST report fatal diagnostics before invoking any runtime

#### Scenario: Missing runtime binding
r[n8nclone.workflow_validation.generic_runner_admission.runtime]
- GIVEN a workflow node references a missing runtime kind or runtime key
- WHEN generic runner admission runs
- THEN the system MUST report the missing runtime binding with the affected node identity before invoking any runtime

#### Scenario: Effect and resource policy rejection
r[n8nclone.workflow_validation.generic_runner_admission.effects]
- GIVEN a workflow node declares runtime effects, abilities, resource bounds, or sandbox requirements that are not provided by the selected runtime policy
- WHEN generic runner admission runs
- THEN the system SHOULD reject execution with deterministic diagnostics aligned with Trellis ability/effect/resource admission semantics where build and license fit

#### Scenario: Terminal state mutation rejection
r[n8nclone.workflow_validation.generic_runner_admission.terminal_state]
- GIVEN a persisted or resumed run record marks a step completed, failed, or cancelled
- WHEN generic runner admission evaluates a transition that would mutate that terminal step state
- THEN the system MUST reject the transition as fatal for the run and MUST NOT mutate persisted workflow configuration
