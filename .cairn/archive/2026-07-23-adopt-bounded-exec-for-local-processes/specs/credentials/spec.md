## ADDED Requirements

### Requirement: Bounded command credential provider
r[lattice.credentials.bounded_command_provider] Lattice MUST execute command credential providers through the shared bounded process mechanism with opaque bounded stdin, a positive deadline, independent stdout and stderr limits, owned teardown, truncation rejection, and diagnostics that omit secret values.

#### Scenario: Command provider resolves inside bounds
r[lattice.credentials.bounded_command_provider.scenario.success]
- GIVEN a configured command credential provider accepts the opaque reference and returns a complete valid response within every process bound
- WHEN Lattice resolves that reference for an admitted effect
- THEN it MUST parse the bounded response and expose the secret only to the existing execution boundary

#### Scenario: Command provider failure remains secret safe
r[lattice.credentials.bounded_command_provider.scenario.failure]
- GIVEN a command credential provider times out, is cancelled, exceeds either stream bound, exits unsuccessfully, or returns malformed output
- WHEN Lattice handles the bounded observation
- THEN it MUST reject credential resolution before the dependent effect and MUST NOT include reference input, stdout, stderr, or resolved secret material in diagnostics or persisted evidence
