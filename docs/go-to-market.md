# Go-to-market plan

This is the public launch plan for the Lattice 0.1 release candidate. It is
grounded in what the repository currently implements and deliberately excludes
claims that the runtime, contracts, or release gates do not establish.

## Positioning

**Lattice is a local-first, graph-native workflow runtime with explicit effect
authority and durable execution state.**

It is for teams that want to run multi-step shell, file, HTTP, and agent
workflows on their own hosts without hiding authority behind an implicit
credential store or an always-on SaaS control plane. Workflows are typed Nickel
values. Execution is a DAG. Effects fail closed unless an operator grants an
exact, finite-lived profile. Acceptance, observation, cancellation, and recovery
are identity-bound and durable.

Short form:

> Typed workflows, durable local execution, and effects that fail closed until
> you grant them.

## Primary audience

1. **Platform and CI engineers** who need durable workflow execution next to
   their build and release systems, with a stable consumer contract instead of
   a hosted control plane.
2. **Agent-runtime builders** who need bounded multi-round orchestration,
   explicit budgets, and approval boundaries rather than an unbounded agent loop.
3. **Local-first operators** who want a TUI and CLI for authoring, inspecting,
   replaying, and recovering workflows on a machine they control.

The first release is not aimed at non-technical workflow authors, teams that need
multi-tenant isolation, or buyers who require a hosted SLA.

## Proof points

Use these links instead of unsupported marketing claims:

- Typed Nickel workflow import/export and validation:
  [README](../README.md), [workflow format](../.cairn/specs/workflow-format/spec.md).
- Fail-closed effect authority with exact grants and finite expiry:
  [effect authority](choregraph-effect-authority.md).
- Durable approvals, scheduling, dispatch recovery, and storage uncertainty:
  [approvals](workflow-approvals.md),
  [storage uncertainty](storage-commit-uncertainty.md).
- Bounded ready-frontier execution and node memoization:
  [frontier](bounded-ready-frontier.md), [memoization](dag-node-memoization.md).
- Bounded agents, budgets, evaluation, and candidate isolation:
  [orchestration identities](orchestration-identities.md),
  [budget usage](orchestration-budget-usage.md),
  [agent evaluation](agent-evaluation.md),
  [candidate worker](candidate-worker.md).
- Versioned consumer contract and local Unix transport:
  [contract](../crates/lattice-workflow-contract/README.md),
  [transport](workflow-exchange-transport.md),
  [application](workflow-exchange-application.md).
- Repeatable local gates:
  [release readiness](release-readiness-integration.md),
  [release process](release-process.md).

## What the launch must not claim

- Lattice is not a sandbox for untrusted workflows.
- The workflow exchange is not an internet-facing service; it has no TLS,
  remote authentication, or multi-tenant isolation.
- Authorization is not evidence that an external effect succeeded.
- Local replay and bounded checks do not prove arbitrary workflow correctness.
- The release candidate does not include a hosted service, support SLA, or
  production-readiness certification.
- The optional Tile compose integration is not part of the public build closure
  and requires authorized source access.

## Launch checklist

### Repository

- [ ] `Cargo.toml` version, `CHANGELOG.md`, and tag agree.
- [ ] `./scripts/check.rs`, `octet-precommit-check`, and `nix flake check` pass on
      the tagged commit.
- [ ] `nix build .#lattice` produces a binary with working `--version` and
      `--help`.
- [ ] `SECURITY.md`, `CONTRIBUTING.md`, and the deployment guide are current.
- [ ] The default build closure has no authenticated source requirement.
- [ ] Release notes state the non-claims and the local-service boundary.

### Demo

- [ ] Run the effect-free import/export quickstart from a clean directory.
- [ ] Show a run failing closed with `AuthorityUnavailable`.
- [ ] Show `authority inspect` and the exact grant tuples.
- [ ] Grant a finite-lived profile, execute the same workflow, and show the
      per-node results.
- [ ] Open the TUI and show the graph, properties, and run history.
- [ ] Start `workflow-exchange serve` against a reviewed handler profile and
      show the socket plus journald logs.
- [ ] Show one durable recovery or approval path, not just the happy path.

### Publication

- [ ] Tag `v0.1.0` and let the release workflow publish the binary archive and
      checksums.
- [ ] Publish a launch post that links the repository, quickstart, security
      policy, and release notes.
- [ ] Post to the developer channels where the target audience already discusses
      workflow and agent runtimes.
- [ ] Announce the Radicle and GitHub source locations.
- [ ] Route security reports to the private advisory process, not public issues.

### First 30 days

- [ ] Track successful quickstarts, first workflow imports, exchange consumers,
      and issue quality; do not use commit counts or archive counts as evidence.
- [ ] Fix documentation gaps reported by new users before adding surface area.
- [ ] Decide whether a hosted or managed offering is in scope. If it is, define
      its isolation, authentication, and support boundary in a separate change.
- [ ] Revisit the Tile compose distribution decision: publish, vendor, or keep it
      as an authorized optional integration.

## Ten-minute demo script

1. **Build** (2 min): `nix build .#lattice`, show `--version`, open `--help`.
2. **Import and export** (2 min): run the daily-digest quickstart in a temporary
   directory. State that neither command executes the shell pipeline.
3. **Fail closed** (2 min): run the saved workflow without an authority profile.
   Show the failed run and the stored `AuthorityUnavailable` error.
4. **Inspect and grant** (2 min): run `authority inspect`, review the exact
   grants, write a one-hour profile, and rerun. Show `steps_run`, `completed`,
   and `failures` rather than claiming the workflow was "correct".
5. **Operate** (2 min): open the TUI, show the graph and run history, then start
   the exchange service and show the socket, systemd status, and bounded logs.

Stop at the evidence. Do not demonstrate destructive or credential-bearing
workflows with real credentials.

## Objection handling

**"How is this different from a hosted workflow SaaS?"**
Lattice runs locally and keeps authority, state, and effects on your host. The
tradeoff is that you operate it; there is no hosted isolation or SLA.

**"How is this different from a general agent framework?"**
The graph, machine transitions, budgets, approvals, and effect authority are
typed and durable. Agents are adapters inside that boundary, not the boundary
itself.

**"Can I run untrusted workflows?"**
No. Workflows run with host authority. Use separate hosts or external sandboxes
for untrusted input; the runtime's bounds are not a confinement proof.

**"Why AGPL?"**
New Lattice-owned source is AGPL-3.0-or-later. Historical MIT grants remain in
the package-local `LICENSE-MIT-HISTORICAL` files. Review the licensing boundary
before importing the contract into a differently licensed consumer.

**"Why is the compose feature separate?"**
The compose integration depends on an exact Tile source revision that is not in
the public build closure. The server and default runtime do not need it.

## Success signals

- A new user completes import, fail-closed execution, inspection, and an
  authorized run without maintainer help.
- A consumer can pin the contract and complete one admitted exchange against the
  documented profile.
- Security reports arrive through the private channel with reproducible detail.
- Documentation issues decrease as quickstart friction is fixed.

## Open decisions

- Hosted versus self-hosted-only distribution.
- Commercial support or managed-service scope.
- Public distribution of the Tile compose integration.
- Long-term compatibility policy for contract and transport versions after 1.0.
