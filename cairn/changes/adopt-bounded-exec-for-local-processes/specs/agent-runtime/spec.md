## ADDED Requirements

### Requirement: Bounded command-agent process
r[lattice.agent_runtime.bounded_command_process] Lattice MUST execute command-backed agent adapters through the shared bounded process mechanism with an explicit positive deadline, bounded stdin, independent stdout and stderr limits, owned teardown, truncation rejection, and no fallback route.

#### Scenario: Command agent returns bounded output
r[lattice.agent_runtime.bounded_command_process.scenario.success]
- GIVEN a configured command-agent route exits before its deadline with complete output inside both stream bounds
- WHEN the production agent adapter invokes it
- THEN Lattice MUST decode the complete output under the requested typed mode and preserve bounded safe metadata

#### Scenario: Command agent timeout or truncation fails closed
r[lattice.agent_runtime.bounded_command_process.scenario.failure]
- GIVEN a command-agent route times out, is cancelled, or exceeds either stream bound
- WHEN bounded execution classifies the observation
- THEN Lattice MUST fail that node before output or handoff parsing and MUST NOT retry through another command, provider, target, or tool
