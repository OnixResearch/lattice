## Context

Lattice already persists workflow runs and durable agent orchestration. Agent orchestration can pause, resume, cancel, abandon, and bind revision-proposal approvals to workflow revisions and policy hashes. These approvals govern successor workflow revisions, not arbitrary gate nodes inside an ordinary workflow.

Choregraph owns graph lifecycle, resume, replay, and transition semantics. The prerequisite Choregraph change adds an opt-in suspended local state and exact resume-signal event without product authority.

The Lattice layer must supply the missing product semantics: request identity, authority, expiry, actor, decision, persistence, resume values, credential references, CLI/TUI actions, and safe evidence.

## Success Contract

Completion requires a typed approval node, exact Choregraph suspension integration, durable local persistence, separate decide and resume commands, current authority checks, restart-safe idempotence, remaining-node execution only, and explicit negative behavior.

A live waiting process, combined approve-and-run command, mutable approval pointer, or orchestration-only adapter is false completion.

## Decisions

### Decision: define a product-owned approval node

**Choice:** Add an approval node with typed configuration for:

- gate ID and safe summary;
- exact workflow revision and node identity;
- selected input and required artifact identities;
- authority policy identity and accepted decision classes;
- expiry policy and clock-source class;
- optional reason policy;
- declared resume input schema;
- allowed credential binding names;
- rejection and expiration terminal policy;
- required non-claims.

The canonical request identity uses domain-separated BLAKE3 over stable typed facts. Prompts, display labels, actor-friendly names, timestamps, and free-form reasons remain outside request authority unless an explicit schema says otherwise.

**Rationale:** A decision is meaningful only for the exact work and evidence that the operator reviewed.

### Decision: persist suspension and end the invocation

**Choice:** When execution reaches an admitted approval node, Lattice reserves one request identity and persists:

- the exact workflow and run identity;
- current Choregraph state and event-chain head;
- completed, terminal, pending, and suspended node facts;
- selected input and artifact identities;
- authority and expiry policy identities;
- request status and safe non-claims.

Lattice applies the Choregraph suspend event and commits the resulting state before it reports suspension. The initiating headless command returns a distinct suspended outcome. The TUI returns to an operator-controlled state. No process remains parked and no dependent node runs.

**Rationale:** Long-lived approval waits must survive process exit and host restart.

### Decision: separate decision from execution

**Choice:** `approval decide` records approve or reject facts only. It validates the current request, expected revision, authority policy, actor evidence, decision class, optional bounded reason, and expiry status under serialized store ownership.

It does not apply the Choregraph resume signal, resolve credentials, reserve downstream runtimes, or execute a workflow node.

`workflow resume` is a separate command and transaction. The TUI exposes separate actions with the same boundary.

**Rationale:** Review authority and execution authority can belong to different operators and times.

### Decision: authority is exact and externally bounded

**Choice:** The first profile supports an explicit local operator authority policy. An optional Basalt/UCAN profile consumes already-verified grant and policy references through the existing Basalt authority core.

Lattice binds the decision to request ID, workflow revision, node, ability, actor, authority-policy hash, verification receipt refs, and current expiry facts. Raw compact tokens, key possession, actor text, environment variables, and process identity alone never grant approval.

The shell owns token parsing, signature verification, key resolution, clocks, revocation, replay stores, and policy loading. The pure core consumes validated facts only.

**Rationale:** Basalt and UCAN provide bounded authority decisions, not ambient human identity or storage truth.

### Decision: resume reconstructs exact remaining work

**Choice:** Resume reloads the persisted request and run, revalidates store ownership, workflow revision, input and artifact identities, decision, authority policy, expiry, Choregraph profile, and event-chain continuity.

It applies one matching Choregraph resume-signal event. Completed nodes remain completed. Failed, cancelled, or abandoned nodes do not become pending. Only newly ready downstream nodes can dispatch through ordinary Lattice admission.

