## ADDED Requirements

### Requirement: Exact Durable Authority State source

r[basalt.durable_authority.source] Basalt MUST acquire both separate authority-state cores from canonical Radicle RID `rad:z3kGDT73ja8rewvGLwJFLF5MTx1ih` at exact object `a0de79303f3f45c18a34fff535b690a85cb3cf59` without fallback.

#### Scenario: Exact source resolves

- GIVEN Cargo and Nix use the admitted HTTPS adapter
- WHEN the two core packages resolve
- THEN both MUST select the same full reviewed object

#### Scenario: A fallback is supplied

- GIVEN a GitHub URL, sibling path, branch, tag, abbreviation, wrong RID, or path override
- WHEN dependency admission runs
- THEN Basalt MUST reject the source

### Requirement: Generic durable-authority composition

r[basalt.durable_authority.composition] Basalt MUST compose UCAN's published durable adapters over caller-owned replay and revocation ports without reimplementing shared transitions.

#### Scenario: Caller supplies both ports

- GIVEN admitted replay and revocation contexts plus caller-owned ports
- WHEN the Basalt shell constructs UCAN boundaries
- THEN the boundaries MUST use the shared adapters and retain the caller's infrastructure ownership

#### Scenario: Product policy enters the component

- GIVEN Basalt action, policy, credential, or outage rules are proposed for shared state cores
- WHEN the boundary is reviewed
- THEN the integration MUST reject that authority transfer

### Requirement: Final replay ordering remains intact

r[basalt.durable_authority.replay_order] Durable replay reservation MUST remain the only state-consuming operation and MUST occur after Basalt and UCAN preflight checks.

#### Scenario: First and duplicate use run

- GIVEN a valid compact holder invocation and an empty replay slot
- WHEN the request runs twice
- THEN the first run MUST reserve once and the duplicate MUST deny without another write

#### Scenario: Policy or revocation denies

- GIVEN an earlier Basalt, UCAN, or revocation check denies
- WHEN enforcement returns
- THEN the replay port MUST receive no write

### Requirement: Current revocation evidence remains caller-owned

r[basalt.durable_authority.revocation] Basalt MUST accept revocation lookup results only from the caller-selected source, generation, explicit time, completeness, and authenticity observations.

#### Scenario: Current complete view is supplied

- GIVEN a current complete authenticated snapshot
- WHEN UCAN checks a proof reference
- THEN the shared adapter MUST return the canonical revoked or not-revoked result

#### Scenario: View evidence is insufficient

- GIVEN an incomplete, unauthenticated, stale, missing, unavailable, or failed view
- WHEN UCAN checks a proof reference
- THEN enforcement MUST fail closed before replay

### Requirement: Product and infrastructure authority remain local

r[basalt.durable_authority.authority] Basalt MUST retain application policy and action mapping, UCAN MUST retain protocol authority, and callers MUST retain persistence, time, authenticity, recovery, outage, and effect authority.

#### Scenario: Pilot evidence is recorded

- GIVEN the shared adapters return bounded state observations
- WHEN Basalt records the pilot result
- THEN it MUST deny claims of global replay truth, revocation authenticity, backend honesty, effect completion, or release eligibility

#### Scenario: State evidence is treated as live authority

- GIVEN a digest or receipt is presented without live Basalt and UCAN admission
- WHEN authority is requested
- THEN Basalt MUST deny the request

### Requirement: Second-consumer validation

r[basalt.durable_authority.validation] Basalt MUST run positive, negative, source, release, Cairn, and Nix checks for the durable-authority composition.

#### Scenario: Pilot gate passes

- GIVEN exact source and complete positive and negative fixtures
- WHEN the repository verification gate runs
- THEN every declared check MUST pass or report one exact bounded inherited blocker

#### Scenario: A negative state case is accepted

- GIVEN duplicate, stale, incomplete, unauthenticated, unavailable, or fallback state is accepted
- WHEN the pilot suite runs
- THEN the suite MUST fail

### Requirement: Pilot lifecycle completion

r[basalt.durable_authority.lifecycle] The pilot MUST sync accepted requirements, archive completed tasks, and publish the verified completion through `origin/main`.

#### Scenario: Pilot is complete

- GIVEN source, implementation, validation, sync, archive, and integration evidence pass
- WHEN completion is evaluated
- THEN the accepted spec and archive MUST be reachable from `origin/main`

#### Scenario: Pilot is incomplete

- GIVEN any required evidence is absent
- WHEN completion is evaluated
- THEN the change MUST remain active or blocked
