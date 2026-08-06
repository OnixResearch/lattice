# Agent Orchestration Specification Delta

## ADDED Requirements

### Requirement: Orchestration decision identities are nominal

r[lattice.agent_orchestration.nominal_identities.types] Lattice MUST represent admitted instance, dispatch, proposal, reservation, fact, trigger-idempotency, revision, policy, event, and ledger-head values with distinct private Rust types.

#### Scenario: Same-role orchestration identity is accepted

r[lattice.agent_orchestration.nominal_identities.types.same_role]
- GIVEN a valid typed instance identity
- WHEN a pure core API requires an instance identity
- THEN the API MUST accept it without another expected-role string.

#### Scenario: Cross-role orchestration identity does not compile

r[lattice.agent_orchestration.nominal_identities.compile_time]
- GIVEN instance and dispatch identities use distinct types
- WHEN source passes a dispatch identity to an API that requires an instance identity
- THEN the source MUST fail compilation with a type mismatch.

#### Scenario: Malformed identity fails before core use

r[lattice.agent_orchestration.nominal_identities.validation]
- GIVEN an untrusted identity is empty, oversized, malformed, stale, or uses unsupported digest spelling
- WHEN boundary admission runs
- THEN admission MUST fail with deterministic diagnostics
- AND the invalid value MUST NOT enter orchestration decisions.

### Requirement: Identity migration follows a semantic inventory

r[lattice.agent_orchestration.nominal_identities.inventory] Lattice MUST classify selected raw orchestration strings as stable identity, revision, digest, idempotency key, wire compatibility, display text, or deferred before migration.

#### Scenario: Identity inventory excludes display text

r[lattice.agent_orchestration.nominal_identities.inventory.display]
- GIVEN orchestration models contain objectives, summaries, labels, diagnostics, and redacted output text
- WHEN the identity inventory runs
- THEN those display values MUST remain outside nominal enforcement unless a separate semantic role is approved.

### Requirement: Pure transitions require exact identity roles

r[lattice.agent_orchestration.nominal_identities.transitions] Pure orchestration transitions MUST use exact typed instance, proposal, approval, fact, trigger, definition-revision, workflow-revision, and authority-policy inputs.

#### Scenario: Stale or wrong-role proposal is rejected

r[lattice.agent_orchestration.nominal_identities.transitions.stale]
- GIVEN an approval carries a valid proposal identity for another instance, epoch, base revision, or authority policy
- WHEN proposal admission runs
- THEN Lattice MUST reject the transition without creating a successor revision or effects.

### Requirement: Recovery requires exact typed ownership

r[lattice.agent_orchestration.nominal_identities.recovery] Dispatch, reservation, run ownership, cancellation, abandonment, and retry correlation MUST preserve exact typed instance and dispatch identities.

#### Scenario: Recovery accepts only exact dispatch ownership

r[lattice.agent_orchestration.nominal_identities.recovery.owner]
- GIVEN a persisted run record has a valid `RunId` but its typed instance or dispatch owner differs from the pending dispatch
- WHEN recovery runs
- THEN Lattice MUST reject correlation without settling or overwriting either record.

### Requirement: Event identities remain distinct from ledger heads

r[lattice.agent_orchestration.nominal_identities.events] Lattice MUST use distinct typed values for event identity, previous-event identity, and current ledger-head identity in append and replay logic.

#### Scenario: Event and ledger-head roles remain distinct

r[lattice.agent_orchestration.nominal_identities.events.distinct]
- GIVEN event and ledger-head values contain equal BLAKE3 bytes under different roles
- WHEN append or replay evaluates them
- THEN Lattice MUST preserve their roles and MUST NOT silently exchange them.

### Requirement: Boundary and persistence formats remain compatible

r[lattice.agent_orchestration.nominal_identities.wire_boundary] Lattice MUST convert Nickel, CLI, scheduler, webhook, JSON, and redb records through explicit adapters while preserving supported external formats.

#### Scenario: Existing persisted orchestration replays

r[lattice.agent_orchestration.nominal_identities.compatibility]
- GIVEN a supported persisted orchestration and event ledger from before migration
- WHEN the new adapters load and replay it
- THEN Lattice MUST reconstruct the same valid state and external report shape without rerunning effects.

### Requirement: Negative identity cases remain explicit

r[lattice.agent_orchestration.nominal_identities.failure_matrix] Lattice MUST test wrong owners, stale revisions, stale approvals, foreign run records, duplicate triggers, conflicting facts, malformed hashes, and event-chain tampering after migration.

#### Scenario: Wrong identity cannot create side effects

r[lattice.agent_orchestration.nominal_identities.failure_matrix.no_effect]
- GIVEN any checked identity mismatch occurs before dispatch
- WHEN admission or recovery returns the failure
- THEN no workflow, provider, credential, webhook, scheduler, or persistence effect beyond the failure record MUST start.

### Requirement: Octet guards orchestration identity scopes

r[lattice.agent_orchestration.nominal_identities.octet] After the policy becomes available, Lattice MUST declare migrated orchestration domains to the reviewed Octet nominal-domain policy.

#### Scenario: Raw domain regression is rejected

r[lattice.agent_orchestration.nominal_identities.octet.guard]
- GIVEN a migrated orchestration core field or signature reintroduces a declared identity as `String`
- WHEN Octet checks the scope
- THEN the selected nominal-domain check MUST fail.

### Requirement: Typed local identities preserve external-effect non-claims

r[lattice.agent_orchestration.nominal_identities.docs] Lattice documentation MUST state that typed orchestration values prove local role separation and checked syntax only.

#### Scenario: Boundary remains visible

r[lattice.agent_orchestration.nominal_identities.final_checks]
- GIVEN typed identities pass admission, persistence, recovery, and replay checks
- WHEN Lattice states the supported claim
- THEN it MUST NOT claim provider truth, remote cancellation, external exactly-once effects, independently witnessed history, or production readiness.
