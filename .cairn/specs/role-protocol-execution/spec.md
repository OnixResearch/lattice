# Role Protocol Execution Specification

## Purpose

Defines the `role-protocol-execution` capability.

## Requirements

### Requirement: Lattice consumes exact Choregraph protocol artifacts
r[lattice.role_protocol.intake] Lattice MUST consume a frozen Choregraph cohort containing one global artifact, one ordered complete local artifact set, one projection receipt, exact schemas, adapter identity, and source revision. Intake MUST reject stale, mismatched, partial, duplicate, reordered, or unsupported artifacts.

#### Scenario: Complete current protocol cohort is admitted
r[lattice.role_protocol.intake.valid]
- GIVEN canonical artifacts and identities match the selected frozen Choregraph cohort
- WHEN pure intake runs
- THEN Lattice MUST preserve every artifact identity and role association in strict DTOs.

#### Scenario: One local artifact is missing
r[lattice.role_protocol.intake.partial]
- GIVEN the global artifact declares three roles but the local set contains two roles
- WHEN intake runs
- THEN admission MUST fail before session planning or persistence.

### Requirement: Every role has one exact placement
r[lattice.role_protocol.placement] A role-protocol session plan MUST bind every declared role to exactly one native or external role instance. Missing, duplicate, extra, mixed-kind, stale, implicit, or fallback placement MUST fail before session creation.

#### Scenario: Complete local placement is admitted
r[lattice.role_protocol.placement.valid]
- GIVEN every declared role has one current role instance with exact runtime or external-contract identities
- WHEN placement admission runs
- THEN Lattice MAY create one deterministic session plan.

#### Scenario: Role has fallback destinations
r[lattice.role_protocol.placement.fallback]
- GIVEN a role instance declares an ordered or implicit fallback destination
- WHEN placement admission runs
- THEN admission MUST fail without selecting a destination.

### Requirement: Protocol sessions have exact identities
r[lattice.role_protocol.session_identity] Lattice MUST derive domain-separated BLAKE3 identities for the session, each role instance, protocol step, message, attempt, observation, and outcome from canonical semantic fields.

#### Scenario: Session placement changes
r[lattice.role_protocol.session_identity.changed]
- GIVEN one role placement or local artifact identity changes
- WHEN session planning runs
- THEN the new session-plan identity MUST differ from the prior identity.

### Requirement: Session state is explicit and deterministic
r[lattice.role_protocol.state] A pure core MUST represent each role cursor, expected action, prepared attempt, observation, outcome, and terminal state as explicit bounded data. Equal canonical inputs MUST produce equal decisions and ordered diagnostics.

#### Scenario: Equal state receives equal observation
r[lattice.role_protocol.state.deterministic]
- GIVEN two equal canonical session states and equal observations
- WHEN the pure decision core runs
- THEN both decisions MUST be equal.

### Requirement: Protocol envelopes bind exact context
r[lattice.role_protocol.envelopes] Every admitted protocol envelope MUST bind protocol, complete local set, session, source role, destination role, source local artifact, destination local artifact, expected step, message, attempt, label, payload tag, optional required value identity, payload reference, schema, and adapter cohort identities.

#### Scenario: Exact expected envelope arrives
r[lattice.role_protocol.envelopes.valid]
- GIVEN an envelope matches every field expected by the receiver cursor
- WHEN envelope admission runs
- THEN the receiver MAY use it as input to the pure transition decision.

#### Scenario: Envelope belongs to another session
r[lattice.role_protocol.envelopes.cross_session]
- GIVEN every envelope field matches except the session identity
- WHEN envelope admission runs
- THEN the envelope MUST be rejected without cursor advancement.

### Requirement: Transfer and choice transitions follow local artifacts
r[lattice.role_protocol.transitions] Lattice MUST advance a role cursor only through the exact send-owned, receive-owned, internal-choice, offer, or terminal action exposed by its current local artifact. Peers, labels, payload tags, values, and continuations MUST match exactly.

#### Scenario: Expected receiver accepts a transfer
r[lattice.role_protocol.transitions.transfer]
- GIVEN the receiver cursor expects the exact admitted transfer envelope
- WHEN the receiver observation is admitted
- THEN the receiver transition MUST advance to the stored local continuation.

#### Scenario: Stale label reaches an offer
r[lattice.role_protocol.transitions.stale_label]
- GIVEN a role cursor exposes an offer whose label set excludes the supplied label
- WHEN choice admission runs
- THEN the choice MUST fail without cursor advancement.

### Requirement: Committed transfer excludes former-owner use
r[lattice.role_protocol.ownership] After an exact receiver observation commits one semantic owned transfer, the session state MUST record the destination as owner and MUST reject a later action that uses the former owner as source without an admitted return transfer.

#### Scenario: Former owner sends committed value
r[lattice.role_protocol.ownership.stale]
- GIVEN a committed transfer moved value `V` from `A` to `B`
- WHEN a later action names `A` as source of `V`
- THEN the action MUST be rejected before dispatch.

### Requirement: Persistence precedes dispatch
r[lattice.role_protocol.persistence] Lattice MUST persist the exact session, cursor, expected action, message, attempt, adapter route, and authorization-decision reference before starting transport or external-role dispatch.

