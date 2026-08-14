# Guarded Agent Loop Templates Specification

## Purpose

Defines the `guarded-agent-loop-templates` capability.

## Requirements

### Requirement: Guarded agent-loop workflow rounds
r[lattice.guarded_agent_templates.rounds] Lattice MUST represent each guarded coding-agent iteration as one immutable acyclic workflow round containing explicit discovery or baseline work, at most one bounded agent invocation, validation, post-measurement, and a typed progress decision under durable orchestration.

#### Scenario: Admitted round invokes one agent
r[lattice.guarded_agent_templates.rounds.scenario.one-agent]
- GIVEN an operator explicitly starts a valid guarded bundle with available work, admitted authority, compatible runtimes and effects, and sufficient positive round, agent, token, and duration budgets
- WHEN one workflow round executes
- THEN Lattice MUST invoke at most one configured agent and MUST expose discovery, validation, measurement, and progress as ordinary typed nodes and persisted round evidence

#### Scenario: Hidden loop or second agent is rejected
r[lattice.guarded_agent_templates.rounds.scenario.hidden-loop]
- GIVEN a candidate bundle contains a cyclic edge, a shell command that owns nested autonomous agent rounds, or a per-round path that can invoke more than one agent
- WHEN bundle or round admission runs
- THEN Lattice MUST reject the candidate before agent dispatch and MUST NOT treat the hidden work as a guarded Lattice loop

### Requirement: Strict monotonic progress admission
r[lattice.guarded_agent_templates.progress] Lattice MUST decide guarded-loop continuation through a pure deterministic comparison of typed bounded before/after observations and SHALL authorize a successor only after all required validations pass and the configured metric improves by at least its positive named minimum delta.

#### Scenario: Strict decrease permits an inert proposal
r[lattice.guarded_agent_templates.progress.scenario.improved]
- GIVEN a decreasing integer metric has valid before and after observations, every required validation and restoration fact is confirmed, and the after value is lower by at least the configured minimum delta
- WHEN the pure progress core evaluates the round
- THEN it MAY return an improved decision that authorizes only an inert successor proposal pending normal approval and fresh budget admission

#### Scenario: Flat, regressing, or invalid observation blocks continuation
r[lattice.guarded_agent_templates.progress.scenario.blocked]
- GIVEN the after metric is equal to or greater than the before metric, or either observation is missing, malformed, overflowing, duplicated with conflicting content, or paired with a failed prerequisite
- WHEN the pure progress core evaluates the round
- THEN it MUST return a stable blocked decision and MUST authorize no successor approval, retry, budget reservation, agent invocation, commit, or push

### Requirement: Cairn agent-drain template
r[lattice.guarded_agent_templates.cairn] Lattice MUST provide a built-in `cairn-agent-drain` workflow/orchestration bundle that deterministically discovers active Cairn changes, selects no more than one admitted change per round, invokes one configured agent, validates the repository, and measures active-change reduction.

#### Scenario: No active change completes without an agent
r[lattice.guarded_agent_templates.cairn.scenario.empty]
- GIVEN the admitted Cairn list adapter returns a valid empty active-change set
- WHEN the round evaluates discovery
- THEN the orchestration MUST complete without invoking an agent, mutation command, approval, retry, commit, or push

#### Scenario: Valid drained change proposes the next round
r[lattice.guarded_agent_templates.cairn.scenario.drained]
- GIVEN discovery selects one exact active change, one agent round completes, repository and Cairn validation pass, the selected change is no longer active, and the active count strictly decreases
- WHEN progress admission evaluates the terminal round
- THEN Lattice MAY record an inert next-round proposal with the exact change, command, workflow, validation, metric, and artifact identities pending explicit operator approval

#### Scenario: Undrained or invalid change stops progression
r[lattice.guarded_agent_templates.cairn.scenario.blocked]
- GIVEN the selected change remains active, active count is flat or higher, validation fails, list output is malformed, the command times out or truncates, or the observed change differs from the admitted selection
- WHEN the round terminates
- THEN Lattice MUST block continuation without fabricating archive success, retrying automatically, reverting the working tree, committing, or pushing

### Requirement: Octet agent burn-down template
r[lattice.guarded_agent_templates.octet] Lattice MUST provide a built-in `octet-agent-burndown` workflow/orchestration bundle that records a bounded no-disabled baseline, confirms exact Octet policy restoration, invokes one configured agent candidate, validates, records one post-candidate probe, and requires strict finding-count reduction.

