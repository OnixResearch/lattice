# GitHub CI Specification

## ADDED Requirements

### Requirement: lattice.github_ci.oidc_acquisition

r[lattice.github_ci.oidc_acquisition]

Lattice SHALL acquire a GitHub Actions OIDC token only through the documented job endpoint, for an explicit Basalt audience, under named HTTPS, host, redirect, timeout, and response bounds.

#### Scenario: GitHub job has identity permission

- **GIVEN** documented request variables, an allowed endpoint, and an explicit Basalt audience
- **WHEN** the transport shell requests identity
- **THEN** it SHALL obtain one transient OIDC token
- **AND** it SHALL pass that token only to the Basalt exchange client

#### Scenario: OIDC acquisition is unsafe

- **GIVEN** missing permission, missing variables, an unapproved URL, unsafe redirect, TLS failure, timeout, oversized body, or malformed response
- **WHEN** Lattice requests identity
- **THEN** the job SHALL fail closed
- **AND** it SHALL NOT fall back to a static Mantle token

### Requirement: lattice.github_ci.attenuated_exchange

r[lattice.github_ci.attenuated_exchange]

Lattice SHALL request the minimum Mantle capability needed by one planned job and SHALL reject an exchanged UCAN whose public audience, operations, resource scope, or expiry exceeds that plan.

#### Scenario: Basalt grants the planned capability

- **GIVEN** an accepted OIDC token and a least-authority job plan
- **WHEN** Basalt returns a matching UCAN
- **THEN** Lattice MAY use it for the planned Mantle requests until expiry

#### Scenario: Returned capability is broader

- **GIVEN** a UCAN with another audience, operation, project, resource, or excessive expiry
- **WHEN** Lattice validates public token facts
- **THEN** it SHALL reject the token before Mantle submission

### Requirement: lattice.github_ci.pure_job_planning

r[lattice.github_ci.pure_job_planning]

Matrix expansion, job identity, concrete request construction, capability planning, idempotency, and state reduction SHALL be pure functions over explicit workflow inputs, policy, events, and time.

#### Scenario: Matrix job is planned

- **GIVEN** a workflow node and valid matrix coordinates
- **WHEN** the planner runs
- **THEN** it SHALL return a canonical BLAKE3-identified job, concrete Mantle request, capability request, and idempotency key
- **AND** it SHALL require no environment, network, filesystem, credential, clock, or process access

#### Scenario: Matrix expansion exceeds policy

- **GIVEN** coordinates whose product exceeds the named job bound
- **WHEN** the planner validates expansion
- **THEN** it SHALL reject the workflow before any remote submission

### Requirement: lattice.github_ci.durable_async_submission

r[lattice.github_ci.durable_async_submission]

Lattice SHALL submit each concrete job with a stable idempotency key and SHALL persist only public attempt, cursor, Check Run, state-version, and evidence identities for asynchronous recovery.

#### Scenario: Submission response is lost

- **GIVEN** Mantle admitted a request but Lattice did not receive the response
- **WHEN** Lattice repeats the same authenticated request and idempotency key
- **THEN** it SHALL recover the same authorized attempt
- **AND** it SHALL NOT intentionally create a second attempt

#### Scenario: Recovery state contains a credential

- **GIVEN** proposed state that contains a GitHub token, OIDC token, UCAN bearer token, provider value, or value-derived hash
- **WHEN** the state core validates persistence
- **THEN** it SHALL reject the state

### Requirement: lattice.github_ci.idempotent_event_reducer

r[lattice.github_ci.idempotent_event_reducer]

Lattice SHALL reduce verified Mantle events through deterministic attempt-identity, event-identity, sequence, transition, and terminal-state rules.

#### Scenario: Next valid event arrives

- **GIVEN** the current public state and the next signed event for the same attempt
- **WHEN** the reducer applies it
- **THEN** it SHALL produce one next workflow and Check Run projection

#### Scenario: Duplicate or gapped event arrives

- **GIVEN** a duplicate event or an event after a sequence gap
- **WHEN** the reducer evaluates it
- **THEN** a duplicate SHALL be an idempotent no-op
- **AND** a gap SHALL require bounded Mantle reconciliation before later transition

#### Scenario: Event changes a terminal state

- **GIVEN** a terminal workflow state
- **WHEN** a non-identical later event attempts to change the terminal class
- **THEN** the reducer SHALL reject the transition and preserve the terminal state

### Requirement: lattice.github_ci.bounded_check_runs

r[lattice.github_ci.bounded_check_runs]

Lattice SHALL create and update GitHub Check Runs idempotently with named annotation, title, summary, text, retry, and rate-limit bounds. It SHALL link long Mantle logs instead of copying them.

#### Scenario: Attempt completes

- **GIVEN** a valid terminal Mantle event and an existing Check Run
- **WHEN** Lattice projects the terminal state
- **THEN** it SHALL send one bounded idempotent update with evidence and log references

#### Scenario: GitHub API is unavailable or denies access

- **GIVEN** a rate limit, abuse response, transient error, revoked credential, missing repository, or permission denial
- **WHEN** the Check Run shell handles the response
- **THEN** it SHALL follow the named retry or terminal policy
- **AND** it SHALL preserve recoverable public orchestration state

### Requirement: lattice.github_ci.secretspec_github_credentials

r[lattice.github_ci.secretspec_github_credentials]

Lattice SHALL resolve GitHub API credentials through its accepted credential registry and the immutable Animus SecretSpec Rust SDK adapter. Workflow definitions and persisted state SHALL contain references only.

#### Scenario: GitHub credential resolves

- **GIVEN** an explicit credential reference, profile, scope, and allowed provider
- **WHEN** the Check Run shell requests the credential
- **THEN** the credential registry SHALL return an opaque transient value
- **AND** the workflow core SHALL remain provider-independent

#### Scenario: Credential cannot resolve

- **GIVEN** a missing declaration, wrong scope, provider failure, timeout, or oversized provider output
- **WHEN** the Check Run shell requests the credential
- **THEN** GitHub mutation SHALL remain disabled
- **AND** Lattice SHALL NOT use an ambient fallback

### Requirement: lattice.github_ci.evidence_boundary

r[lattice.github_ci.evidence_boundary]

Lattice SHALL link workflow, matrix job, Basalt exchange, Mantle attempt, Mantle event, result, and GitHub Check Run identities through Valence without recording bearer or provider secret material.

#### Scenario: Check Run succeeds

- **GIVEN** a successful terminal orchestration
- **WHEN** Lattice emits evidence
- **THEN** it SHALL record bounded observed identities and outcomes
- **AND** it SHALL state that the Check Run does not prove build correctness, hermeticity, host isolation, identity beyond checked claims, or release eligibility
