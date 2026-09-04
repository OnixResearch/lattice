# Role Protocol Execution Specification Delta

## ADDED Requirements

### Requirement: Lattice publishes an immutable role-protocol runtime profile [r[lattice.role_protocol.runtime_profile]]

Lattice MUST publish one closed, canonical runtime and service-integration profile for each supported projected-role execution surface. The profile MUST bind its runtime revision, selected source manifest, package, binary, command prefix, supported commands, state backend, service mode, startup behavior, effect-dispatch capability, authority boundary, consumer-required inputs, and explicit non-claims.

#### Scenario: Existing operator runtime is published [r[lattice.role_protocol.runtime_profile.scenario.operator]]

- GIVEN the selected runtime revision exposes the reviewed projected-role CLI
- WHEN the canonical profile is evaluated
- THEN it MUST identify the operator-invoked one-shot service surface
- AND it MUST list only create, inspect, recover, retry, abandon, and replay
- AND it MUST state that automatic startup and CLI effect dispatch are unavailable.

#### Scenario: Consumer requests autonomous dispatch [r[lattice.role_protocol.runtime_profile.scenario.unsupported]]

- GIVEN a consumer requires a boot-started daemon or CLI effect dispatch
- WHEN it compares the request with the published profile
- THEN compatibility MUST fail before service assembly
- AND the consumer MUST NOT infer the missing behavior from internal Rust APIs.

#### Scenario: Profile semantics change [r[lattice.role_protocol.runtime_profile.scenario.identity]]

- GIVEN any runtime, service, capability, state, authority, input, or non-claim field changes
- WHEN canonical export runs
- THEN the profile BLAKE3 identity MUST change
- AND the old profile MUST remain pinned to its original runtime revision and source manifest.

#### Scenario: Profile contains unsupported authority [r[lattice.role_protocol.runtime_profile.scenario.authority]]

- GIVEN a profile enables startup, embeds a credential or endpoint, claims CLI dispatch, or grants deployment authority
- WHEN contract admission runs
- THEN the profile MUST fail before publication.