#### Scenario: Persistence fails before dispatch
r[lattice.role_protocol.persistence.failed]
- GIVEN the prepared-attempt transaction cannot commit
- WHEN dispatch planning runs
- THEN no transport or external-role effect MUST start.

### Requirement: Runtime uncertainty remains explicit
r[lattice.role_protocol.uncertainty] A dispatch that lacks an exact admitted receiver observation or terminal failure MUST remain an explicit unknown outcome. Lattice MUST NOT advance semantic ownership, report success, or retry automatically from that state.

#### Scenario: Process crashes after dispatch starts
r[lattice.role_protocol.uncertainty.crash]
- GIVEN a prepared attempt started transport dispatch and no admitted receiver observation was persisted
- WHEN Lattice restarts
- THEN the attempt MUST remain unknown
- AND no automatic resend or cursor advancement MUST occur.

### Requirement: External roles retain product authority boundaries
r[lattice.role_protocol.external_roles] An external role dispatch MUST require the exact external contract, adapter cohort, destination policy, credential references, limits, and current product authorization. External role metadata MUST NOT grant authority or prove external correctness.

#### Scenario: External contract matches but authority is missing
r[lattice.role_protocol.external_roles.denied]
- GIVEN an external role and adapter contract match the protocol artifact
- AND current authorization is absent or denied
- WHEN dispatch admission runs
- THEN no external effect MUST start.

### Requirement: Protocol actions use current authorization
r[lattice.role_protocol.authorization] Before transport or external effects, Lattice MUST bind the exact protocol action to the accepted Choregraph effect-intent authority boundary and current adapter policy. Historical authorization MUST NOT become fresh authority.

#### Scenario: Earlier allowance is replayed
r[lattice.role_protocol.authorization.replayed]
- GIVEN an earlier attempt has a passing authorization record
- WHEN a new attempt is planned
- THEN the earlier allowance MUST NOT authorize the new dispatch.

### Requirement: Protocol replay has no effects
r[lattice.role_protocol.replay] Replay MUST recompute session, cursor, envelope, transition, and outcome decisions from persisted records without emitting dispatchable transport or external effects.

#### Scenario: Replay reaches a prior send action
r[lattice.role_protocol.replay.send]
- GIVEN persisted history contains a prior prepared or dispatched send
- WHEN replay runs
- THEN replay MUST report the historical state
- AND it MUST emit no fresh message or external effect.

### Requirement: Protocol diagnostics are safe
r[lattice.role_protocol.diagnostics] Operator output MUST use stable issue codes and bounded protocol, session, role, cursor, step, attempt, outcome, count, and identity fields. It MUST omit payload bytes, credentials, commands, endpoint secrets, source text, and host paths.

#### Scenario: Adapter error contains a secret endpoint
r[lattice.role_protocol.diagnostics.redacted]
- GIVEN a raw adapter error contains credential or endpoint-secret text
- WHEN safe inspection output is produced
- THEN that text MUST be omitted from persisted and displayed diagnostics.

### Requirement: Protocol evidence is narrow
r[lattice.role_protocol.evidence] Lattice MUST bind protocol artifacts, placement, session records, actions, attempts, authorization references, adapter cohorts, observations, outcomes, recovery actions, diagnostics, and non-claims into deterministic evidence.

#### Scenario: Exact session replay is reported
r[lattice.role_protocol.evidence.replay]
- GIVEN one canonical persisted session replays without mismatch
- WHEN evidence is emitted
- THEN it MAY report local record and decision consistency for that session
- AND it MUST NOT claim remote delivery or external correctness.

### Requirement: Runtime authority remains in Lattice
r[lattice.role_protocol.boundary] Choregraph projectability and Trellis proofs MUST NOT grant Lattice transport, host, credential, persistence, retry, or effect authority. Passing runtime checks MUST NOT claim exactly-once delivery, endpoint identity truth, global progress, deadlock freedom under faults, production readiness, or release eligibility.

#### Scenario: Projectability is used as delivery evidence
r[lattice.role_protocol.boundary.overclaim]
- GIVEN a protocol is projectable and all local artifacts match
- WHEN a report claims remote delivery without an admitted runtime observation
- THEN the claim MUST be rejected.

### Requirement: Protocol execution has positive and negative fixtures
r[lattice.role_protocol.fixtures] Maintained fixtures MUST pair valid native, external, transfer, choice, ownership, terminal, replay, recovery, and crash cases with stale, partial, mismatched, duplicated, reordered, unauthorized, leaking, and overclaim cases.

#### Scenario: Protocol fixture corpus runs
r[lattice.role_protocol.fixtures.coverage]
- GIVEN the reviewed role-protocol fixture corpus
- WHEN maintained checks run
- THEN valid cases MUST reach their exact expected states
- AND invalid cases MUST fail before forbidden advancement or effects.

### Requirement: Protocol execution has maintained validation
r[lattice.role_protocol.validation] Maintained checks MUST cover frozen adapter intake, pure decisions, persistence order, crash windows, replay, transport and external adapters, authorization, diagnostics, positive and negative fixtures, Cairn, Octet, and relevant Nix rails.

#### Scenario: Choregraph cohort changes
r[lattice.role_protocol.validation.stale]
- GIVEN the Choregraph protocol schema, adapter, source, or artifact identities differ from the frozen cohort
- WHEN maintained validation runs
- THEN protocol execution evidence MUST fail until the cohort is reviewed again.

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
