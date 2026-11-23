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
r[lattice.role_protocol.uncertainty] A possibly dispatched action without exact admitted disposition MUST remain DispatchedUnknown. Deterministic adapter/profile selection MUST precede effects. Generic HTTP status, failure-shaped response, summary or missing observation MUST NOT establish rejection, cursor progress, ownership transfer or external-effect completion. Mixed accepted/rejected atomic fanout MUST remain blocked against blind retry. Recovery MUST preserve explicit duplicate-risk acknowledgment.

#### Scenario: Partial fanout cannot erase prior acceptance
r[lattice.role_protocol.uncertainty.fanout]
- GIVEN a valid projected three-role choice plans two recipient envelopes
- WHEN a later adapter is missing/mismatched or a later recipient rejects or remains unknown
- THEN preflight MUST prevent all effects for deterministic selection failure
- AND prior possible delivery MUST otherwise remain blocked until exact per-message reconciliation or acknowledged recovery
- AND one recipient's success MUST NOT resolve another recipient's unknown outcome.
- AND a later no-start rejection MUST be independently persisted with its exact envelope and zero delivery/effect counts before it is admitted through the current observer contract.

#### Scenario: Generic HTTP response is not an observation
r[lattice.role_protocol.uncertainty.http]
- GIVEN the receiver independently records not-applied, applied-once or in-progress work
- WHEN transport returns 202, 500, 502 or loses acknowledgment
- THEN no response alone MUST commit or definitively fail the action
- AND current exact receiver observations MUST remain reconcilable.

#### Scenario: Genuine first rejection starts no receiver work
r[lattice.role_protocol.uncertainty.rejection]
- GIVEN an adapter guarantees rejection before any recipient work starts
- WHEN no earlier message started
- THEN the attempt MAY fail terminally
- AND the receiver ledger MUST contain no effect or delivery.

#### Scenario: Agent history cannot rewrite authoritative reality
r[lattice.role_protocol.uncertainty.history]
- GIVEN an independently persisted operation/effect ledger and authoritative attempt records
- WHEN unchanged history, additive truthful correction, false done handoff or stale failed handoff is presented directly, after process restart or after handoff reconstruction
- THEN summary-only mutations MUST preserve authority, disposition and idempotency identities
- AND replay/reconstruction MUST start no adapter work
- AND missing/unavailable observation MUST preserve uncertainty.
- AND the deterministic handoff reader MUST consume each selected representation and derive its proposed completion/retry intent, while safe reconstruction derives claims from authoritative replay and the receiver ledger instead.

#### Scenario: Current exact observer contract rejects substitution
r[lattice.role_protocol.uncertainty.observations]
- GIVEN a current receiver contract authenticates exact ledger observations before Runtime::observe
- WHEN wrong-session, wrong-attempt, wrong-message, wrong-role, wrong-adapter, stale-generation, duplicated or substituted records are supplied
- THEN observation MUST be rejected without authoritative cursor or ownership advancement.

#### Scenario: Independent checker detects broken completion controls
r[lattice.role_protocol.uncertainty.control]
- GIVEN a control trusts a summary or generic response instead of an admitted current observation
- WHEN the control claims absent completion or duplicates effects or introduces compensation
- THEN the independent operation/effect ledger checker MUST reject the control even if an internal trace claims success.
- AND duplicate-effect checking MUST aggregate a stable logical-operation key across fresh attempt/message identities, while keeping distinct fanout recipients separate
- AND the broken summary controls MUST consume the supplied false claims and exercise both false completion and actual independently persisted duplicate effects, including a fresh canonical retry attempt.

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

### Requirement: Lattice publishes an immutable role-protocol runtime profile

r[lattice.role_protocol.runtime_profile] Lattice MUST publish one closed, canonical runtime and service-integration profile for each supported projected-role execution surface. The profile MUST bind its runtime revision, selected source manifest, package, binary, command prefix, supported commands, state backend, service mode, startup behavior, effect-dispatch capability, authority boundary, consumer-required inputs, and explicit non-claims.

#### Scenario: Existing operator runtime is published

r[lattice.role_protocol.runtime_profile.scenario.operator]
- GIVEN the selected runtime revision exposes the reviewed projected-role CLI
- WHEN the canonical profile is evaluated
- THEN it MUST identify the operator-invoked one-shot service surface
- AND it MUST list only create, inspect, recover, retry, abandon, and replay
- AND it MUST state that automatic startup and CLI effect dispatch are unavailable.

#### Scenario: Consumer requests autonomous dispatch

r[lattice.role_protocol.runtime_profile.scenario.unsupported]
- GIVEN a consumer requires a boot-started daemon or CLI effect dispatch
- WHEN it compares the request with the published profile
- THEN compatibility MUST fail before service assembly
- AND the consumer MUST NOT infer the missing behavior from internal Rust APIs.

#### Scenario: Profile semantics change

