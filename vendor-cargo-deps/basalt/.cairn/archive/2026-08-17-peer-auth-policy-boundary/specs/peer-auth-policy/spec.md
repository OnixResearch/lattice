# Peer Auth Policy Boundary Specification

## Purpose

Define a typed, reviewable peer-authentication policy and enforcement boundary for Basalt without transferring transport, key custody, clock, or replay authority.

## ADDED Requirements

### Requirement: Peer-auth policy stays a reviewable contract

r[basalt.peer_auth.policy] Basalt MUST provide peer-authentication policy as a typed Nickel contract. The policy MUST declare message, version, freshness, replay, and authority facts.

#### Scenario: Operator supplies a policy

- GIVEN an operator authors a peer-auth policy with explicit version, freshness, replay, and authority facts
- WHEN policy typecheck and review run
- THEN the policy MUST typecheck
- AND every declared fact MUST be explicit

#### Scenario: Policy omits a required fact

- GIVEN a peer-auth policy omits a version, freshness, replay, or authority fact
- WHEN policy typecheck runs
- THEN typecheck MUST fail
- AND no default fact MUST be assumed

### Requirement: Enforcement evaluates supplied verdicts deterministically

r[basalt.peer_auth.enforcement] Basalt MUST evaluate the policy over an authenticated-message verdict and caller-supplied expectations. Enforcement MUST read no clock, key store, or replay store.

#### Scenario: Admitted peer message

- GIVEN a verdict matches the policy version, freshness, replay, and authority facts
- WHEN enforcement runs
- THEN enforcement MUST admit the action with the required authority
- AND the message content MUST NOT widen the caller policy

#### Scenario: Denied peer message

- GIVEN a wrong version, denied freshness, replay denial, or missing authority
- WHEN enforcement runs
- THEN enforcement MUST deny with a deterministic issue code
- AND no decision MUST authorize the action

### Requirement: Decisions do not grant end-user authority

r[basalt.peer_auth.decision] Only an admitted policy evaluation MUST construct the peer-auth decision. The type MUST represent bounded authentication for one bound action.

#### Scenario: Consumer receives a decision

- GIVEN the peer-auth policy admits an action
- WHEN enforcement returns the decision
- THEN the value MUST contain only bounded public facts
- AND it MUST state that end-user authentication and authorization remain downstream responsibilities

#### Scenario: Metadata attempts promotion

- GIVEN a message label, transport hint, or key label claims end-user identity or general authority
- WHEN enforcement evaluates the input
- THEN that metadata MUST NOT create a decision or grant authority

### Requirement: The reference stays bounded

r[basalt.peer_auth.reference] The change MUST record the reviewed Celld peer-protocol design as a bounded, non-parity input with its revision and license evidence.

#### Scenario: Reference overclaims parity

- GIVEN documentation claims parity with, or equivalence to, the Celld peer protocol
- WHEN reference verification runs
- THEN that claim MUST fail
- AND the reference MUST remain non-parity

### Requirement: Consumer boundary stays explicit

r[basalt.peer_auth.boundary] Transport, key custody, clock, replay, end-user, and authorization MUST remain consumer-owned. The boundary MUST NOT claim them.

#### Scenario: Projection overstates built-in support

- GIVEN a support surface claims built-in transport, key custody, clock, replay, or authorization support
- WHEN support verification runs
- THEN verification MUST fail
- AND the consumer-owned boundary MUST remain explicit

### Requirement: Roles and failure coverage remain explicit

r[basalt.peer_auth.verification] The boundary MUST preserve BLAKE3 identities separately from raw secrets. Positive and negative fixtures MUST cover every declared boundary.

#### Scenario: Digest or secret substitution fails

- GIVEN a raw secret, an empty message, or a substituted digest role is supplied as the message identity
- WHEN typed evaluation runs
- THEN evaluation MUST fail with a clear issue
- AND the raw value MUST NOT be exposed except as a bounded identity

#### Scenario: Complete focused matrix passes

- GIVEN policy, enforcement, fixtures, documentation, and dependency evidence are complete
- WHEN focused policy, package, workspace, Clippy, Cairn, and Nix verification runs
- THEN admitted inputs MUST pass
- AND each wrong-version, stale, replayed, secret-bearing, or overclaiming input MUST fail as declared
