# Agent Orchestration Specification

## Purpose

Defines the `agent-orchestration` capability.

## Requirements

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

### Requirement: Honest durable usage settlement
r[lattice.agent_orchestration.usage] The orchestration shell MUST retain a host-owned receipt distinguishing reservation, conservative charged budget, host run interval, provider reports and unknown coverage before settlement. Provider reports and model-written assertions MUST NOT be relabeled as measured billing or physical allocation.

#### Scenario: Restart after observation persistence
r[lattice.agent_orchestration.usage.scenario.recovery]
- GIVEN the exact run receipt and its conservative charge were persisted before the settlement event
- WHEN the host restarts or recovery is repeated
- THEN recovery MUST reuse that exact workflow/instance/dispatch/run/reservation-bound receipt once, preserve previously consumed budget, and MUST NOT rederive a cheaper charge from missing or changed telemetry
- AND later exact owned outcome evidence MAY monotonically refine absent cancellation telemetry without changing any frozen usage, reservation or charge; stale partial writes MUST NOT erase known evidence, and contradictory known outcomes MUST be rejected

#### Scenario: Missing or conflicting telemetry
r[lattice.agent_orchestration.usage.scenario.uncertainty]
- GIVEN telemetry is missing, malformed, truncated, unsupported, interrupted or incomplete
- WHEN a terminal owned run is settled
- THEN unknown usage MUST retain the admitted conservative fallback rather than zero; valid known reports MUST remain distinct, and conflicting or crossed durable receipts MUST block settlement without refunding the reservation
- AND production Agent metadata split into ordered history rows MUST be reassembled as one exact port-bound JSON document within a byte bound, without truncation; valid totals with only one non-contradictory component MUST remain reports, while invalid fields or conflicting full sums MUST NOT establish token usage

#### Scenario: Cancellation outcome and settlement remain correlated
r[lattice.agent_orchestration.usage.scenario.cancellation]
- GIVEN a current completion has exact owned run and cancellation observations
- WHEN its budget reservation is settled
- THEN observed cancellation MUST close ownership only after the charge is durable, successful non-cancelled completion MUST win a late cancellation request, and failed/requested work MUST NOT become retry evidence
- AND absent cancellation telemetry MUST retain pending ownership and conservative budget without inventing a negative observation or replacement
- AND recovery that observes the terminal run record before the producer returns its typed outcome MUST remain pending until that later exact owned evidence is durable, then settle once without changing its frozen charge

#### Scenario: Unqualified shell routes cannot evade worker accounting
r[lattice.agent_orchestration.usage.scenario.shell]
- GIVEN worker invocation, token or monetary accounting is enforced and no accounted shell-worker profile is admitted
- WHEN a workflow contains any Shell node, including an opaque wrapper
- THEN admission MUST reject that route before runner effects rather than infer model usage from a command name or zero Agent nodes

#### Scenario: Worker and service qualification is unavailable
r[lattice.agent_orchestration.usage.scenario.bindings]
- GIVEN no admitted exact worker/service/request or currency/pricing identity is supplied by its existing owner
- WHEN usage is persisted or exposed
- THEN Lattice MUST retain explicit unknown coverage and MUST NOT invent GPU leases, served-weight identities, authoritative billing, shared-allocation apportionment or completed worker qualification

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

#### Scenario: Duplicate admission survives process restart
r[lattice.agent_orchestration.triggers.scenario.process-restart]
- GIVEN one process durably accepted an orchestration trigger fact and released its store ownership
- WHEN a separately started process reopens that store and submits the exact same definition revision, trigger identity, and idempotency key
- THEN Lattice MUST return the original instance and MUST NOT create another admission, instance, run, or budget charge

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

### Requirement: Atomic owner-bound run identity
r[lattice.agent_orchestration.run_identity] Lattice MUST reserve globally unique run identities atomically before workflow effects, MUST reject duplicate run-record writes, and MUST reconcile a pending orchestration dispatch only with a run record bound to the exact instance and dispatch identities.

#### Scenario: Concurrent allocators reserve distinct identities
r[lattice.agent_orchestration.run_identity.scenario.concurrent]
- GIVEN multiple allocators or restarted executors reserve runs before any of them persists completion
- WHEN they reserve run identities against the same store
- THEN each allocator MUST receive a distinct identity and a later allocator MUST NOT infer availability only from completed records

#### Scenario: Duplicate record cannot overwrite history
r[lattice.agent_orchestration.run_identity.scenario.duplicate]
- GIVEN a run record already exists for a reserved numeric identity
- WHEN another execution attempts to persist the same identity
- THEN Lattice MUST reject the second write and MUST preserve the first record unchanged

#### Scenario: Exact owner record reconciles
r[lattice.agent_orchestration.run_identity.scenario.match]
- GIVEN a persisted run record names the pending orchestration instance and dispatch identities
- WHEN recovery evaluates that record
- THEN Lattice MUST reconcile the owned result through the ordinary run-finished transition

