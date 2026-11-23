# Typed Policy Operations Specification

## ADDED Requirements

### Requirement: Versioned bounded operation profile
r[lattice.typed_policy.profile] Lattice MUST describe a typed policy operation with a versioned `lattice.typed-policy-profile.v1` profile that declares one `select` task shape, its workflow and node binding, an input projection with schema identity, byte limit and per-feature integer pointer and range, a bounded unique label set, a result schema with a receipt byte limit, a deterministic operation budget and a positive wall-clock deadline, the exact artifact/runtime cohort, and one declared fallback. Profiles with unknown fields, unsupported shapes, empty or duplicate labels, unbounded limits or malformed identities MUST be refused before any invocation. The profile identity MUST be the canonical BLAKE3 identity of its encoding.

#### Scenario: Valid profile is admitted
r[lattice.typed_policy.profile.scenario.valid]
- GIVEN a profile whose fields, limits, labels and identities are within bounds
- WHEN Lattice admits it
- THEN it returns a stable profile identity and a typed profile.

#### Scenario: Unbounded or unsupported profile is refused
r[lattice.typed_policy.profile.scenario.invalid]
- GIVEN a profile with an unsupported shape, an unknown field, a zero deadline, a duplicate label or an unsafe identity
- WHEN Lattice admits it
- THEN admission refuses with a typed error and no binding is created.

### Requirement: Route only through declared continuations
r[lattice.typed_policy.route] An enabled profile MUST bind to exactly one Condition node of its named workflow, and every profile label MUST already be a declared branch of that node. The policy result MUST select at most one declared branch and MUST flow only through the existing Condition output ports, ready-frontier admission and effect authority. A policy result MUST NOT create a node, port, tool, credential, protocol label or effect, bypass capacity or conflict planning, advance a role-protocol cursor, or settle an uncertain effect. Disabled or absent profiles MUST leave Condition evaluation, cache keys and run records byte-identical.

#### Scenario: Accepted label routes through the existing branch
r[lattice.typed_policy.route.scenario.accepted]
- GIVEN an enabled profile bound to a Condition node with declared branches
- WHEN the policy result is accepted
- THEN only the matching declared branch port receives the Condition input and downstream nodes run under existing admission.

#### Scenario: Label outside the declared branches
r[lattice.typed_policy.route.scenario.undeclared]
- GIVEN a profile naming a label that is not a declared branch, or a binding to a non-Condition node
- WHEN the executor reaches that node
- THEN the node fails before invocation and no undeclared continuation runs.

#### Scenario: Profile is disabled
r[lattice.typed_policy.route.scenario.disabled]
- GIVEN a disabled or absent profile
- WHEN the workflow runs
- THEN results, cache keys and serialized run details equal the run without any profile.

### Requirement: Identity-bound frozen policy invocation
r[lattice.typed_policy.invocation] Lattice MUST invoke the frozen policy only through the Animus adapter with immutable artifact, model, feature-schema, action-schema, policy-kind and runtime identities from the profile. A mutable model name or file path MUST NOT serve as identity. Inference and artifact loading MUST remain outside the pure Lattice core, and the host MUST enforce the wall-clock deadline and observe cancellation.

#### Scenario: Artifact bytes do not match the bound identity
r[lattice.typed_policy.invocation.scenario.stale]
- GIVEN an artifact whose canonical identity differs from the profile cohort
- WHEN the adapter admits it
- THEN the decision is not accepted and the profile's fallback or terminal path is taken.

### Requirement: Independent typed validation
r[lattice.typed_policy.validation] Lattice MUST validate every adapter response independently: receipt size, receipt integrity, schema, runtime cohort, artifact and model identities, bound schemas, projected features, candidate set, label membership and outcome consistency. It MUST classify the response as `accepted`, `abstained`, `unavailable`, `timed_out`, `invalid` or `cancelled`. Confidence or scores MUST NOT grant permission.

#### Scenario: Malformed, oversized or undeclared output
r[lattice.typed_policy.validation.scenario.invalid]
- GIVEN a malformed or oversized receipt, an undeclared label, a missing candidate or a crossed binding
- WHEN Lattice validates it
- THEN the outcome is `invalid` with a bounded reason.

