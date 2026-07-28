## Phase 1: Dependencies and baseline

- [ ] [depends:type-orchestration-identities] A1 Begin persisted approval implementation only after existing orchestration identity types and compatibility adapters are complete. r[lattice.workflow_approval.compatibility]
- [ ] [serial] A2 Record the exact published Choregraph suspension revision, profile identity, canonical fixtures, proof evidence, downstream result, and non-claims before enabling the node. r[lattice.workflow_approval.suspension]
- [ ] [serial] A3 Run and record current workflow run, orchestration approval, pause/resume, credential, redaction, store ownership, replay, operator E2E, and crash-boundary baselines. r[lattice.workflow_approval.node] r[lattice.workflow_approval.resume]
- [ ] [serial] A4 Inventory every approval-like DTO, identity, policy hash, CLI action, TUI action, persisted record, event, replay path, and authority adapter to prevent semantic conflation. r[lattice.workflow_approval.compatibility]

## Phase 2: Typed node and pure request core

- [ ] [serial] N1 Add typed Nickel and Rust approval-node configuration for gate, workflow, selected inputs, artifacts, authority, expiry, reason, resume schema, credential bindings, terminal policy, and non-claims. r[lattice.workflow_approval.node]
- [ ] [serial] N2 Add domain-separated BLAKE3 request, decision, authority-input, resume, event, and receipt identities with named field and collection bounds. r[lattice.workflow_approval.node] r[lattice.workflow_approval.decision]
- [ ] [serial] N3 Add pure request, decision, expiry, authority-fact, terminal-policy, and resume-admission validation over already loaded values. r[lattice.workflow_approval.authority] r[lattice.workflow_approval.terminal]
- [ ] [parallel] N4 Add positive and negative node and core fixtures for duplicate gates, unknown fields, unsafe schemas, self-approval policy, stale identities, contradictory policies, and missing non-claims. r[lattice.workflow_approval.node]

## Phase 3: Choregraph suspension and durable persistence

- [ ] [serial] S1 Add exact conversion from an admitted product request identity to the pinned Choregraph suspension profile and opaque reference. r[lattice.workflow_approval.suspension]
- [ ] [serial] S2 Persist request, run, graph state, event head, completed and pending node facts, selected inputs, artifacts, authority policy, expiry policy, and safe non-claims before reporting suspension. r[lattice.workflow_approval.suspension]
- [ ] [serial] S3 Return distinct suspended headless and TUI outcomes, end the current invocation, park no process, and dispatch no dependent node. r[lattice.workflow_approval.suspension]
- [ ] [parallel] S4 Add crash fixtures before and after request reservation, Choregraph event application, state commit, response output, and process exit. r[lattice.workflow_approval.suspension]

## Phase 4: Separate decisions and authority

- [ ] [serial] D1 Add read-only list and show commands with bounded safe request, revision, status, expiry, decision, and next-action facts. r[lattice.workflow_approval.decision]
- [ ] [serial] D2 Add a serialized decision command that records approve or reject only and never applies resume or dispatches workflow work. r[lattice.workflow_approval.decision]
- [ ] [serial] D3 Add the explicit local operator authority profile with exact request, actor, ability, policy, and expiry binding. r[lattice.workflow_approval.authority]
- [ ] [serial] D4 Add optional Basalt/UCAN authority input adaptation from already verified grants, receipts, caveat, replay, policy, and currentness facts without raw token authority. r[lattice.workflow_approval.authority]
- [ ] [parallel] D5 Add negative wrong-request, wrong-revision, wrong-ability, wrong-actor, stale-policy, stale-receipt, caveat-denied, replay-denied, raw-token, actor-text-only, expired, self-approval, and contradictory-decision cases. r[lattice.workflow_approval.authority] r[lattice.workflow_approval.terminal]

## Phase 5: Resume and terminal behavior

- [ ] [serial] R1 Add resume revalidation for store ownership, request, workflow, graph, selected inputs, artifacts, decision, authority policy, expiry, Choregraph profile, and event continuity. r[lattice.workflow_approval.resume]
- [ ] [serial] R2 Apply one matching Choregraph resume signal and dispatch only newly ready nodes through ordinary Lattice admission. r[lattice.workflow_approval.resume]
- [ ] [serial] R3 Preserve completed, failed, cancelled, abandoned, and unknown prior node outcomes without automatic rerun or reset. r[lattice.workflow_approval.resume] r[lattice.workflow_approval.terminal]
- [ ] [serial] R4 Accept only declared typed non-secret resume values and opaque credential references, recording key and reference identities without resolved values. r[lattice.workflow_approval.resume]
- [ ] [serial] R5 Add rejection, decision expiry, resume expiry, duplicate, contradictory, stale-revision, active-owner, and already-terminal outcomes with deterministic idempotence or denial. r[lattice.workflow_approval.terminal]
- [ ] [parallel] R6 Add process-restart integration fixtures that prove exact remaining-node execution and no duplicate completed local effects. r[lattice.workflow_approval.resume]

## Phase 6: Compatibility, UX, and evidence

- [ ] [serial] C1 Preserve existing orchestration `RevisionProposal` and `ApprovalDecision` storage, replay, CLI, JSON, and behavior through explicit compatibility adapters and parity tests. r[lattice.workflow_approval.compatibility]
- [ ] [parallel] C2 Add TUI inspection, separate approve or reject, and separate resume actions with safe confirmation and stale-state handling. r[lattice.workflow_approval.decision] r[lattice.workflow_approval.resume]
- [ ] [serial] C3 Add safe metadata-only approval receipts that bind selected identities, status, authority profile, expiry class, event continuity, and non-claims without reasons, credentials, workflow content, or raw tokens by default. r[lattice.workflow_approval.authority]
- [ ] [parallel] C4 Add leak and overclaim tests for prompts, reasons, actor secrets, tokens, credentials, environment values, workflow content, artifacts, provider output, human-identity claims, and independent-review claims. r[lattice.workflow_approval.authority]
- [ ] [serial] C5 Document approval-node authoring, suspension, separate decision and resume, local and Basalt authority, expiry, credentials, rejection, recovery, compatibility, and non-claims. r[lattice.workflow_approval.node] r[lattice.workflow_approval.compatibility]

## Phase 7: Closeout

- [ ] [serial] X1 Run focused tests, formatting, Clippy, `scripts/check.rs`, operator E2E crash evidence, Octet, pinned Choregraph downstream checks, Cairn validation and gates, and bounded Nix validation. r[lattice.workflow_approval.suspension] r[lattice.workflow_approval.resume]
- [ ] [serial] X2 Review dry-run sync and archive plans, make sure that every requirement reaches workflow-execution, and archive only after current positive and negative evidence passes. r[lattice.workflow_approval.terminal] r[lattice.workflow_approval.compatibility]
