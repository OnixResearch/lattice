# Resource Operation Plans Delta

## ADDED Requirements

### Requirement: Resource operation roles are nominal

r[lattice.resource_operations.roles] Lattice MUST keep logical resource, exact content, replica, locator, observation, plan, attempt, and outcome identities in separate nominal roles.

#### Scenario: Valid catalog facts enter planning

- GIVEN a validated producer supplies distinct resource, content, replica, locator, and observation values
- WHEN Lattice adapts the values for planning
- THEN each value MUST retain its role and producer identity
- AND transport, storage, and display values MUST remain outside the pure plan types.

#### Scenario: A locator substitutes for content identity

- GIVEN an input places a path, URL, device label, or replica locator in an exact content field
- WHEN role validation runs
- THEN Lattice MUST reject the input before planning or effects.

### Requirement: Preview plans bind exact supplied facts

r[lattice.resource_operations.plan] Lattice MUST produce a canonical preview plan that binds exact operation, resource, content, replica, observation, policy, authority, runtime, estimate, conflict, effect, and non-claim facts.

#### Scenario: A valid copy preview is created

- GIVEN source and destination facts are current, bounds are positive, conflicts are resolved, and capability policy admits preview
- WHEN pure planning runs
- THEN Lattice MUST emit a deterministic plan and full-width domain-separated BLAKE3 identity
- AND planning MUST perform no catalog, filesystem, cloud, network, clock, persistence, credential, logging, or UI effect.

#### Scenario: A sampled fingerprint is supplied as exact content

- GIVEN a request supplies sampled bytes, a shortened digest, or a heuristic duplicate score as `ContentRef`
- WHEN plan admission runs
- THEN Lattice MUST reject the request
- AND it MUST NOT use the fingerprint for deduplication, integrity, artifact, or authority claims.

### Requirement: Resource effects use a closed plan

r[lattice.resource_operations.effects] Lattice MUST limit the first resource-operation contract to copy replica, move replica, materialize content, generate derivative, and delete replica effects.

#### Scenario: An admitted effect is planned

- GIVEN the selected operation method maps to one closed effect and all required facts are present
- WHEN effect planning runs
- THEN the plan MUST name the exact requested effect, target roles, bounds, conflicts, capability class, and approval class.

#### Scenario: Global deletion is requested

- GIVEN a request asks to delete a logical resource, all replicas, or globally shared content
- WHEN the first contract evaluates the request
- THEN Lattice MUST reject it as unsupported
- AND it MUST NOT reinterpret the request as replica deletion.

### Requirement: External capabilities cross narrow ports

r[lattice.resource_operations.ports] Lattice MUST obtain catalog facts, authority decisions, approvals, resource effects, and outcome records through application-owned ports with Lattice types.

#### Scenario: A fake catalog supplies bounded facts

- GIVEN a test composition uses an in-memory fake catalog
- WHEN a resource plan is prepared
- THEN the core MUST receive ordinary typed facts without database, OpenDAL, filesystem, Iroh, or UI values.

#### Scenario: A required port is unavailable

- GIVEN catalog, authority, approval, executor, or outcome recording is unavailable
- WHEN the shell reaches that boundary
- THEN it MUST return a typed non-success outcome
- AND it MUST NOT discover or construct an ambient fallback.

### Requirement: Approval and currentness precede protected effects

r[lattice.resource_operations.approval] Lattice MUST bind approval to the exact plan identity and MUST recheck catalog revision, observations, policy, authority, and runtime before any protected resource effect.

#### Scenario: Exact approved plan executes

- GIVEN an authorized operator approved the current exact plan and all bound facts still match
- WHEN execution begins
- THEN the shell MAY dispatch only the effects listed in that plan
- AND the executor MUST receive no broader authority than the approved capability set.

#### Scenario: Facts changed after preview

- GIVEN a replica, locator, observation, conflict, policy, authority, runtime, or plan identity changed after preview
- WHEN execution rechecks currentness
- THEN execution MUST stop before protected effects
- AND the operator MUST receive a fresh preview before another attempt.

### Requirement: Outcomes preserve uncertainty

r[lattice.resource_operations.outcomes] Lattice MUST record completed, denied, failed, cancelled, and pending-unobserved resource attempts without inventing external results or claiming exactly-once execution.

#### Scenario: Effect completion is observed

- GIVEN an executor returns a bounded result and post-effect observations match the plan outcome contract
- WHEN the shell settles the attempt
- THEN Lattice MUST record the completed outcome with exact attempt, plan, and observation identities.

#### Scenario: The process stops after dispatch

- GIVEN an effect was dispatched but no accepted terminal observation exists
- WHEN recovery inspects the attempt
- THEN it MUST remain pending-unobserved or become explicitly abandoned-unknown
- AND automatic retry MUST NOT occur.

### Requirement: Verification covers acceptance and rejection

r[lattice.resource_operations.verification] The change MUST include positive and negative pure, shell, adapter, approval, currentness, cancellation, uncertainty, and retry tests.

#### Scenario: Closeout evidence is complete

- GIVEN maintainers intend to archive the change
- WHEN closeout review runs
- THEN evidence MUST name focused Cargo, Clippy, Octet, Cairn, positive, and negative checks
- AND each skipped check MUST include an exact blocker.

### Requirement: Evidence retains bounded claims

r[lattice.resource_operations.evidence] Resource-operation plans and receipts MUST state that they do not prove catalog truth, filesystem stability, transfer completion, storage durability, remote deduplication, exactly-once effects, authorization beyond supplied facts, or release readiness.

#### Scenario: A plan passes admission

- GIVEN pure planning returned a valid plan
- WHEN the plan is displayed or recorded
- THEN the output MUST describe only admitted intent and supplied facts
- AND it MUST NOT claim that any requested effect occurred.
