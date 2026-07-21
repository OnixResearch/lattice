## ADDED Requirements

### Requirement: Durable orchestration model
r[lattice.agent_orchestration.model] Lattice MUST model a durable orchestration as a typed definition and instance bound to immutable BLAKE3-identified definition and workflow revisions, explicit objectives, termination predicates, budgets, authority policy, trigger policy, and redaction policy.

#### Scenario: Valid definition creates one durable identity
r[lattice.agent_orchestration.model.scenario.valid]
- GIVEN a typed orchestration definition references an available immutable workflow revision and valid objective, termination, budget, authority, trigger, and redaction policies
- WHEN Lattice validates and starts the definition
- THEN it MUST create one durable instance bound to the exact definition and workflow revision identities

#### Scenario: Mutable workflow-name drift is rejected
r[lattice.agent_orchestration.model.scenario.revision-drift]
- GIVEN a definition names a workflow whose current content does not match the referenced workflow revision hash
- WHEN orchestration admission runs
- THEN Lattice MUST reject the definition before creating an instance or executing workflow effects

### Requirement: Pure orchestration decision core
r[lattice.agent_orchestration.core] Lattice MUST advance orchestration lifecycle, objective, termination, approval, and next-round decisions through a pure deterministic core over in-memory definition, state, policy, and explicit fact inputs.

#### Scenario: Terminal objective completes the instance
r[lattice.agent_orchestration.core.scenario.complete]
- GIVEN a running instance receives a terminal round result whose typed outputs satisfy its configured success predicate
- WHEN the pure orchestration core evaluates that fact
- THEN it MUST return a completion decision without requesting another round

#### Scenario: Terminal state mutation is rejected
r[lattice.agent_orchestration.core.scenario.terminal-mutation]
- GIVEN an instance is completed, failed, cancelled, or exhausted
- WHEN a fact requests another run, approval, resume, or revision transition
- THEN the pure core MUST reject the transition and MUST produce no side-effecting decision

### Requirement: Seaglass-bounded orchestration rounds
r[lattice.agent_orchestration.rounds] Lattice MUST execute each orchestration round as a normal immutable typed workflow run whose graph admission, ordering, readiness, and lifecycle decisions use the existing Seaglass-backed runner boundary.

#### Scenario: Admitted round delegates graph decisions
r[lattice.agent_orchestration.rounds.scenario.execute]
- GIVEN an orchestration decision starts a round for a valid immutable workflow revision
- WHEN the runner plans and executes that round
- THEN graph-generic decisions MUST remain delegated to Seaglass while Lattice retains runtime, value, policy, event, artifact, objective, and authority ownership

#### Scenario: Feedback cannot create a hidden cycle
r[lattice.agent_orchestration.rounds.scenario.hidden-cycle]
- GIVEN a round output proposes feedback to an earlier role or graph step
- WHEN the current round has not reached a terminal state or the proposed successor graph is cyclic
- THEN Lattice MUST reject immediate continuation and MUST NOT bypass normal cycle admission

### Requirement: Authority-gated agent proposals
r[lattice.agent_orchestration.authority] Lattice MUST treat agent-produced workflow, policy, trigger, credential, effect, or budget changes as inert proposals until an authorized decision binds the instance epoch, base revision, proposal hash, authority-policy hash, actor authority reference, and decision.

#### Scenario: Authorized proposal becomes a successor revision
r[lattice.agent_orchestration.authority.scenario.approved]
- GIVEN a terminal round produced a valid proposal and an authorized approval binds the current instance epoch, base revision, proposal, policy, and actor authority identities
- WHEN proposal admission runs
- THEN Lattice MAY adopt the exact successor revision and MUST record the approval and revision lineage before another round starts

#### Scenario: Agent output has no ambient mutation authority
r[lattice.agent_orchestration.authority.scenario.ambient-denied]
- GIVEN an agent output requests a new credential, trigger, runtime ability, budget increase, graph revision, or self-approval
- WHEN no matching authorized decision exists
- THEN Lattice MUST keep the output inert and MUST NOT mutate active orchestration or workflow authority

#### Scenario: Stale approval is rejected
r[lattice.agent_orchestration.authority.scenario.stale]
- GIVEN an approval references an earlier instance epoch, base revision, proposal hash, or authority-policy hash
- WHEN approval admission runs against the current instance
- THEN Lattice MUST reject the approval without adopting any successor revision

