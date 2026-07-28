## Why

Lattice has hash-bound approvals for agent-orchestration revision proposals. Ordinary workflows do not have a generic node that can persist a durable wait, let the process exit, record one independent decision, and resume only the remaining graph.

Swamp demonstrates a useful operator contract for manual gates: suspension is persisted, approval does not execute downstream work, and a later resume reconstructs the run. Lattice will adapt that behavior with stronger BLAKE3 revision binding, typed authority evidence, explicit expiration, opaque credential references, Choregraph suspension transitions, and unknown-outcome handling.

## What Changes

- Add a typed workflow approval node with exact request, workflow revision, input, artifact, authority-policy, expiry-policy, resume-schema, and non-claim identities. r[lattice.workflow_approval.node]
- Persist one suspended run and stop the current invocation without parking a process or executing downstream nodes. r[lattice.workflow_approval.suspension]
- Separate approval or rejection from resume. A decision records bounded facts and performs no downstream execution. r[lattice.workflow_approval.decision]
- Validate local or Basalt/UCAN-backed decision authority against the exact request and current policy without treating raw tokens or actor text as authority. r[lattice.workflow_approval.authority]
- Resume from persisted state through the exact Choregraph suspension profile, skip completed nodes, and accept only declared non-secret values or opaque credential references. r[lattice.workflow_approval.resume]
- Deny stale, expired, rejected, duplicate, contradictory, changed-revision, and terminal decisions without rerunning prior effects or inventing completion. r[lattice.workflow_approval.terminal]
- Preserve existing orchestration proposal approvals as a separate compatible product surface. r[lattice.workflow_approval.compatibility]

## Success Criteria

- Reaching a gate persists suspended state before the initiating process exits.
- Approval and rejection commands never run workflow nodes.
- Resume requires an accepted current decision bound to the same gate request, workflow revision, selected inputs, artifacts, and authority policy.
- Completed nodes do not rerun after resume.
- Rejected or expired gates terminate according to explicit policy and never execute dependents.
- Resume input cannot carry plaintext secret material. Credential needs use existing `SecretRef` values only.
- Restart, duplicate command, stale revision, concurrent owner, and crash-boundary tests preserve one deterministic outcome or explicit uncertainty.
- The Lattice consumer pins current Choregraph suspension evidence before enabling the node.

## False Completion

A process blocked on stdin is not durable suspension. An approval command that also resumes is not the required separation.

Reusing orchestration revision approval without a workflow-node contract, persisted graph state, or exact remaining-node resume is not completion.

A matching actor string, raw UCAN token, or stored approval reason does not establish authority.

## Impact

- **Lattice core**: pure request validation, decision validation, authority-input binding, resume admission, terminal policy, idempotence, and safe receipt projection.
- **Lattice shell**: workflow node configuration, persisted suspension and decisions, CLI and TUI actions, Choregraph event application, optional Basalt/UCAN adapter, and restart recovery.
- **Choregraph**: consumed at an exact revision with durable-suspension profile, matching resume signal, replay, and proof evidence. Product authority remains in Lattice.
- **Credentials**: existing provider and `SecretRef` boundaries remain authoritative. Resume does not introduce another secret store.
- **Testing**: positive and negative unit, integration, process-restart, CLI, TUI, redaction, authority, replay, and operator crash-boundary evidence.

## Out of Scope

- Remote approval services, email or chat notifications, web UI, quorum approval, delegation UI, or distributed multi-writer consensus.
- Secret values supplied directly during resume, automatic retries, automatic resume, or approval inferred from model output.
- Claims for human identity truth, UCAN verifier correctness, revocation freshness, persistence under arbitrary host loss, downstream effect safety, production readiness, or release eligibility.

## Dependencies

- Local Cairn change: `type-orchestration-identities` for durable typed orchestration and approval identity compatibility.
- External evidence dependency: a published Choregraph revision that completes `model-durable-suspension-transitions`.

## Affected Specs

- `workflow-execution`: approval node, suspension, decisions, authority, resume, terminal behavior, and compatibility.