r[lattice.role_protocol.runtime_profile.scenario.identity]
- GIVEN any runtime, service, capability, state, authority, input, or non-claim field changes
- WHEN canonical export runs
- THEN the profile BLAKE3 identity MUST change
- AND the old profile MUST remain pinned to its original runtime revision and source manifest.

#### Scenario: Profile contains unsupported authority

r[lattice.role_protocol.runtime_profile.scenario.authority]
- GIVEN a profile enables startup, embeds a credential or endpoint, claims CLI dispatch, or grants deployment authority
- WHEN contract admission runs
- THEN the profile MUST fail before publication.

### Requirement: Declared preference factors are default-off metadata

r[lattice.role_protocol.preference_factors] Lattice MUST admit declared preference factors, and an optional exogenous world variable, only from an admitted Choregraph `choregraph.role-choreography.profile.preference-annotations.v1` cohort. It MUST record them only under an explicitly selected `record-declared-factors` session profile. The default `exact-label-baseline` profile MUST ignore them, and every report MUST distinguish `factors-recorded`, `factors-ignored-profile-off` and `game_mapping = not-formalized`.

#### Scenario: Selected profile records the decider's factors

r[lattice.role_protocol.preference_factors.recorded]
- GIVEN an annotated projected cohort with factors and a world variable on a decider `InternalChoice`
- WHEN it is admitted under `record-declared-factors`
- THEN the cohort MUST record, per role, each decision's paths, exact labels, factors and world
- AND the report MUST state `factors-recorded` and `not-formalized`
- AND session inspection MUST expose the recorded factors after persistence.

#### Scenario: Annotated input without opt-in records nothing

r[lattice.role_protocol.preference_factors.default_off]
- GIVEN the same annotated cohort
- WHEN it is admitted under the default profile, or offered to the producer baseline selection or the standalone CLI
- THEN no factor data MUST enter the session
- AND the cohort, session and inspection bytes MUST equal those of the unannotated `admit_projection` path
- AND the report MUST state `factors-ignored-profile-off`.

#### Scenario: Annotated producer cohort is the exact pinned revision

r[lattice.role_protocol.preference_factors.producer]
- GIVEN Choregraph `fcbae0aa17bc0eab1345529adb9ff104a46540ef` publishes the annotation profile on its canonical Radicle seed
- WHEN source admission and the frozen role-protocol intake run
- THEN every Choregraph Cargo dependency, the Cargo lock source, the flake input and lock, the admission receipt and `FROZEN_CHOREGRAPH_REVISION` MUST name that revision
- AND the admission receipt MUST bind its source archive BLAKE3 and the lock its narHash
- AND the producer's v1 projection receipts MUST remain the ones the unchanged v1 intake admits.

### Requirement: Recorded factors are identity-bound to the cohort

r[lattice.role_protocol.preference_factors.identity] A recorded factor set MUST be verified with the producer's `verify_receipt` and `verify_local_set` against the annotated bundle. It MUST be linked to the admitted v1 receipt, global artifact, ordered local artifacts and complete local set, and it MUST be bound into the session-plan identity. A crossed, stale, reordered, incomplete or tampered receipt or annotated local set MUST be rejected.

#### Scenario: Receipt or local set comes from another cohort

r[lattice.role_protocol.preference_factors.identity.crossed]
- GIVEN two annotated cohorts over the same protocol with different factors
- WHEN one cohort's bundle is paired with the other's receipt body, receipt bytes or annotated local set
- THEN intake MUST fail with `protocol.identity-mismatch` under both profiles.

### Requirement: Factors cannot influence labels, ownership or authority

r[lattice.role_protocol.preference_factors.boundary] Recorded factors MUST NOT select or narrow a label, weigh branches, convert a label set into a Boolean, change cursor or ownership decisions, enter envelopes or authority requests, or grant authority. Exact-label admission MUST stay unchanged.

#### Scenario: Baseline and recorded sessions run the same choice

r[lattice.role_protocol.preference_factors.baseline_comparison]
- GIVEN baseline and recorded sessions over one annotated choice fixture
- WHEN the same undeclared, factor-identifier, wrong-role and declared labels are planned, and the same choice, owned transfer and denied-authority dispatch run
- THEN both sessions MUST return the same rejections and plannable labels
- AND their committed cursors, owners, attempt states and envelope semantics MUST be byte-identical
- AND no envelope or authority request MUST contain a factor or world name.

#### Scenario: Forged record tries to steer a choice

r[lattice.role_protocol.preference_factors.boundary.forged]
- GIVEN a record that narrows or reorders labels, moves a decision to another role, reassigns a factor, misaddresses a choice, drops its factors or a non-claim, names another profile or receipt, or adds a selected-label, weight, utility or authority field
- WHEN the session is planned or the cohort is decoded
- THEN planning MUST fail with a stable issue code or decoding MUST fail before a session exists.