#### Scenario: Approval cannot bypass successor validation
r[lattice.agent_orchestration.authority.scenario.validation]
- GIVEN an authorized approval binds a proposed successor whose graph, ports, runtime bindings, effects, credentials, triggers, budgets, or policy are invalid under the current definition and runtime policy
- WHEN proposal admission runs
- THEN Lattice MUST reject the successor despite the approval and MUST execute no successor effects

### Requirement: Multi-dimensional budget admission
r[lattice.agent_orchestration.budgets] Lattice MUST reserve and settle configured round, agent-invocation, token, monetary, and duration budgets through deterministic admission before starting new orchestration work.

#### Scenario: Bounded round reserves before dispatch
r[lattice.agent_orchestration.budgets.scenario.reserve]
- GIVEN an admitted round has policy-acceptable upper-bound charges within every enforced remaining balance
- WHEN dispatch planning runs
- THEN Lattice MUST atomically reserve those charges before emitting a dispatch intent

#### Scenario: Missing or exhausted bound blocks work
r[lattice.agent_orchestration.budgets.scenario.block]
- GIVEN an enforced budget dimension has no policy-acceptable upper bound or insufficient remaining balance
- WHEN dispatch planning runs
- THEN Lattice MUST block the round without invoking the workflow runner or an agent adapter

#### Scenario: Observed overage stops new work
r[lattice.agent_orchestration.budgets.scenario.overage]
- GIVEN settled provider or runtime usage exceeds the reserved charge for an enforced dimension
- WHEN Lattice records the observation
- THEN it MUST record a budget breach, consume the observed usage, stop new dispatch, and MUST NOT claim it reversed already-incurred external usage

#### Scenario: Concurrent reservations cannot oversubscribe a balance
r[lattice.agent_orchestration.budgets.scenario.concurrent]
- GIVEN two dispatch plans concurrently request reservations whose combined charge exceeds one enforced remaining balance
- WHEN the durable shell commits the reservation decisions
- THEN at most the policy-admitted reservation MUST commit and no runner invocation may start for a rejected reservation

### Requirement: Durable idempotent dispatch admission
r[lattice.agent_orchestration.durability] Lattice MUST persist each accepted orchestration event, budget reservation, and dispatch intent atomically and MUST reconcile stable BLAKE3 dispatch identities with run history after interruption.

#### Scenario: Restart reconciles acknowledged run
r[lattice.agent_orchestration.durability.scenario.reconcile]
- GIVEN Lattice persisted a dispatch intent and the runner accepted its stable run identity before the orchestrator process stopped
- WHEN orchestration recovery runs
- THEN Lattice MUST reconcile the existing run instead of creating a distinct replacement run

#### Scenario: Duplicate fact is idempotent
r[lattice.agent_orchestration.durability.scenario.duplicate]
- GIVEN a trigger, approval, acknowledgement, or completion fact with a previously accepted stable identity is received again
- WHEN the pure core and shell process it
- THEN durable orchestration state and budget usage MUST remain equivalent to processing that fact once

#### Scenario: Remote exactly-once remains an explicit boundary
r[lattice.agent_orchestration.durability.scenario.external-effects]
- GIVEN a runtime cannot propagate or honor an idempotency key across an indeterminate crash boundary
- WHEN status or evidence describes recovered execution
- THEN Lattice MUST identify the external effect as potentially at-least-once and MUST NOT claim remote exactly-once execution

### Requirement: Operator lifecycle controls
r[lattice.agent_orchestration.controls] Lattice MUST expose authority-checked pause, resume, cancellation, approval, and rejection controls that operate through the same deterministic orchestration decisions as automated inputs.

#### Scenario: Pause prevents new dispatch
r[lattice.agent_orchestration.controls.scenario.pause]
- GIVEN an authorized operator pauses an instance
- WHEN the pause decision is persisted
- THEN Lattice MUST start no new rounds until an authorized resume decision is accepted

#### Scenario: Cancel does not fabricate effect termination
r[lattice.agent_orchestration.controls.scenario.cancel]
- GIVEN an authorized operator cancels an instance with an active runner invocation
- WHEN Lattice forwards cancellation to the runner
- THEN it MUST record the request and observed runner outcome separately and MUST NOT report an unobserved external effect as terminated

### Requirement: Orchestration event ledger and replay
r[lattice.agent_orchestration.events] Lattice MUST persist an ordered redaction-safe hash-linked orchestration event ledger with a separately stored current-head identity and MUST reconstruct instance state, round lineage, approvals, revisions, budgets, and terminal outcomes without rerunning effects.

