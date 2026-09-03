# Design: Pilot linear process-permit flow

## Context

Lattice consumes Choregraph machines and executes admitted effect intents. Basalt evaluates supplied authorization facts. Bounded Exec owns bounded child-process mechanics.

Lattice already records durable attempts and explicit unknown outcomes. The new pilot binds those runtime facts to a static Choregraph resource-flow contract.

The permit is a symbolic single-use obligation. It is not the existing scheduler capacity permit and does not grant authority.

## Decisions

### Decision: Limit the pilot to local shell nodes

**Choice:** Enable the resource-flow profile only for one local shell-node path that uses Bounded Exec.

Command agents, credential providers, file effects, network effects, remote agents, and other process users remain outside this pilot.

**Rationale:** One bounded consumer can test the contract without a broad runtime migration.

### Decision: Pin the complete producer chain

**Choice:** Bind the exact Choregraph revision, handoff profile, machine, graph, transition, Trellis proof profile, and Kamacite artifact identities.

No runtime or validation path uses an ambient sibling checkout.

**Rationale:** Runtime admission must fail when any semantic or artifact producer changes.

### Decision: Keep generic row semantics out of Lattice

**Choice:** Lattice validates exact closed-row and transition identities from Choregraph. It does not run row unification, fork partition, or join admission.

**Rationale:** Choregraph owns graph admission. A second checker can drift and create split authority.

### Decision: Use a pure permit state machine

**Choice:** Add these symbolic states:

```text
Available
Owned { attempt_identity }
Settled { attempt_identity, observation_identity }
Unknown { attempt_identity, uncertainty_identity }
```

The pure core admits state transitions from supplied intent, authorization, persistence, and observation facts.

**Rationale:** Explicit states keep runtime ownership and uncertainty visible without host I/O in the core.

### Decision: Make the permit single-use

**Choice:** One permit occurrence can bind one durable attempt. `Settled` and `Unknown` are terminal for that occurrence.

A fresh retry requires a new permit occurrence and a new attempt identity.

**Rationale:** Returning to `Available` can hide duplicate external execution after uncertain outcomes.

### Decision: Keep authority separate

**Choice:** Lattice requires an accepted Choregraph handoff and an accepted Basalt decision before process preparation.

A valid permit never grants executable, path, credential, actor, or effect authority.

**Rationale:** Resource conservation and authorization answer different questions.

### Decision: Persist ownership before execution

**Choice:** The shell first validates intent and authority. It then commits the exact `Owned` attempt before Bounded Exec starts the child.

If storage returns an uncertain commit outcome, Lattice reconciles storage before any process execution.

**Rationale:** An external effect must not start without durable attempt ownership.

### Decision: Settle only from trustworthy observations

**Choice:** A completed Bounded Exec observation settles the permit only when attempt, intent, transition, process profile, and observation identities match.

Success and observed failure can both settle the permit. Truncated, stale, foreign, conflicting, or missing observations cannot settle it.

**Rationale:** Settlement means Lattice observed a terminal result for the owned attempt. It does not mean the command succeeded.

### Decision: Preserve unknown outcomes

**Choice:** Missing or conflicting terminal evidence changes `Owned` to `Unknown` with an exact uncertainty identity.

`Unknown` denies automatic retry, automatic permit restoration, and reuse of the original occurrence.

**Rationale:** Lattice must not invent process completion or absence.

### Decision: Require explicit recovery

**Choice:** Recovery supports these bounded actions:

- settle from a later exact terminal observation
- abandon the unknown occurrence without reuse
- start a fresh attempt with a new permit after explicit duplicate-risk acknowledgment

Each action records a new decision and evidence identity.

**Rationale:** Operator action can choose policy. It cannot rewrite the historical outcome.

### Decision: Keep Bounded Exec ownership unchanged

**Choice:** The shell continues to use the pinned Bounded Exec adapter for argv, environment, working directory, streams, deadline, cancellation, teardown, and child reaping.

The process-permit core receives only normalized observations and identity facts.

**Rationale:** Lattice must not duplicate bounded process mechanics.

### Decision: Keep legacy workflows unchanged

**Choice:** Existing workflows and effect paths keep current behavior and identities. Only an explicit profile and resource-aware Choregraph handoff enable the pilot.

**Rationale:** Existing runs must not gain retroactive linear-resource claims.

## Runtime Flow

```text
pinned Choregraph intent and transition
  -> pure handoff admission
  -> Basalt authorization decision
  -> durable Owned permit and attempt commit
  -> Bounded Exec process execution
  -> normalized terminal or uncertain observation
  -> pure Settled or Unknown decision
  -> durable state and bounded evidence
```

The pure core makes state and dispatch decisions. The imperative shell owns storage, authorization calls, process execution, observations, and operator commands.

## Port Boundaries

The application can reuse existing ports for:

- loading and committing attempt state
- obtaining authorization decisions
- executing a bounded local process
- recording observations and evidence

Add a new port only when no existing application-owned capability represents the required boundary.

## Identity and Persistence

Persisted pilot records bind:

- workflow and revision identity
- run and node identity
- Choregraph graph, machine, intent, and transition identities
- permit occurrence and state identity
- attempt identity
- Basalt decision identity
- Bounded Exec profile identity
- observation or uncertainty identity
- previous record identity
- deterministic state-transition identity

Records use existing Lattice BLAKE3 framing and atomic persistence patterns.

## Tests

Positive tests include:

- exact handoff admission
- authorization followed by durable ownership
- observed successful process settlement
- observed failed process settlement
- exact later observation that settles an unknown attempt
- explicit abandonment
- fresh retry with new permit and duplicate-risk acknowledgment
- restart and replay of each persisted state

Negative tests include:

- stale Choregraph profile or transition
- resource row or occurrence mismatch
- authorization denial with no process execution
- process start before durable ownership
- storage commit uncertainty followed by process execution
- duplicate attempt binding
- foreign or stale terminal observation
- truncated output presented as settlement
- unknown state followed by automatic retry
- unknown state followed by original permit reuse
- fresh retry without duplicate-risk acknowledgment
- legacy workflow gaining the profile implicitly

Shell tests assert port-call order and no protected effect on every denial path.

## Compatibility

The pilot is opt-in and limited to one local shell-node profile. Legacy persistence, commands, workflows, and effect paths remain unchanged.

A later expansion requires a separate Cairn change with evidence for each new effect class.

## Evidence and Claim Boundary

Lattice evidence can establish local intent binding, authorization ordering, durable ownership, bounded process observation, uncertainty preservation, and explicit recovery behavior.

It does not prove Choregraph correctness, Trellis soundness, Kamacite codec soundness, host sandboxing, process correctness, provider truth, external cancellation, exactly-once effects, or production readiness.

## Risks and Trade-offs

- A terminal `Unknown` occurrence can require operator work. This is safer than silent reuse.
- More identities increase persistence and diagnostic complexity. Exact linkage prevents stale producer use.
- A narrow pilot does not validate other effect classes. Later adoption needs separate evidence.
- Storage and process crash windows remain possible. Durable preparation and explicit uncertainty make them visible.
- Strict producer pins require coordinated upgrades. This protects semantic and evidence compatibility.