### Requirement: Declared bounded fallback
r[lattice.typed_policy.fallback] Every non-accepted outcome MUST take the profile's declared fallback: one declared label, the Condition node's own deterministic predicates when exactly one matches, or an explicit terminal failure. Cancellation MUST be terminal and MUST NOT take a fallback.

#### Scenario: Timeout or unavailable backend
r[lattice.typed_policy.fallback.scenario.timeout]
- GIVEN a wall-clock timeout, a deterministic operation-budget refusal or an unavailable backend
- WHEN the outcome is resolved
- THEN the declared fallback routes or the node fails explicitly, and the record names the reason.

### Requirement: Durable decision record
r[lattice.typed_policy.record] Each invocation MUST persist one bounded `lattice.typed-policy-decision.v1` record in the existing run detail record of the Condition node with invocation, projected-input, candidate-set, artifact, model and profile identities, outcome, accepted and routed labels, fallback reason, the raw receipt when the profile permits retention, reserved and observed operation units kept distinct, and the eventual workflow outcome. Runs that suspend at or resume from a durable approval gate MUST persist their decisions too, leaving the eventual workflow outcome absent until the run finishes. Runs without a bound profile MUST serialize no policy field.

#### Scenario: Decision is persisted with the run
r[lattice.typed_policy.record.scenario.persisted]
- GIVEN a headless run that executes a bound Condition node
- WHEN the run detail record is saved and reloaded
- THEN it carries exactly one decision record whose identities match the invocation.

#### Scenario: Decision is persisted across approval suspension and resume
r[lattice.typed_policy.record.scenario.suspended]
- GIVEN a bound Condition node that decides before or after a durable approval gate
- WHEN the run suspends, is approved and resumes with the same profile
- THEN the suspended run detail carries any decision made so far without a workflow outcome, and the finished run detail carries every decision with the eventual workflow outcome.

### Requirement: Replay without inference
r[lattice.typed_policy.replay] Replay MUST reconstruct the routed label from the persisted record and receipt only, starting no adapter, model or effect work. A changed profile or artifact MUST be reported as a compatibility refusal and MUST NOT rewrite the prior decision. Cached re-execution MUST key on the bound profile identity. A resume from a durable approval gate MUST revalidate every stored decision against the current profile before the resume is claimed, reuse a compatible decision without adapter work, and refuse a stale one without re-deciding it.

#### Scenario: Persisted decision replays
r[lattice.typed_policy.replay.scenario.persisted]
- GIVEN a persisted accepted or fallback decision
- WHEN it is replayed against the same profile and workflow
- THEN the same routed label is returned without any adapter call.

#### Scenario: Profile changed after the decision
r[lattice.typed_policy.replay.scenario.changed]
- GIVEN a persisted decision and a changed profile or artifact
- WHEN it is replayed
- THEN replay refuses as incompatible and the record is unchanged.

#### Scenario: Resume after an approval gate
r[lattice.typed_policy.replay.scenario.resumed]
- GIVEN a decision persisted before an approval suspension
- WHEN the approved run resumes with the same profile, or with a changed profile, artifact or no bound profile
- THEN the same profile reuses the stored decision with no adapter call, and any other profile refuses the resume as stale before it is claimed, leaving the stored record and approval request unchanged.

### Requirement: Stateless first profile
r[lattice.typed_policy.stateless] A profile requesting session, recurrent or checkpointed state MUST be refused. The Animus agent session rejection remains in force.

#### Scenario: Stateful profile is requested
r[lattice.typed_policy.stateless.scenario.refused]
- GIVEN a profile declaring `stateful = true`
- WHEN Lattice admits it
- THEN admission refuses before any binding.

### Requirement: Pre-declared comparative measurement
r[lattice.typed_policy.measurement] The first fixture MUST compare the deterministic rule, the present reasoning-model path and the fast policy on the same admitted label set under thresholds declared before evaluation, using held-out task families, retaining failures, abstentions and unavailable arms, and reporting whole-workflow success, total latency and deterministic cost, coverage, invalid-output and fallback rates. No improvement claim MAY be made from a fixture artifact.

#### Scenario: An arm cannot run
r[lattice.typed_policy.measurement.scenario.unavailable]
- GIVEN an arm whose runtime or credentials are unavailable
- WHEN the comparison is recorded
- THEN the arm is reported as unavailable with its reason and the verdict cannot claim adoption.
