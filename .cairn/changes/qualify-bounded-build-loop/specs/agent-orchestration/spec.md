## ADDED Requirements

### Requirement: Bounded build-loop reference runtime
r[lattice.agent_orchestration.bounded_build_loop] The local planner/generator/evaluator example MUST distinguish finite graph construction from runtime repetition, execute a real Rust build/test evaluator through the production executor, retain the specification/source/verdict handoffs and exact revision/recipe/result references for every produced round, and use the existing durable orchestration transitions and one total budget. Generated text MUST NOT grant authority or mutate an in-flight graph. Optional masked repair MUST remain disabled without a qualified producer and adapter.

#### Scenario: Real evaluator pass stops repetition
r[lattice.agent_orchestration.bounded_build_loop.scenario.pass]
- GIVEN the fixed local producer emits the required greeting implementation
- WHEN real Rust compilation and unit tests pass
- THEN the durable orchestration MUST complete after one round without another dispatch

#### Scenario: Failed check is retained before runtime retry
r[lattice.agent_orchestration.bounded_build_loop.scenario.retry]
- GIVEN the first candidate compiles but fails its actual unit test
- WHEN an exact owned terminal failure enables the one authorized retry
- THEN the previous source/verdict MUST remain attributable to its original revision and the next round MUST consume the same total budget

#### Scenario: Iteration exhaustion is durable
r[lattice.agent_orchestration.bounded_build_loop.scenario.exhausted]
- GIVEN both allowed candidates fail their actual test
- WHEN retry preflight proves the iteration cap exhausted
- THEN the orchestration MUST persist Exhausted and MUST dispatch no third round

#### Scenario: Resource exhaustion does not reset accounting
r[lattice.agent_orchestration.bounded_build_loop.scenario.resource]
- GIVEN the total Agent budget permits only one round
- WHEN its actual test fails and retry preflight rejects the resource charge
- THEN the loop MUST persist Exhausted without resetting consumed usage or dispatching another round

#### Scenario: Candidate revision and runtime admission precede dispatch
r[lattice.agent_orchestration.bounded_build_loop.scenario.admission]
- GIVEN source is regenerated or an undeclared runtime is proposed during an active round
- WHEN the host considers another round
- THEN it MUST preserve the running snapshot and reject the unapproved revision before dispatch while retaining earlier source/verdict receipts under the original revision

#### Scenario: Role and child references cannot grant effects
r[lattice.agent_orchestration.bounded_build_loop.scenario.authority]
- GIVEN a role has no matching current effect grant, or a saved child requests ungranted effects, or generated output requests an additional effect or self-approval
- WHEN admission or dispatch is attempted
- THEN the missing grant MUST prevent that adapter's effects and generated output MUST NOT widen authority

#### Scenario: Active owned cancellation prevents later rounds
r[lattice.agent_orchestration.bounded_build_loop.scenario.cancel]
- GIVEN the real evaluator process has started
- WHEN the host requests cancellation
- THEN it MUST forward the request to the owned executor, retain the observed termination separately from intent, settle the admitted reservation and dispatch no later round

#### Scenario: Actual completion wins late cancellation
r[lattice.agent_orchestration.bounded_build_loop.scenario.cancel-winner]
- GIVEN a real evaluator has already completed successfully or unsuccessfully without observed cancellation
- WHEN an operator cancellation request is durably recorded before completion settlement
- THEN `RunFinishedObserved` MUST preserve the actual completion, consume the reservation exactly once, prevent retry after failed completion plus cancellation intent, and replay the same state without rerunning effects