#### Scenario: Foreign or legacy record fails closed
r[lattice.agent_orchestration.run_identity.scenario.foreign]
- GIVEN a persisted record has the same numeric run identity but is unscoped, legacy, or bound to another instance or dispatch
- WHEN recovery or abandonment evaluates the pending dispatch
- THEN Lattice MUST reject identity correlation without settling, clearing, retrying, or overwriting either execution

### Requirement: Explicit abandonment of locally unobserved dispatches
r[lattice.agent_orchestration.unobserved_abandon] Lattice MUST allow an authorized operator to abandon a cancellation-requested pending dispatch only when no matching persisted run record is observable, MUST terminate the local orchestration as failed without retrying or claiming a remote outcome, and MUST preserve the abandonment as a replayable event.

#### Scenario: Acknowledged dispatch is abandoned conservatively
r[lattice.agent_orchestration.unobserved_abandon.scenario.acknowledged]
- GIVEN recovery reports that an acknowledged pending dispatch has no matching persisted run record and an authorized operator has requested cancellation
- WHEN that operator abandons the pending dispatch
- THEN Lattice MUST consume the complete reserved upper bound, clear the pending and active run identities, terminate the instance as failed, and record that the remote outcome remains unknown

#### Scenario: Planned dispatch is abandoned before shell invocation
r[lattice.agent_orchestration.unobserved_abandon.scenario.planned]
- GIVEN a pending dispatch remains planned, has no matching persisted run record, and an authorized operator has requested cancellation
- WHEN that operator abandons the pending dispatch
- THEN Lattice MUST release the reservation without consumption, terminate the instance as failed, and MUST NOT invoke the workflow shell

#### Scenario: Observed run must use ordinary recovery
r[lattice.agent_orchestration.unobserved_abandon.scenario.observed]
- GIVEN the store contains a run record matching the pending dispatch
- WHEN an operator requests abandonment
- THEN Lattice MUST reject abandonment without mutation and direct the operator to ordinary recovery

#### Scenario: Abandonment requires cancellation and authority
r[lattice.agent_orchestration.unobserved_abandon.scenario.authority]
- GIVEN a pending dispatch is locally unobserved but cancellation was not requested or the actor is not an authorized approver
- WHEN abandonment is requested
- THEN Lattice MUST reject the request without changing the dispatch, reservation, budget, or event chain

#### Scenario: Repeated or mismatched abandonment fails closed
r[lattice.agent_orchestration.unobserved_abandon.scenario.identity]
- GIVEN an abandonment fact is repeated, names a different dispatch or run, or targets a terminal instance
- WHEN the state machine evaluates it
- THEN Lattice MUST reject it without a second settlement or event

#### Scenario: Replay preserves bounded non-claims
r[lattice.agent_orchestration.unobserved_abandon.scenario.replay]
- GIVEN an unobserved dispatch was abandoned
- WHEN Lattice replays the instance event chain
- THEN replay MUST reconstruct the same terminal state, conservative budget settlement, and fixed safe summary without asserting provider cancellation, provider failure, workflow success, billing truth, or remote exactly-once execution

### Requirement: Bounded observed-failure retry policy
r[lattice.agent_orchestration.observed_failure_retry] Lattice MUST admit retries only under an immutable positive retry bound and exact locally persisted terminal-failure evidence.

#### Scenario: Exact owned failure becomes retryable
r[lattice.agent_orchestration.observed_failure_retry.scenario.observed]
- GIVEN a retry-enabled orchestration round terminalizes with an exact instance-and-dispatch-owned run record whose failure count is nonzero
- WHEN Lattice commits the failed run observation
- THEN it MUST retain only bounded run and dispatch retry evidence and MUST NOT retain output, error, prompt, credential, reasoning, or provider content as retry authority

#### Scenario: Retry policy is explicit and bounded
r[lattice.agent_orchestration.observed_failure_retry.scenario.policy]
- GIVEN an orchestration definition enables retries
- WHEN Lattice validates and hashes the definition
- THEN the policy MUST declare a positive maximum observed-failure retry count and the immutable definition revision MUST bind that count

#### Scenario: Default-disabled compatibility remains stable
r[lattice.agent_orchestration.observed_failure_retry.scenario.compatibility]
- GIVEN a schema-v1 definition or event omits retry fields
- WHEN Lattice deserializes, reserializes, hashes, or replays it
- THEN retries MUST remain disabled and omitted default fields MUST NOT change its canonical identity or event state

### Requirement: Atomic authorized retry round
r[lattice.agent_orchestration.retry_round] Lattice MUST start an authorized retry as one durable transition that reserves fresh budget and installs a fresh run and dispatch identity before invoking the workflow shell.

#### Scenario: Authorized retry starts a fresh attempt
r[lattice.agent_orchestration.retry_round.scenario.success]
- GIVEN an exact observed terminal failure remains below its retry and multidimensional budget ceilings
- WHEN an authorized operator invokes `orchestration retry`
- THEN Lattice MUST atomically increment the retry and round counts, reserve fresh budget, install distinct run and BLAKE3 dispatch identities, execute the immutable workflow revision, settle the attempt, and preserve valid replay

