## Context

Lattice already owns typed workflow values, runtime policy, credentials, events, artifacts, schedules, webhooks, replay, and operator surfaces. Seaglass owns reusable graph admission, deterministic order, readiness, lifecycle, critical-path, and bottleneck decisions for one finite DAG. The missing layer is not another graph executor. It is a durable Lattice-owned state machine that decides whether and why another immutable DAG run may start.

The design goal is a headless-first orchestration layer that can be tested without standing up runtimes and that never converts agent output into ambient authority.

## Architecture

```text
Typed Nickel OrchestrationDefinition
        |
        v
OrchestrationCore (pure)
  state + policy + explicit facts
        |
        v
OrchestrationDecision[]
  reserve / dispatch / wait / approve / pause / finish
        |
        v
OrchestratorShell
  store + clock facts + credentials + trigger adapters + event/outbox I/O
        |
        v
existing RunnerShell -> Seaglass execution graph -> registered node runtimes
        |
        v
round result + usage + proposal artifacts -> explicit facts -> core
```

`OrchestrationCore` owns no filesystem, database, process, network, clock, async runtime, model call, or UI access. The shell captures those observations as typed facts, persists accepted decisions, and invokes the existing runner. Seaglass remains below the orchestration boundary and does not decide objectives, approvals, budgets, credentials, triggers, or cross-run authority.

## Domain Model

### Orchestration definition

`OrchestrationDefinition` is immutable after activation and contains:

- stable definition id and BLAKE3 revision id;
- objective text plus typed success, failure, and stop predicates;
- initial `WorkflowRevisionRef` and declared workflow input/output bindings;
- budget policy and per-runtime charge-bound requirements;
- authority policy for start, pause, resume, cancel, proposal approval, and trigger admission;
- retry, overlap, and trigger policy;
- redaction and retention policy.

A `WorkflowRevisionRef` binds the canonical workflow snapshot, workflow schema identity, runtime-binding identities, declared effect policy, and content hash. A round never follows a mutable workflow name alone.

### Orchestration instance

`OrchestrationInstance` contains only durable state:

- instance id, definition revision, epoch, and current workflow revision;
- lifecycle state: `Ready`, `Running`, `WaitingForApproval`, `Paused`, `Completed`, `Failed`, `Cancelled`, or `Exhausted`;
- current round number and active run reference when present;
- budget reservations, settled usage, and remaining balances;
- pending revision proposal and approval references;
- last accepted event sequence and durable dispatch intent references.

Terminal states cannot transition. `Paused` prevents new dispatch but does not pretend an already-running external effect was stopped. Cancellation requests are recorded and forwarded to the existing runner; the final state reflects the observed runner outcome.

### Facts and decisions

The core consumes facts such as `StartRequested`, `TriggerAdmitted`, `DispatchAcknowledged`, `RunCompleted`, `RunFailed`, `UsageObserved`, `ProposalSubmitted`, `ApprovalGranted`, `ApprovalRejected`, `PauseRequested`, `ResumeRequested`, and `CancelRequested`.

It returns decisions such as `ReserveBudget`, `PersistDispatchIntent`, `StartRound`, `RecordProposal`, `RequestApproval`, `AdoptRevision`, `Pause`, `Resume`, `RequestRunCancellation`, `Complete`, `Fail`, `Exhaust`, or a stable rejection diagnostic. Decisions are data; only the shell performs effects.

## Round and Feedback Semantics

One orchestration round references one immutable workflow revision and produces one normal Lattice run lineage. The existing runner converts that workflow into its typed runner plan and delegates graph-generic decisions to Seaglass.

A workflow may produce an `OrchestrationProposal` artifact containing:

- base definition, instance epoch, and workflow revision hashes;
- proposed successor workflow revision or predeclared successor id;
- rationale and expected objective impact;
- requested input changes, runtime bindings, effects, credentials, triggers, and budget changes;
- BLAKE3 identity of the canonical proposal payload.

The proposal is inert. An ad hoc successor cannot execute until an approval receipt binds the instance, epoch, base revision, proposal hash, authority-policy hash, actor authority reference, and decision. Approval is necessary but never sufficient: the successor must still pass current workflow, graph, port, runtime, effect, credential, trigger, budget, and orchestration-policy validation before adoption. Stale approvals and approvals for a different proposal or base revision are rejected. A predeclared transition may advance automatically only when the active definition already authorizes that exact successor and does not rely on agent-supplied authority.

This creates a sequence of acyclic runs rather than one hidden cyclic graph:

```text
revision A --round 1--> result/proposal --approval--> revision B --round 2--> ...
```

## Budget Admission

Budgets are named policy values rather than code literals. Supported dimensions initially include round count, agent invocation count, token units, monetary microunits, and wall-duration units. Definitions choose which dimensions are enforced.

