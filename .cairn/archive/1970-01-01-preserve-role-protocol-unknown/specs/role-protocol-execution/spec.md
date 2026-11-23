# Role Protocol Uncertainty Correction

## MODIFIED Requirements

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
