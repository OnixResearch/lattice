# Lattice documentation

Start with the [README](../README.md) for the quickstart. This index groups the
maintained topic documents by the question they answer.

## Operating the runtime

- [Server deployment](server-deployment.md): run the workflow exchange as a
  local systemd service, with socket, peer, and hardening guidance.
- [Choregraph effect authority](choregraph-effect-authority.md): inspect exact
  workflow and policy identities, author a finite-lived operator profile, and
  understand fail-closed effects.
- [Storage commit uncertainty](storage-commit-uncertainty.md): what a crash or
  lost observation does and does not prove.
- [Durable workflow approvals](workflow-approvals.md): bind an approval decision
  to an exact suspended workflow and resume it separately.
- [Bounded ready-frontier dispatch](bounded-ready-frontier.md): concurrent
  execution and resource limits.

## Authoring and execution

- [TUI authoring](tui-authoring.md): typed editors, validation, and lifecycle
  behavior in the terminal UI.
- [Expression dependencies](expression-dependencies.md): explicit data mapping
  and dependency edges.
- [Workflow domain boundaries](workflow-domain-boundaries.md): which layer owns
  pure decisions, persistence, and host effects.
- [Bounded DAG node memoization](dag-node-memoization.md): node result reuse and
  its identity rules.
- [Workflow template bundles](workflow-template-bundles.md): importable local
  template catalogs.
- [Role-protocol execution](role-protocol-execution.md): projected local
  protocols and their bounds.

## Agents and orchestration

- [Orchestration identities](orchestration-identities.md): immutable revisions,
  budgets, and approval boundaries.
- [Orchestration budget usage](orchestration-budget-usage.md): reservations
  versus measured tokens, duration, and cost.
- [Guarded agent templates](guarded-agent-templates.md): objective checks,
  progress rules, and operator-controlled continuation.
- [Agent evaluation](agent-evaluation.md): identity-bound plans and
  metadata-only receipts.
- [Candidate worker boundary](candidate-worker.md): the bounded command route and
  its non-claims.
- [Structured input projection](structured-input-projection.md): bounded agent
  input projection.
- [Secretspec credentials](secretspec-credentials.md): credential references and
  provider delivery.

## Operations, contracts, and integration

- [Operation contracts](operation-contracts.md): typed reusable methods.
- [Operation data](operation-data.md): versioned output data.
- [Operation packs](operation-packs.md): content-addressed runtime admission.
- [Execution backend orchestration](execution-backend-orchestration.md): admitted
  backends and dispatch bounds.
- [Workflow exchange transport](workflow-exchange-transport.md): the versioned
  local Unix transport.
- [Durable workflow exchange application](workflow-exchange-application.md):
  handler profiles, durable admission, lifecycle, and logs.
- [Workflow exchange consumer compatibility](workflow-exchange-consumer-compatibility.md):
  the pinned consumer boundary and licensing record.
- [Compose surface pane binding](compose-surface-pane-binding.md): the optional
  Tile integration in the separate `compose/` workspace.

## Evidence and release

- [Release readiness integration](release-readiness-integration.md): Cairn
  selection, observed MCP inputs, and required gates.
- [Release process](release-process.md): versioning, tags, artifacts, and
  rollback.
- [Go-to-market](go-to-market.md): positioning, launch checklist, and demo
  script.
- [Security policy](../SECURITY.md): reporting and trust boundaries.
- [Contributing](../CONTRIBUTING.md): environment, checks, and lifecycle.

## Source-specific records

- [Content identity source](content-identity-source.md)
- [Nickel evaluator source](nickel-eval-source.md)
- [Radicle core pilot decisions](radicle-core-pilot-decisions.md)
- [GitHub asynchronous checks](github-async-checks.md)