Before dispatch, the shell supplies a planned upper-bound charge derived from the admitted workflow/runtime plan. The pure core admits or rejects that charge, and the shell commits the resulting reservation with compare-and-swap or serializable transaction semantics against the instance version. An enforced dimension with no upper bound, an insufficient balance, a concurrent stale instance version, or a duplicate reservation blocks dispatch. After the round, observed usage settles the reservation and releases any remainder.

Provider usage is an observation, not trusted truth. If observed usage exceeds the reservation, Lattice records a budget-breach event, consumes the observed amount, and blocks new work; it cannot undo already-incurred external cost. If an enforced provider-metered dimension has no acceptable observation or estimate under policy, the instance waits or exhausts rather than silently treating usage as zero.

## Durable Dispatch and Recovery

The orchestration event append, budget reservation, and dispatch intent are persisted atomically. Each intent receives a BLAKE3 idempotency token derived from stable instance, epoch, event, revision, and action identities. The shell then invokes the runner with the stable run identity and records acknowledgement or failure.

On restart, the shell reconciles pending intents against stored run history before dispatching again. Duplicate trigger facts, completion facts, approvals, and acknowledgements are idempotent by stable identity. Completed rounds are never reopened.

This does not promise remote exactly-once effects. Adapters that can propagate idempotency keys SHOULD do so. Adapters without that facility remain explicitly at-least-once across an indeterminate crash boundary, and status/evidence must expose that risk.

## Operator and Trigger Boundaries

The first operator surface is headless:

```text
lattice orchestration validate <definition.ncl>
lattice orchestration start <definition.ncl>
lattice orchestration status <instance>
lattice orchestration pause|resume|cancel <instance>
lattice orchestration approve|reject <instance> <proposal>
lattice orchestration replay <instance>
```

TUI work reuses the same query/command ports and event model; it is not a second orchestration implementation.

Existing schedule and webhook adapters may create or signal instances only through explicit trigger bindings. Trigger idempotency keys bind to the orchestration definition, trigger, and requested action. Trigger payloads become typed inputs; they do not carry plaintext authority or silently select credentials.

## Format and Interchange

Nickel is the human-authored source for orchestration definitions and contracts. Import validates all stable references, budgets, predicates, transitions, authority references, trigger bindings, and workflow revision hashes before persistence. Secret values never appear in the definition.

Existing n8n JSON import continues to map supported workflow nodes into a Lattice workflow revision that may be referenced as a leaf round. Lattice does not infer objectives, approvals, budgets, or cross-run transitions from n8n JSON. Default n8n export fails when orchestration semantics are present unless a later change defines an exact mapping; best-effort export must label omitted semantics and cannot claim parity.

## Event Ledger, Replay, and Evidence

Every accepted fact and decision emits an ordered redaction-safe event carrying instance id, epoch, round, workflow revision, run reference, event sequence, prior-event hash, event hash, policy hash, and safe artifact references. Instance state stores the expected current event-head identity separately; optional independently controlled checkpoints may witness a prefix. Replay folds those immutable events through the pure core without invoking the runner or any adapter.

The terminal summary records objective-predicate outcomes, round lineage, budget reservations and settlements, approvals, proposal/revision lineage, dispatch recovery facts, artifact identities, and explicit non-claims. Passing replay proves only that the supplied event sequence folds consistently under the selected policy and content identities and matches the supplied local head or checkpoint. A local head does not prove external anchoring, independently witnessed completeness, or integrity against an attacker who can replace both the ledger and its local head.

## Portfolio Search Decision

Four mechanisms were considered:

| Family | Mechanism | Result |
|---|---|---|
| Single cyclic workflow | Put feedback edges inside one long-lived graph | Rejected: violates Seaglass DAG admission and hides run/effect boundaries. |
| Ambient autonomous controller | Let an agent mutate the active graph and immediately dispatch it | Rejected: grants implicit authority and makes budgets, approvals, and replay advisory. |
| Static meta-DAG | Precompute a DAG whose nodes are complete workflow runs | Useful for fixed pipelines, but insufficient for operator-reviewed dynamic revisions and stop decisions. |
| Event-sourced orchestration state machine | Fold explicit facts into bounded decisions over immutable DAG runs | Selected: preserves the existing runner, makes authority visible, and supports deterministic recovery. |

The independent review agreed that missing-agent defaults, reused completed nodes, and unconstrained graph rewrites are authority and hidden-cycle hazards. This design rejects missing authority, binds every round to a fresh durable run identity, and admits revisions only between terminal round boundaries.

## Risks / Trade-offs

- Atomic event/outbox persistence requires a store transaction boundary and migration; partial writes must fail closed.
- Provider token and cost reports can be absent or inaccurate; evidence records observations and policy treatment without claiming billing truth.
- Approval receipts can become stale after policy or revision changes; all relevant hashes and the instance epoch are mandatory bindings.
- Strict revision hashing makes edits explicit and may add operator friction, but prevents silent workflow-name drift.
- Headless-first delivery delays rich canvas controls while establishing one testable orchestration contract.
- Existing runtimes without idempotency support cannot provide exactly-once external effects; the product must expose, not conceal, that boundary.