#### Scenario: Reduced findings permit the next proposal
r[lattice.guarded_agent_templates.octet.scenario.reduced]
- GIVEN the baseline and post-candidate probes are valid, validation passes, exact original policy bytes are confirmed after every probe, and the post-candidate finding count is lower
- WHEN progress admission evaluates the terminal round
- THEN Lattice MAY record an inert next-round proposal with exact probe, policy, validation, workflow, and metric identities pending explicit operator approval and fresh budget admission

#### Scenario: Policy restoration failure blocks the agent
r[lattice.guarded_agent_templates.octet.scenario.restoration]
- GIVEN a no-disabled probe changes the policy file and the exact captured original bytes and BLAKE3 identity cannot be confirmed before agent dispatch or continuation
- WHEN the round or recovery path evaluates restoration
- THEN Lattice MUST block for operator inspection and MUST NOT invoke another agent, overwrite an uncertain operator file, accept progress, commit, or push

#### Scenario: Non-improving candidate remains operator-owned
r[lattice.guarded_agent_templates.octet.scenario.non-improving]
- GIVEN the agent candidate leaves findings equal or higher, validation or a probe fails, output is malformed, timed out, or truncated, or policy restoration is uncertain
- WHEN the round terminates
- THEN Lattice MUST block continuation, retain bounded diagnostics, and MUST NOT automatically retry, roll back working-tree edits, relax Octet policy, commit, or push

### Requirement: Guarded template authority and budgets
r[lattice.guarded_agent_templates.authority] Lattice MUST bind guarded-loop workflows, prompts, helper assets, commands, agent targets, progress policy, positive budgets, actors, and successor approvals to exact identities and MUST deny agent output any ambient authority to change those bindings.

#### Scenario: Explicit operator approval admits an exact successor
r[lattice.guarded_agent_templates.authority.scenario.approved]
- GIVEN an improved terminal round produced an exact inert proposal and an authorized operator approves that proposal against the current instance epoch, workflow revision, policy, and remaining budgets
- WHEN normal orchestration admission succeeds
- THEN Lattice MAY start one successor round with a fresh run identity, dispatch identity, and budget reservation

#### Scenario: Agent authority escalation remains inert
r[lattice.guarded_agent_templates.authority.scenario.denied]
- GIVEN agent output requests a second agent, different executable or target, broader working directory, credential, trigger, budget increase, validation relaxation, self-approval, commit, push, force-push, or pull-request operation
- WHEN the round maps that output
- THEN Lattice MUST treat the request only as ordinary bounded output and MUST NOT apply or execute the requested authority

#### Scenario: Budget exhaustion stops before effects
r[lattice.guarded_agent_templates.authority.scenario.budget]
- GIVEN any enforced round, agent-invocation, token, or duration bound is absent, non-positive, exhausted, or insufficient for the next admitted upper bound
- WHEN dispatch planning runs
- THEN Lattice MUST refuse the round before invoking discovery, probe, agent, validation, measurement, commit, or push effects

### Requirement: Guarded template evidence and non-claims
r[lattice.guarded_agent_templates.evidence] Lattice MUST retain bounded redaction-safe identities and outcomes for guarded template rounds and MUST NOT promote lifecycle or metric observations into unsupported correctness, safety, or completion claims.

#### Scenario: Status reports actionable bounded evidence
r[lattice.guarded_agent_templates.evidence.scenario.status]
- GIVEN a guarded instance is waiting, running, blocked, awaiting approval, completed, failed, cancelled, or exhausted
- WHEN an operator inspects status or replay
- THEN Lattice MUST report safe bundle, revision, round, selected-work fingerprint, validation class, before/after metric, progress decision, restoration state, budget, and next-action facts without raw prompts, model responses, credentials, or unrestricted command output

#### Scenario: Passing metric remains a bounded observation
r[lattice.guarded_agent_templates.evidence.scenario.non-claim]
- GIVEN a Cairn active count or Octet finding count strictly decreases and local replay validates
- WHEN Lattice renders terminal evidence
- THEN it MUST NOT claim implementation correctness, semantic improvement, proof validity, complete repository drainage, release eligibility, provider truth, remote effect uniqueness, arbitrary crash safety, or production readiness