Resume input is limited to declared typed non-secret values and opaque credential references. It never accepts raw secret values. Supplied keys and reference identities enter an audit record, while resolved credential values remain execution-only.

**Rationale:** Resume is reconstruction from a checkpoint, not continuation of a live thread.

### Decision: rejection, expiry, and duplicates are explicit

**Choice:** A rejection records a terminal gate decision and applies the configured workflow failure or cancellation policy without running dependents.

Expiry is evaluated from an injected current clock fact against the persisted policy. An expired request cannot accept a new decision. Already recorded approval can also expire before resume when policy requires resume freshness.

Equivalent repeated decision and resume requests are idempotent only for the same facts and already observed outcome. Contradictory decisions, changed reasons when reason is identity-bound, stale revisions, and another active store owner fail closed.

**Rationale:** Approval value can decay, and retries must not create duplicate effects or conflicting history.

### Decision: preserve existing orchestration approvals

**Choice:** The workflow gate uses common typed identity and authority helpers where possible, but it does not reinterpret `RevisionProposal` approvals or change their public record shape implicitly.

A later migration can share additional cores only after parity tests prove that workflow-gate and revision-proposal semantics remain distinct.

**Rationale:** Approval is a broad word for different product transitions. Accidental unification can authorize the wrong action.

### Decision: require exact Choregraph evidence

**Choice:** Lattice pins a reviewed Choregraph revision with the durable-suspension profile, canonical fixture identities, production-linked proof evidence, and downstream compatibility result.

Lattice maps product request identity into the opaque Choregraph suspension reference. It revalidates product authority before constructing the matching resume signal.

Passing Choregraph transition evidence does not satisfy approval authority or persistence correctness.

**Rationale:** Graph state semantics and product authority remain separate but linked.

## Command Shape

The final command spelling can evolve during implementation, but the semantic split remains:

```text
lattice approvals list
lattice approvals show <request-id>
lattice approvals decide <request-id> --approve|--reject --actor <actor>
lattice workflow resume <run-id> [typed non-secret values and credential references]
```

All read commands omit private workflow content by default. Mutation commands require explicit request and expected-revision identities.

## Validation Strategy

Positive cases cover suspension persistence, process exit, list/show, local approval, verified Basalt approval, separate resume, exact remaining-node execution, typed resume values, credential references, rejection, expiry, and idempotent repeated reads.

Negative cases cover self-approval when denied, raw token, stale receipt, wrong ability, wrong request, changed workflow, changed artifacts, expired decision, contradictory decision, duplicate resume, secret value input, missing Choregraph profile, corrupted event chain, active store owner, crash before and after each commit boundary, and downstream executor uncertainty.

The operator E2E harness must kill the initiating process after suspension persistence and prove that a fresh process can inspect and resume without rerunning completed local effects.

## Adversarial Review

The final audit must try to show that:

- approval dispatches downstream work;
- resume accepts a decision for another workflow revision;
- completed nodes rerun after restart;
- a raw token or actor string grants authority;
- expiry can be bypassed by delayed resume;
- a plaintext secret reaches persisted resume input;
- a Choregraph reference match is described as human approval truth;
- duplicate or conflicting decisions create two histories.

Any successful counterexample blocks closeout.

## Risks and Trade-offs

- Durable gates add long-lived run state and operator cleanup needs. List, reject, expire, and inspect commands make this visible.
- Local authority is useful for one operator but does not prove independent review. Receipts identify the selected profile and non-claims.
- Basalt/UCAN integration depends on current verified facts from external shells. Stale or unavailable verification blocks approval.
- Resume after uncertain external effects remains dangerous. Completed or unknown prior nodes cannot be rerun automatically.

## Non-Claims

This design does not prove human identity, independent review, UCAN or Basalt correctness, key or revocation freshness, persistence under arbitrary host loss, notification delivery, downstream operation correctness, exactly-once external effects, production readiness, or release eligibility.
