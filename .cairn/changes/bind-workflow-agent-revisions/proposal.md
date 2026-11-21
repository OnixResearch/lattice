# Bind workflow and agent revisions, and compare an OCI package envelope

Tracking: [Lattice #25](https://github.com/OnixResearch/lattice/issues/25), the follow-ups "Pin compiled workflow and agent revisions to Cairn campaigns" (fixture level) and "NVIDIA research follow-up: compare a digest-bound OCI skill/package envelope". Owner-approved on 2026-11-21.

## Why

`manage-agent-revisions` manages agent profile revisions only. A Cairn campaign runs a compiled workflow-intent plan with an agent profile, and either can change: a plan, a prompt (obligation guidance), a tool, a requirement or a policy. Nothing binds the two into one immutable revision, so promotion could select a worker revision for a plan it was never qualified with, and an adaptation has no identity of its own or slot for the Kiln/Cairn applicability decision.

Separately, the issue asks for a bounded comparison of the NVIDIA agent-package draft (`697872570744c8cfc1cf6b1d72e04db66e1a2c83`) with Lattice bundle identities before any distribution format is chosen.

## What Changes

- `lattice.workflow-agent-bundle.v1`: a lifecycle `Bundle` whose optional `workflow` binding pairs a compiled plan's identities (plan schema, compiler revision and artifact, plan, graph, runtime profile, the Cairn cohort and per-obligation identities) with a worker bundle (definition, name, profile revision and bundle identity). Its definition is `lattice-workflow-agent-definition-v1` over the pairing name and worker definition; its revision is the bare-hex `lattice-workflow-agent-revision-v1` over the canonical binding. A plain agent bundle omits the field, so its bytes and identity are unchanged. r[lattice.agent_revisions.workflow]
- The existing desire, prepare, qualify, promote, admit, revoke and rollback transitions manage these bundles unchanged; a qualification answers for the bundle's own definition and revision. r[lattice.agent_revisions.workflow.scenario.lifecycle] r[lattice.agent_revisions.workflow.scenario.evaluation]
- `workflow::adaptation::propose` describes a proposed adaptation as a new revision with its changes and an applicability slot fixed to `unknown`, owned by Cairn #14 and Kiln #83. It changes no lifecycle. r[lattice.agent_revisions.workflow.adaptation]
- `workflow::publication::slots` projects `lattice.workflow-agent-revision-slots.v1` for Kiln #84 with the workflow and worker revisions apart; `agent-revision inspect` adds it as `workflow_slots`. r[lattice.agent_revisions.workflow.projection]
- `agent-revision resolve-workflow` and `desire-workflow` compose a bundle from an implementation-attempt request and a typed intent naming this compiler executable. r[lattice.agent_revisions.workflow.inspect]
- `workflow::check_attempt` checks an attempt against the workflow-agent bundle it names. An implementation-attempt request may carry an optional, per-attempt `workflow_agent` `{definition, revision}`; `seam::subject` is the seam's one subject selection, so admission, effects, retained output and closure key on the named pair, and admission loads the bundle and runs `check_attempt` before any intent. Absent, requests keep their bytes, identity and profile revision. r[lattice.agent_revisions.workflow.attempt]
- A draft-schema fixture comparison of one reviewed skill and one plugin with bundled skills, 31 negative cases, the payload-change, relocation, revocation and rollback cases, and the decision `docs/agent-package-comparison.md` (useful mapping, adoption deferred). r[lattice.agent_revisions.package_comparison]

## Impact

- **Pure core:** new `crates/lattice-core/src/agent_revision/workflow/{mod,adaptation,publication}.rs`; `facts.rs` (optional `workflow`), `identity.rs` (two domains), one module line in `agent_revision/mod.rs`.
- **Shell:** new `src/agent_revision/composite.rs`; `src/agent_revision/{mod,commands,bundle,seam}.rs`; `src/workflow/intent/{model,cli}.rs` (plan accessors, `load` visible in the crate); `src/implementation_attempt.rs` (`WorkflowAgent`), the `workflow_agent` request field in `src/implementation_attempt/cli.rs`, `cli/session/revision.rs` and the pin read in `cli/inspect/mod.rs`.
- **Tests and fixtures:** `tests/test_agent_revision_workflow.rs`, `tests/test_agent_package_comparison.rs`, two tests in `tests/test_agent_revision_shell.rs`, `src/implementation_attempt/cli/tests/workflow_agent.rs`, `tests/fixtures/agent-package/`, `tests/fixtures/agent-revisions/workflow-agent-identities.json`.
- **Docs:** `docs/agent-revisions.md`, `docs/candidate-worker.md`, new `docs/agent-package-comparison.md`, changelog.
- **Compatibility:** every existing bundle, revision, lifecycle row, inventory, request without `workflow_agent` and CLI command keeps its bytes and behavior. No Cargo dependency is added.

## Out of Scope

- Publication to Kiln #84 and any Kiln reader; live preparation, qualification or canary (Kiln #49); a Cairn #14 producer cohort.
- Any OCI registry, installer, materialization or activation controller.
