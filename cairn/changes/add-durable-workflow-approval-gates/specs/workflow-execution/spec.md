# Workflow Execution

## ADDED Requirements

### Requirement: Workflows can declare typed approval nodes

r[lattice.workflow_approval.node] Lattice MUST model a workflow approval node with exact request, workflow revision, selected input, artifact, authority-policy, expiry-policy, resume-schema, credential-binding, terminal-policy, and non-claim facts.

#### Scenario: Valid approval node is admitted

r[lattice.workflow_approval.node.valid]
- GIVEN an approval node has one stable gate ID, safe summary, exact workflow and node facts, bounded policies, declared resume inputs, allowed credential bindings, and required non-claims
- WHEN workflow decoding and validation run
- THEN Lattice MUST produce one deterministic approval-request plan and domain-separated BLAKE3 identity
- AND node metadata alone MUST grant no decision, credential, resume, or downstream execution authority.

#### Scenario: Approval configuration is unsafe

r[lattice.workflow_approval.node.invalid]
- GIVEN an approval node has duplicate identity, unknown fields, unbounded text, missing policy, plaintext credential material, broad authority, ambiguous terminal behavior, or missing non-claims
- WHEN workflow validation runs
- THEN Lattice MUST reject the workflow before persistence, credential resolution, suspension, decision, or downstream effects
- AND it MUST NOT infer safe defaults for the missing authority or terminal policy.

### Requirement: Reaching an approval node durably suspends the run

r[lattice.workflow_approval.suspension] Lattice MUST persist exact approval-request and workflow state before reporting suspension, MUST end the current invocation without parking a process, and MUST execute no dependent node.

#### Scenario: Gate state survives process exit

r[lattice.workflow_approval.suspension.persisted]
- GIVEN an admitted workflow reaches an approval node after zero or more nodes complete
- WHEN Lattice applies the exact Choregraph suspension transition and commits the request, run, graph, event, input, artifact, authority, and expiry facts
- THEN the command MUST report a distinct suspended outcome and can exit
- AND a fresh process MUST inspect the same suspended request without rerunning completed nodes.

#### Scenario: Suspension persistence is incomplete

r[lattice.workflow_approval.suspension.uncertain]
- GIVEN the process stops before or during request reservation, Choregraph event application, state commit, or response output
- WHEN recovery opens the store
- THEN Lattice MUST reconstruct one exact committed state or report an explicit incomplete or uncertain boundary
- AND it MUST NOT invent suspension, approval, failure, cancellation, resume authority, or downstream completion.

### Requirement: Approval decisions and workflow resume are separate

r[lattice.workflow_approval.decision] Lattice MUST record approve or reject decisions separately from workflow resume, and decision commands MUST perform no downstream workflow execution.

#### Scenario: Approval is recorded without resume

r[lattice.workflow_approval.decision.approve]
- GIVEN a current suspended request and admitted decision authority
- WHEN the operator records approval
- THEN Lattice MUST persist one exact decision bound to the request, workflow revision, policy, actor, ability, expiry facts, and event history
- AND the node MUST remain non-executing until a separate resume command succeeds.

#### Scenario: Rejection is recorded without downstream work

r[lattice.workflow_approval.decision.reject]
- GIVEN a current suspended request and admitted rejection authority
- WHEN the operator records rejection
- THEN Lattice MUST persist the rejection and apply only the declared terminal policy
- AND no dependent node, credential provider, operation runtime, or fallback workflow MAY execute.

### Requirement: Decision authority is exact and current

r[lattice.workflow_approval.authority] Lattice MUST validate approval authority against the exact request, workflow revision, node, ability, actor, authority-policy identity, verification references, replay facts, caveat facts, and current expiry facts before accepting a decision.

#### Scenario: Local or verified delegated authority matches

r[lattice.workflow_approval.authority.valid]
- GIVEN the configured local profile admits the exact actor or already verified Basalt and UCAN facts allow the exact request and ability under current policy
- WHEN pure decision admission runs
- THEN Lattice MAY accept the bounded decision and persist its safe authority references
- AND the decision MUST grant no authority for another request, workflow, node, ability, actor, policy, or time window.

#### Scenario: Authority evidence is absent or stale

