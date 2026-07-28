# Choregraph Effect Authority

## ADDED Requirements

### Requirement: Existing boundary baseline
r[lattice.choregraph_effect_authority.baseline] Lattice MUST record current effect adapters, capability roots, durable attempt ordering, replay behavior, Basalt source, supplied UCAN facts, and crash fixtures before integration.

#### Scenario: Attempt ordering is unknown
r[lattice.choregraph_effect_authority.baseline.scenario.unknown]
- GIVEN a proposed effect integration without a durable baseline
- WHEN readiness is checked
- THEN implementation MUST remain blocked.

### Requirement: Exact intent binding
r[lattice.choregraph_effect_authority.binding] Every authorization request MUST bind machine, workflow revision, run, node, transition, operation, resource, ability, request, payload, policy, and authority identities.

#### Scenario: Run identity differs
r[lattice.choregraph_effect_authority.binding.scenario.run-mismatch]
- GIVEN an authority receipt for another run
- WHEN binding validation runs
- THEN the intent MUST be denied.

### Requirement: Policy is separate from graph data
r[lattice.choregraph_effect_authority.policy] Basalt authorization policy MUST be reviewed separately from graph-authored capability requirements and MUST have an exact BLAKE3 identity.

#### Scenario: Graph grants its own ability
r[lattice.choregraph_effect_authority.policy.scenario.self-grant]
- GIVEN graph data that attempts to modify authorization policy
- WHEN workflow admission runs
- THEN the graph MUST NOT gain authority.

### Requirement: Basalt enforcement boundary
r[lattice.choregraph_effect_authority.basalt] Lattice MUST evaluate the exact contract, resource, ability, request, grant, receipt, caveat, revocation, replay, and non-claim bindings before product effects.

#### Scenario: Ability is not granted
r[lattice.choregraph_effect_authority.basalt.scenario.ability]
- GIVEN a valid intent without a matching admitted ability
- WHEN Basalt enforcement runs
- THEN the product effect MUST NOT execute.

### Requirement: Durable pre-effect attempt
r[lattice.choregraph_effect_authority.durability] Lattice MUST reserve and persist the bound intent and local attempt identity before invoking a product effect.

#### Scenario: Persistence fails
r[lattice.choregraph_effect_authority.durability.scenario.persistence]
- GIVEN an allowed authorization decision
- WHEN attempt persistence fails before dispatch
- THEN no product effect MUST run.

### Requirement: Denial has no product effect
r[lattice.choregraph_effect_authority.denial] A denied or unavailable authorization decision MUST produce a deterministic outcome event and MUST NOT invoke a product runtime.

#### Scenario: Policy source is unavailable
r[lattice.choregraph_effect_authority.denial.scenario.unavailable]
- GIVEN no current admitted policy decision
- WHEN dispatch is requested
- THEN dispatch MUST fail closed without a product effect.

### Requirement: Capability-relative file effects
r[lattice.choregraph_effect_authority.files] Local file effects MUST map configured root identifiers to `cap_std::fs::Dir`, validate relative paths through `cap-root`, and apply Lattice no-follow and file-operation policy.

#### Scenario: Path escapes its root
r[lattice.choregraph_effect_authority.files.scenario.escape]
- GIVEN an absolute, parent-traversing, or escaping relative path
- WHEN file admission runs
- THEN access MUST fail before file I/O.

### Requirement: Root authority is operator supplied
r[lattice.choregraph_effect_authority.root_authority] Graph data MUST NOT open ambient roots or call `ambient_authority()` and MAY only reference configured root identifiers.

#### Scenario: Unknown root identifier is supplied
r[lattice.choregraph_effect_authority.root_authority.scenario.unknown]
- GIVEN an intent naming a root outside runtime configuration
- WHEN root resolution runs
- THEN the file effect MUST be denied.

### Requirement: Non-file adapters remain bounded
r[lattice.choregraph_effect_authority.adapters] Process and network intents MUST use their selected bounded runtime adapters and MUST NOT treat `cap-std` as process or network authorization.

#### Scenario: Process intent is allowed
r[lattice.choregraph_effect_authority.adapters.scenario.process]
- GIVEN an allowed process intent
- WHEN runtime dispatch runs
- THEN the existing bounded process limits MUST still apply.

### Requirement: Replay cannot dispatch
r[lattice.choregraph_effect_authority.replay] Replayed intents and prior authorization receipts MUST NOT become fresh dispatch authority.

#### Scenario: Allowed receipt appears in history
r[lattice.choregraph_effect_authority.replay.scenario.history]
- GIVEN a prior allowed receipt during replay
- WHEN state reconstruction runs
- THEN no product effect MUST execute.

### Requirement: Authenticated Basalt source
r[lattice.choregraph_effect_authority.source] Release adoption MUST use an exact admitted Basalt and UCAN source that supports the selected build environment without hidden fallback.

#### Scenario: UCAN source requires unavailable SSH authority
r[lattice.choregraph_effect_authority.source.scenario.ssh]
- GIVEN a release environment without the required source credentials
- WHEN dependency admission runs
- THEN release readiness MUST report a blocker rather than skip checks.

### Requirement: Positive and negative authority fixtures
r[lattice.choregraph_effect_authority.fixtures] Tests MUST include exact allowed cases and missing, mismatched, stale, revoked, replayed, root, path, adapter, duplicate, and crash failures.

#### Scenario: Authority corpus runs
r[lattice.choregraph_effect_authority.fixtures.scenario.corpus]
- GIVEN the reviewed positive and negative fixture corpus
- WHEN maintained checks run
- THEN allowed cases MUST dispatch through bounded adapters and denied cases MUST produce no product effect.

### Requirement: Maintained authority validation
r[lattice.choregraph_effect_authority.validation] Maintained checks MUST run policy, binding, capability, adapter, replay, crash, Octet, Cairn, and Nix evidence.

#### Scenario: Crash fixtures are skipped
r[lattice.choregraph_effect_authority.validation.scenario.crash]
- GIVEN passing authorization tests without current crash-boundary evidence
- WHEN readiness runs
- THEN release readiness MUST fail.

### Requirement: Authority claim boundary
r[lattice.choregraph_effect_authority.non_claims] Passing checks MUST NOT claim key truth, revocation freshness, replay-store durability, UCAN cryptographic correctness, operating-system correctness, external effect success, exactly-once behavior, or whole-workflow correctness.

#### Scenario: Authorized effect completes
r[lattice.choregraph_effect_authority.non_claims.scenario.scope]
- GIVEN one locally observed allowed effect result
- WHEN evidence is reported
- THEN reporting MUST remain scoped to the exact recorded local facts.
