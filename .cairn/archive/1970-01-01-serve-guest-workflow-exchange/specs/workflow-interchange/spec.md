## ADDED Requirements

### Requirement: Listener-generic exchange serving

r[lattice.workflow_interchange.guest_listener] The exchange server MUST drive every listener kind through one bounded serve loop that observes the peer, applies profile admission before reading, applies the profile deadlines, and enforces frame and log limits. The Unix listener MUST keep its user-ID admission, wire bytes and failure classes. A caller-connected stream MUST receive the same client classification as `UnixClient`.

#### Scenario: Unix behaviour is unchanged
r[lattice.workflow_interchange.guest_listener.scenario.unix]
- GIVEN the existing Unix transport and application suites
- WHEN they run against the listener-generic serve loop
- THEN they MUST pass unchanged.

### Requirement: Guest vsock transport profile

r[lattice.workflow_interchange.vsock_profile] Lattice MUST publish `lattice.workflow-vsock-transport-profile.v1` with a port, the transport limits, a strictly ascending non-empty peer-CID allow-list of at most 64 entries, and every transport non-claim. It MUST refuse `VMADDR_PORT_ANY`, port zero, and `VMADDR_CID_ANY` as a peer.

#### Scenario: Admitted profile
r[lattice.workflow_interchange.vsock_profile.scenario.admitted]
- GIVEN a vsock profile whose allow-list holds the host CID
- WHEN it is validated
- THEN it MUST be admitted, and only listed CIDs MUST be admissible peers.

#### Scenario: Refused profile
r[lattice.workflow_interchange.vsock_profile.scenario.refused]
- GIVEN a vsock profile with a wrong schema, an invalid port, invalid limits, an empty, unsorted, duplicated, oversized or wildcard CID list, missing non-claims, or an unknown field
- WHEN it is validated or decoded
- THEN it MUST be refused.

### Requirement: Guest vsock listener

r[lattice.workflow_interchange.vsock_listener] The vsock listener MUST bind `VMADDR_CID_ANY:port`, observe each peer CID from the kernel, and invoke the handler only for allowed CIDs, using the same frames, limits and failure classes as the Unix transport.

#### Scenario: Unlisted peer
r[lattice.workflow_interchange.vsock_listener.scenario.denied]
- GIVEN a connection whose observed peer CID is not allowed
- WHEN the guest serves it
- THEN the handler MUST NOT be invoked and the connection MUST count as a rejected peer.

### Requirement: Handler profile version two

r[lattice.workflow_interchange.handler_profile_v2] Lattice MUST accept `lattice.workflow-exchange-handler-profile.v2` with an explicit `unix` or `vsock` listener under its own identity domain, while version one stays accepted with an unchanged shape and identity. Both versions MUST refuse workflow pause and resume.

#### Scenario: Pause and resume stay refused
r[lattice.workflow_interchange.handler_profile_v2.scenario.controls]
- GIVEN a version-two profile that advertises pause or resume
- WHEN it is validated or a handler starts with it
- THEN it MUST be refused.

#### Scenario: Listener validation
r[lattice.workflow_interchange.handler_profile_v2.scenario.listener]
- GIVEN a version-two profile whose listener is invalid, whose active limit exceeds the listener's connection limit, or whose listener has an unknown kind or field
- WHEN it is validated or decoded
- THEN it MUST be refused.

#### Scenario: Separate identity domains
r[lattice.workflow_interchange.handler_profile_v2.scenario.identity]
- GIVEN equivalent version-one and version-two profiles
- WHEN their identities are derived
- THEN the version-one identity MUST equal its unchanged identity and the version-two identity MUST differ.

#### Scenario: Typed Nickel profiles
r[lattice.workflow_interchange.handler_profile_v2.scenario.typed]
- GIVEN the checked-in version-one and version-two Nickel profiles and the negative version-two profiles
- WHEN they are loaded
- THEN the positive profiles MUST load as their own version and the negative ones MUST be refused.

### Requirement: Reconnect rule table

r[lattice.workflow_interchange.reconnect_rules] Lattice MUST publish the pure rule table `lattice.workflow-exchange-recovery.v1`. It maps the in-flight command, the client outcome and gateway incarnation evidence to exactly one next step. An unknown dispatch MUST be recovered, not resent. A missing acceptance MUST be redispatched only on the same incarnation. An unknown control MUST be observed first, and a control whose expected sequence is behind MUST never be resent.

#### Scenario: Exhaustive fixtures
r[lattice.workflow_interchange.reconnect_rules.scenario.fixtures]
- GIVEN the checked-in rule-table fixtures
- WHEN every command, outcome and incarnation row is evaluated
- THEN each MUST equal the published step and match the planned table.

#### Scenario: Refused steps
r[lattice.workflow_interchange.reconnect_rules.scenario.refused]
- GIVEN the fixture's forbidden steps
- WHEN the table is evaluated for those inputs
- THEN it MUST never produce them.

#### Scenario: Lost dispatch response
r[lattice.workflow_interchange.reconnect_rules.scenario.lost-dispatch]
- GIVEN a dispatch whose response is lost after the guest handled it
- WHEN the host follows the rule table
- THEN recovery MUST return the same acceptance, and a replayed identical dispatch MUST NOT create a second execution.

#### Scenario: Incomplete write
r[lattice.workflow_interchange.reconnect_rules.scenario.not-accepted]
- GIVEN a dispatch that is not completely written
- WHEN the host follows the rule table
- THEN nothing MUST be admitted, and the identical resend MUST be admitted once.

#### Scenario: Cancel racing reconnect
r[lattice.workflow_interchange.reconnect_rules.scenario.cancel-race]
- GIVEN a cancel whose response is lost
- WHEN the host observes before deciding
- THEN the durable record MUST keep the cancellation request, the observation MUST NOT report cancelled until the worker is observed stopped, and the superseded control MUST NOT be resent.

### Requirement: Observation continuity

r[lattice.workflow_interchange.observation_continuity] Lattice MUST publish the pure check `lattice.workflow-observation-continuity.v1`. It rejects a changed request, execution or workflow identity, a decreased sequence, a changed terminal state, and a log cursor that moves backwards for any (node, stream) pair. It MUST report a gap for every log reference with `gap_before`, without `retained`, or with `truncated`.

#### Scenario: Continuity fixtures
r[lattice.workflow_interchange.observation_continuity.scenario.fixtures]
- GIVEN the checked-in continuity fixtures
- WHEN each case is checked
- THEN the result MUST equal the published outcome, and the fixtures MUST cover continuous, gap and every rejection reason.

#### Scenario: Handler restart
r[lattice.workflow_interchange.observation_continuity.scenario.restart]
- GIVEN a running execution whose handler incarnation is replaced
- WHEN the new handler reports it
- THEN the observation MUST be `Unknown` with a higher sequence, MUST be continuous with the last running observation, and MUST NOT be overwritten by the abandoned worker.

#### Scenario: Log resume with explicit gap
r[lattice.workflow_interchange.observation_continuity.scenario.gap]
- GIVEN a log read whose response is lost
- WHEN the host retries at the same cursor
- THEN it MUST receive the same bytes, a later reference marked `gap_before` MUST be reported as a gap, and a cursor that moves back MUST be rejected.