r[lattice.workflow_approval.authority.denied]
- GIVEN authority uses only raw token possession, actor text, process identity, environment values, wrong request or ability, stale policy or receipt, denied caveat, denied replay, expired facts, or prohibited self-approval
- WHEN decision admission runs
- THEN Lattice MUST deny before decision persistence or downstream effects
- AND it MUST NOT select another actor, authority profile, verifier, policy, request, or time window.

### Requirement: Resume reconstructs exact remaining workflow work

r[lattice.workflow_approval.resume] Lattice MUST resume only from current persisted suspension and accepted decision facts, apply one matching Choregraph resume signal, preserve prior terminal node states, and dispatch only newly ready work.

#### Scenario: Approved workflow resumes

r[lattice.workflow_approval.resume.valid]
- GIVEN a suspended run has a current accepted decision, matching workflow and artifact identities, valid authority policy, unexpired resume status, intact event history, and no active conflicting owner
- WHEN a separate resume command runs
- THEN Lattice MUST apply the matching Choregraph signal and execute only nodes that become ready after the gate
- AND completed nodes MUST remain completed without another local or external effect attempt.

#### Scenario: Resume supplies declared references

r[lattice.workflow_approval.resume.inputs]
- GIVEN the approval node declares typed non-secret resume values and allowed credential binding names
- WHEN resume supplies matching values and opaque `SecretRef` identities
- THEN Lattice MAY bind them to downstream preparation and audit their safe keys and reference identities
- AND it MUST NOT accept, persist, hash, log, or render resolved plaintext secret values.

#### Scenario: Resume state is stale or contradictory

r[lattice.workflow_approval.resume.denied]
- GIVEN the workflow, request, selected inputs, artifacts, decision, authority policy, expiry, Choregraph profile, event chain, store owner, or prior node state differs from the persisted checkpoint
- WHEN resume admission runs
- THEN Lattice MUST deny before applying a resume signal or dispatching work
- AND it MUST not reset, repair, refresh, retry, or select fallback facts automatically.

### Requirement: Rejection, expiry, duplicates, and terminal states fail safely

r[lattice.workflow_approval.terminal] Lattice MUST handle rejected, expired, duplicate, contradictory, stale, cancelled, failed, abandoned, and already terminal approval requests through deterministic idempotence or denial without rerunning prior effects.

#### Scenario: Approval or resume expires

r[lattice.workflow_approval.terminal.expired]
- GIVEN request decision time or resume time exceeds the exact configured freshness window
- WHEN decision or resume admission evaluates injected current clock facts
- THEN Lattice MUST record or report the expired outcome according to policy and block downstream work
- AND it MUST NOT extend the window, replace the clock source, or reuse an expired authority receipt automatically.

#### Scenario: Duplicate or contradictory mutation arrives

r[lattice.workflow_approval.terminal.duplicate]
- GIVEN a decision or resume command repeats the exact accepted facts or conflicts with a recorded decision, revision, event, or terminal outcome
- WHEN serialized mutation admission runs
- THEN an exact repeat MUST be idempotent only when the observed outcome already matches and a conflict MUST fail closed
- AND the store MUST retain one coherent history without duplicate downstream dispatch.

### Requirement: Existing orchestration approvals retain their semantics

r[lattice.workflow_approval.compatibility] Lattice MUST preserve existing agent-orchestration revision-proposal approval records, commands, replay, JSON, and authority behavior while adding workflow approval gates.

#### Scenario: Existing orchestration ledger replays

r[lattice.workflow_approval.compatibility.valid]
- GIVEN a supported orchestration ledger created before workflow approval gates
- WHEN compatibility loading and replay run after the change
- THEN Lattice MUST reconstruct the same orchestration state and external report shape without workflow-gate reinterpretation
- AND no revision-proposal approval MAY resume an ordinary workflow gate unless a separate exact product decision exists.

#### Scenario: Shared helper conflates approval classes

r[lattice.workflow_approval.compatibility.denied]
- GIVEN an adapter maps a workflow-gate decision to a revision proposal or treats a revision-proposal approval as gate authority
- WHEN compatibility and authority validation run
- THEN Lattice MUST reject the conflated mapping
- AND common identity helpers MUST NOT erase distinct transition, input, artifact, actor, or authority facts.
