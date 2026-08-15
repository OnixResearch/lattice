## Context

Lattice already persists workflow runs and attempts before effects. It distinguishes observed terminal failures from unknown external outcomes. It also consumes Choregraph machine decisions while retaining product authority.

Projected role protocols add ordered peer interactions and branch synchronization. The runtime must not reinterpret protocol syntax or invent missing projection semantics.

## Success Contract

The accepted result is one durable runtime path for a frozen Choregraph role-protocol cohort. It admits exact artifacts, creates a session, advances role cursors through expected actions, records external uncertainty, and replays without effects.

The initial profile can run all roles under one Lattice owner. Distributed placement can remain blocked until OnixOS publishes an admitted binding.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Durable role-session coordinator | Persist exact cursors and attempts around bounded adapters | selected |
| Compile protocols into ordinary DAG nodes | Erase peer and choice semantics into workflow edges | rejected |
| Stateless message loop | Recompute expected state from arriving messages | rejected |
| Automatic resend | Retry every unacknowledged transfer | rejected |
| New transport stack | Implement sockets and routing inside the protocol core | rejected |

## Decisions

### Decision: consume frozen canonical protocol artifacts

**Choice:** Lattice consumes one exact Choregraph global artifact, ordered complete local set, projection receipt, schema cohort, and adapter identity.

The pure intake adapter checks canonical identities and maps each local artifact into Lattice-owned strict DTOs. Similar field names or JSON shape do not establish compatibility.

**Rationale:** Lattice must not become a second compiler or projection authority.

### Decision: require complete role placement before session creation

**Choice:** A session plan maps every declared role to exactly one role instance.

A native role instance selects one admitted Lattice runtime route. An external role instance selects one immutable external contract and adapter cohort.

Missing, duplicate, extra, mixed-kind, stale, or fallback placement fails before persistence.

**Rationale:** Partial placement can create a session that waits forever or contacts an unintended endpoint.

The first profile can use a local reviewed placement fixture. Later OnixOS output can supply the same strict binding shape.

### Decision: bind one exact session and step envelope

**Choice:** Every protocol envelope contains canonical identities for:

- protocol and complete local set;
- session and role instances;
- source and destination roles;
- source and destination local artifacts;
- expected protocol step;
- message and attempt;
- label and payload tag;
- optional owned value identity required by the step;
- canonical payload reference; and
- adapter cohort and schema version.

The envelope has domain-separated BLAKE3 identity. Display names, host paths, and raw credentials are excluded.

**Rationale:** Payload shape alone cannot prevent stale, cross-session, or wrong-step acceptance.

### Decision: separate pure decisions from runtime effects

**Choice:** Pure cores admit session plans, classify expected actions, validate envelopes, compute cursor transitions, classify outcomes, and plan persistence actions.

Shells own database transactions, transport, credentials, authorization facts, external adapters, clocks, cancellation, and presentation.

**Rationale:** Protocol decisions must be testable without network, process, or storage setup.

### Decision: persist before dispatch

**Choice:** Before an outbound transport effect, Lattice persists the session identity, source cursor, exact expected action, message identity, attempt identity, adapter route, authority-decision reference, and `Prepared` state.

After dispatch starts, the attempt becomes `DispatchedUnknown` until an admitted receiver observation or exact terminal failure settles it.

**Rationale:** A crash must not erase the possibility that an external effect occurred.

### Decision: commit semantic transfer only after exact receipt admission

**Choice:** The receiver advances only after it validates the exact expected envelope against its current local cursor.

The coordinator commits the semantic transfer and both linked cursor transitions only after the accepted receiver observation matches the prepared outbound attempt.

A crash or missing observation leaves the attempt uncertain. It does not advance ownership and does not authorize automatic resend.

**Rationale:** Trellis models an atomic semantic transfer. Lattice must expose the wider runtime uncertainty around that boundary.

This is local product state. It does not prove remote byte uniqueness or durable receiver storage.

### Decision: branch selection uses exact labels

**Choice:** A decider emits one exact choice envelope bound to the session, decider artifact, step, label, and attempt.

A role accepts that label only when its local cursor exposes the matching offer from that decider. Unknown, stale, duplicated, or reordered labels fail without cursor advancement.

**Rationale:** Boolean branch synchronization cannot represent a general labeled choice safely.

### Decision: replay never dispatches

**Choice:** Replay reads canonical session records and recomputes pure cursor and outcome decisions. It emits no transport, external-role, or other product effect.

Earlier authorization and adapter readiness records remain historical evidence only.

**Rationale:** Replaying a protocol must not repeat external communication.

### Decision: use existing authority and adapter boundaries

**Choice:** Before dispatch, Lattice revalidates the current protocol action through the accepted Choregraph intent authority boundary and the selected bounded adapter policy.

An external role also requires its exact contract, current adapter cohort, credentials by reference, destination policy, limits, and current authorization facts.

**Rationale:** Projectability and role kind are not execution authority.

### Decision: provide explicit recovery outcomes

**Choice:** Operator recovery can settle an exact terminal observation, abandon an unknown attempt, or request a new attempt only after explicit duplicate-risk acknowledgement and unchanged protocol artifacts.

A new attempt receives a new identity. It cannot reuse an earlier message or authorization identity.

**Rationale:** Hidden automatic retry can duplicate transfers or branch selections.

### Decision: keep diagnostics safe and structured

**Choice:** Inspection reports protocol, session, role, cursor, step, attempt, outcome, safe issue codes, counts, and bounded identities.

It omits payload bytes, credentials, commands, endpoint secrets, source text, and host paths.

**Rationale:** Operators need protocol state without another secret-bearing transcript.

## State Sketch

```text
Planned
  -> Prepared
  -> DispatchedUnknown
  -> ObservedAccepted
  -> Committed

Prepared or DispatchedUnknown
  -> ObservedRejected
  -> Failed

DispatchedUnknown
  -> AbandonedUnknown
```

Terminal records do not accept later mutation. A fresh retry creates a new attempt and preserves the earlier uncertain record.

## Validation Strategy

Positive fixtures cover local native roles, exact transfer, labeled choice, shared-prefix offer, chained ownership, external role admission, receiver acceptance, terminal failure, replay, and explicit recovery.

Negative fixtures cover stale protocol, incomplete placement, wrong role, wrong peer, wrong step, wrong local artifact, wrong label, wrong payload tag, wrong value, duplicate message, reordered message, replay dispatch, stale authorization, wrong external contract, missing credential reference, crash uncertainty, automatic retry, and unsafe diagnostics.

Crash tests cover before persistence, after prepared persistence, after dispatch start, after receiver acceptance, and before commit publication.

## Adversarial Audit

The audit must try to show that:

- a message advances two cursors twice;
- one session accepts another session envelope;
- a stale label selects a branch;
- a former owner sends an already committed value;
- replay dispatches a transport effect;
- unknown outcome becomes success or automatic retry;
- external role metadata grants authority; or
- an incomplete local artifact set starts a session.

Any successful counterexample blocks archive.

## Risks and Trade-offs

- Coordinated local commit does not provide distributed consensus.
- External transports can report ambiguous outcomes.
- Exact envelope identities increase storage and migration work.
- A frozen Choregraph adapter can drift as compiler schemas evolve.
- The first local profile gives behavior evidence only for one runtime topology.

## Non-Claims

This design does not prove transport confidentiality, endpoint identity truth, remote storage durability, external-role correctness, exactly-once delivery, global progress, deadlock freedom under faults, operating-system confinement, production readiness, or release eligibility.