#### Scenario: Policy, authority, and budget refuse before dispatch
r[lattice.agent_orchestration.retry_round.scenario.refusal]
- GIVEN retry is disabled, exhausted, unauthorized, not backed by exact observed failure evidence, or cannot fit fresh budget
- WHEN retry admission runs
- THEN Lattice MUST reject before invoking the workflow shell or creating a new dispatch, run record, or budget reservation

#### Scenario: Pending-unobserved work is never retried
r[lattice.agent_orchestration.retry_round.scenario.pending-unobserved]
- GIVEN an orchestration retains a pending dispatch with no exact owned persisted run record
- WHEN an operator invokes `orchestration retry`
- THEN Lattice MUST reject without mutation or redispatch and direct the operator to recovery or explicit authorized abandonment

#### Scenario: Abandoned and foreign outcomes are never retried
r[lattice.agent_orchestration.retry_round.scenario.unsafe-evidence]
- GIVEN a failed state arose from abandonment, a shell failure lacks an exact terminal run record, or a numeric run record is foreign or legacy-unscoped
- WHEN retry admission runs
- THEN Lattice MUST reject without allocating authority from that evidence

#### Scenario: Retry claims remain bounded
r[lattice.agent_orchestration.retry_round.scenario.non-claim]
- GIVEN a bounded retry reaches a local terminal result
- WHEN Lattice renders status, replay, or inspection evidence
- THEN it MUST NOT claim provider cancellation, arbitrary kill safety, provider truth, actual token or billing measurement, remote effect deduplication, remote exactly-once execution, or production readiness

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

### Requirement: Bounded build-loop reference runtime
r[lattice.agent_orchestration.bounded_build_loop] The local planner/generator/evaluator example MUST distinguish finite graph construction from runtime repetition, execute a real Rust build/test evaluator through the production executor, retain the specification/source/verdict handoffs and exact revision/recipe/result references for every produced round, and use the existing durable orchestration transitions and one total budget. Generated text MUST NOT grant authority or mutate an in-flight graph. Optional masked repair MUST remain disabled without a qualified producer and adapter.

#### Scenario: Real evaluator pass stops repetition
r[lattice.agent_orchestration.bounded_build_loop.scenario.pass]
- GIVEN the fixed local producer emits the required greeting implementation
- WHEN real Rust compilation and unit tests pass
- THEN the durable orchestration MUST complete after one round without another dispatch

#### Scenario: Failed check is retained before runtime retry
r[lattice.agent_orchestration.bounded_build_loop.scenario.retry]
- GIVEN the first candidate compiles but fails its actual unit test
- WHEN an exact owned terminal failure enables the one authorized retry
- THEN the previous source/verdict MUST remain attributable to its original revision and the next round MUST consume the same total budget

#### Scenario: Iteration exhaustion is durable
r[lattice.agent_orchestration.bounded_build_loop.scenario.exhausted]
- GIVEN both allowed candidates fail their actual test
- WHEN retry preflight proves the iteration cap exhausted
- THEN the orchestration MUST persist Exhausted and MUST dispatch no third round

#### Scenario: Resource exhaustion does not reset accounting
r[lattice.agent_orchestration.bounded_build_loop.scenario.resource]
- GIVEN the total Agent budget permits only one round
- WHEN its actual test fails and retry preflight rejects the resource charge
- THEN the loop MUST persist Exhausted without resetting consumed usage or dispatching another round

#### Scenario: Candidate revision and runtime admission precede dispatch
r[lattice.agent_orchestration.bounded_build_loop.scenario.admission]
- GIVEN source is regenerated or an undeclared runtime is proposed during an active round
- WHEN the host considers another round
- THEN it MUST preserve the running snapshot and reject the unapproved revision before dispatch while retaining earlier source/verdict receipts under the original revision

#### Scenario: Role and child references cannot grant effects
r[lattice.agent_orchestration.bounded_build_loop.scenario.authority]
- GIVEN a role has no matching current effect grant, or a saved child requests ungranted effects, or generated output requests an additional effect or self-approval
- WHEN admission or dispatch is attempted
- THEN the missing grant MUST prevent that adapter's effects and generated output MUST NOT widen authority

#### Scenario: Active owned cancellation prevents later rounds
r[lattice.agent_orchestration.bounded_build_loop.scenario.cancel]
- GIVEN the real evaluator process has started
- WHEN the host requests cancellation
- THEN it MUST forward the request to the owned executor, retain the observed termination separately from intent, settle the admitted reservation and dispatch no later round

#### Scenario: Actual completion wins late cancellation
r[lattice.agent_orchestration.bounded_build_loop.scenario.cancel-winner]
- GIVEN a real evaluator has already completed successfully or unsuccessfully without observed cancellation
- WHEN an operator cancellation request is durably recorded before completion settlement
- THEN `RunFinishedObserved` MUST preserve the actual completion, consume the reservation exactly once, prevent retry after failed completion plus cancellation intent, and replay the same state without rerunning effects