#### Scenario: Replay reconstructs without side effects
r[lattice.agent_orchestration.events.scenario.replay]
- GIVEN a valid orchestration event ledger references several workflow rounds, proposals, approvals, and artifacts
- WHEN an operator replays the instance
- THEN Lattice MUST fold the recorded events through the pure core and MUST NOT invoke Seaglass runners, node runtimes, credential providers, or external adapters

#### Scenario: Tampered ledger is rejected
r[lattice.agent_orchestration.events.scenario.tampered]
- GIVEN an event is removed, reordered, duplicated with conflicting content, or changed relative to the persisted instance head or a supplied checkpoint
- WHEN replay validation runs
- THEN Lattice MUST reject the ledger as untrusted and MUST NOT present the reconstructed state as valid history

#### Scenario: Local chain does not imply external completeness
r[lattice.agent_orchestration.events.scenario.non-claim]
- GIVEN an event ledger and its current-head identity validate locally without an independently controlled checkpoint or witness
- WHEN Lattice renders replay evidence
- THEN it MUST describe local sequence consistency and MUST NOT claim external anchoring, independently witnessed completeness, or protection from an attacker who can replace both local history and its local head

### Requirement: Orchestration trigger admission
r[lattice.agent_orchestration.triggers] Lattice MUST route schedule and webhook requests for orchestration start or signal actions through typed trigger, idempotency, overlap, payload, and authority admission before mutating an instance.

#### Scenario: Valid trigger starts one instance
r[lattice.agent_orchestration.triggers.scenario.start]
- GIVEN an enabled trigger references a valid orchestration definition and supplies compatible typed inputs with a fresh idempotency key
- WHEN trigger admission succeeds
- THEN Lattice MUST start exactly one durable instance for that admitted trigger fact

#### Scenario: Duplicate trigger does not duplicate work
r[lattice.agent_orchestration.triggers.scenario.duplicate]
- GIVEN an orchestration trigger idempotency key was already accepted
- WHEN the same trigger fact is received again
- THEN Lattice MUST return the prior admission outcome and MUST NOT create another instance or budget reservation

### Requirement: Typed Nickel orchestration format
r[lattice.agent_orchestration.format] Lattice MUST import and export orchestration definitions through typed Nickel contracts without plaintext secrets and MUST keep unsupported n8n orchestration semantics outside default parity claims.

#### Scenario: Nickel round trip preserves orchestration meaning
r[lattice.agent_orchestration.format.scenario.round-trip]
- GIVEN a valid definition contains objectives, revision references, typed bindings, budgets, authority, transitions, triggers, retry, redaction, and retention policy
- WHEN Lattice imports, exports, and reimports it
- THEN the resulting definition MUST preserve equivalent validated orchestration meaning and stable referenced identities

#### Scenario: Plaintext authority or credential material is rejected
r[lattice.agent_orchestration.format.scenario.secret]
- GIVEN a definition embeds a resolved credential, bearer token, private key, or untyped approval authority payload
- WHEN Nickel import validation runs
- THEN Lattice MUST reject the definition and MUST NOT persist the plaintext material

#### Scenario: Unsupported n8n export fails visibly
r[lattice.agent_orchestration.format.scenario.n8n-boundary]
- GIVEN an orchestration includes objectives, approvals, budgets, revision transitions, or multi-round state with no exact n8n mapping
- WHEN default n8n export is requested
- THEN Lattice MUST reject the export rather than emit workflow JSON that claims equivalent orchestration semantics

### Requirement: Headless orchestration CLI
r[lattice.agent_orchestration.cli] Lattice MUST provide deterministic headless commands to validate, start, inspect, pause, resume, cancel, approve, reject, and replay durable orchestration instances.

#### Scenario: Operator inspects actionable status
r[lattice.agent_orchestration.cli.scenario.status]
- GIVEN a persisted orchestration instance is running, paused, waiting for approval, terminal, or blocked by budget
- WHEN the operator requests text or JSON status
- THEN Lattice MUST report lifecycle, current revision and round, active run, remaining budgets, pending approval, last safe event, and one actionable next state without exposing secret material

#### Scenario: Invalid command leaves state unchanged
r[lattice.agent_orchestration.cli.scenario.invalid]
- GIVEN a command references a missing instance, stale proposal, unauthorized action, invalid transition, or malformed input
- WHEN the command runs
- THEN Lattice MUST return a deterministic failure and MUST leave persisted orchestration, workflow, credential, and trigger state unchanged
