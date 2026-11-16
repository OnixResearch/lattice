## ADDED Requirements

### Requirement: Configured agent binding facts
r[lattice.agent_runtime.configured_bindings] Lattice MUST project existing configured runtime compatibility independently of owned worker instances, compute reservations and model-service admission, without creating authority or capacity claims.

#### Scenario: Truthful configured compatibility
r[lattice.agent_runtime.configured_bindings.scenario.compatibility]
- GIVEN existing Pi, Clankers or Animus runtime configuration
- WHEN material-free configured profile facts are requested
- THEN projection MUST reuse existing route selection and planning, reject invalid/ambiguous configuration, preserve direct-adapter limits and omit commands, prompts, credentials, endpoints and physical paths
- AND unqualified generic-command semantics MUST remain unknown rather than becoming qualified executable support

#### Scenario: Durable configuration provenance
r[lattice.agent_runtime.configured_bindings.scenario.usage]
- GIVEN a production adapter returns provider usage with the host-selected runtime configuration identity
- WHEN orchestration normalizes and persists that report
- THEN it MUST retain the exact existing admission configuration identity, reject malformed/crossed target provenance, preserve absent provenance as unknown, and reject changes to frozen content across duplicate writes and restart
- AND existing reservation, known partial usage, unknown remainder and owned-outcome refinement meanings MUST remain distinct

#### Scenario: Missing producer APIs remain unavailable
r[lattice.agent_runtime.configured_bindings.scenario.boundary]
- GIVEN the selected compute and serving producers do not yet implement the required versioned reservation/readiness/request-admission APIs
- WHEN Lattice publishes configured profile or usage facts
- THEN it MUST expose those producer capabilities as unsupported, MUST NOT invent grants or owned worker observations, and MUST retain exact inspected source identities and remaining qualification prerequisites
