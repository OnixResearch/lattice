## Context

This change depends on the typed local bundle catalog from `add-ui-workflow-template-catalog`. Lattice already has generic Pi/Clankers agent nodes, a bounded Animus leaf loop, bounded local process execution, and durable orchestration over immutable acyclic workflow rounds. The missing artifact is a reviewed template that composes those boundaries into an operator-visible coding-agent loop.

Two sibling scripts are design references, not runtime inputs:

- `../aspen/scripts/octet-burndown.scm`, reviewed at BLAKE3 `791fba3ab3c78ededa9406b57a3d90163984541d453f183f632d8326e3aa5322`, demonstrates baseline/probe, one fixer command, validation, strict count reduction, and policy restoration on observed paths.
- `../../mc/scripts/ralph-drain-cairns.scm`, reviewed at BLAKE3 `94ff7d3c3c527d272782c1bbd84b79da557fc6392af9dfbbaf7a21cb03f703b9`, demonstrates deterministic active-change discovery, bounded selection, plan-first output, one sequential Pi invocation per selected change, and positive/negative self-tests.

The implementation may port these invariants, but it must not import sibling paths, silently track future sibling revisions, or claim semantic equivalence with either script.

## Success Contract

Completion requires two UI-loadable, identity-bound bundles whose real loop is visible as Lattice rounds and whose deterministic fixtures prove strict progress and fail-closed authority. Merely adding documentation, embedding either Steel script, or executing one unbounded shell command is not completion.

False completion includes accepting a flat metric, starting more than one agent per round, continuing after validation/restoration failure, auto-committing or pushing, treating agent prose as a progress observation, or claiming that a lower count/archive proves correctness.

## Shared Round Architecture

```text
explicit operator start
        |
        v
Round N: immutable acyclic workflow revision
  Discover/Probe -> Select/Baseline -> One Agent -> Validate -> Measure -> Progress Gate
        |                                                  |
        | no work / terminal failure                       v
        +----------------------------------------------> terminal result
                                                           |
                                                           v
                                       inert successor proposal + evidence
                                                           |
                                                operator approval + budgets
                                                           |
                                                           v
                                                       Round N+1
```

Every stage is a normal typed node under the existing runner. Feedback occurs only between terminal rounds through durable orchestration. The template cannot add a graph back-edge, invoke a nested autonomous loop, or turn agent output into approval.

The bundle fixes the workflow, orchestration definition, role prompt, helper assets, command text, target labels, metric direction, and budget policy by BLAKE3 identity. Editing any of these creates a different draft that must be revalidated and explicitly started.

## Pure Monotonic Progress Core

A small pure core consumes explicit typed observations:

```text
ProgressPolicy { direction, minimum_delta }
ProgressObservation { before, after, validation_passed, restoration_confirmed }
ProgressDecision = Improved | Complete | Blocked(reason)
```

The first policies use decreasing unsigned integer metrics and a named one-unit minimum delta. Checked arithmetic rejects overflow or underflow. `Improved` requires successful validation, confirmed restoration where applicable, and `after < before`. Equality, regression, missing values, malformed values, duplicate/conflicting observations, or failed prerequisites return stable blocked decisions and no successor authority.

Metrics are observations, not truth. The core does not read files, parse process streams, run validation, call agents, commit, push, or mutate orchestration state. Shell adapters parse bounded outputs into typed observations; orchestration admission consumes only accepted decisions.

## Cairn Agent Drain Bundle

The `cairn-agent-drain` round follows the Ralph reference at Lattice boundaries:

1. Run the fixed declared Cairn list/validation adapter in the admitted repository root and parse a bounded deterministic active-change list.
2. If the list is empty, return terminal `Complete` without invoking an agent.
3. Select exactly one change by an explicit requested id or deterministic sorted-first policy; record the selected change and pre-round active count.
4. Invoke exactly one configured Pi agent with a compact goal-first prompt bound to that change, the repo-local instructions, proposal/design/tasks/spec artifacts, required gates, smallest relevant baseline, validation expectations, archive conditions, and no-push rule.
5. Run the fixed post-agent Cairn/repository validation and list adapters.
6. Admit an inert successor proposal only when validation passes, the selected change is no longer active, and the active count strictly decreases.

The template starts in plan/preview mode and displays the exact root, commands, agent target, selected change, budgets, and effects before operator start. An agent may edit, test, sync, archive, and commit only within the authority explicitly granted by the configured Pi route and working directory. The template never grants push, force-push, PR, credential, approval, command-selection, or budget-expansion authority.

If the agent exits successfully but the change remains active or validation fails, the round is blocked. Lattice records the bounded observation and does not fabricate completion, automatically retry, or roll back the working tree.

## Octet Agent Burn-down Bundle

The `octet-agent-burndown` round follows the Octet guard reference while keeping the agent as one typed node:

1. Capture the exact pre-probe policy bytes and BLAKE3 identity, then run the fixed no-disabled Octet probe through a bounded helper adapter.
2. Confirm the policy path has the exact captured bytes before agent dispatch. A mismatch blocks the round.
3. Invoke exactly one configured Pi agent with the latest bounded probe summary and a prompt to make one small low-risk refactor without relaxing checks, committing, or pushing.
4. Run the fixed validation command.
5. Run one post-agent no-disabled probe and again confirm exact policy restoration.
6. Admit an inert successor proposal only when validation passes and the post-agent finding count is strictly lower.

The helper should prefer a non-mutating Octet configuration mechanism when supported. If temporary policy replacement is unavoidable, it must use an owned same-filesystem backup, exact-byte/BLAKE3 checks, cleanup on every observed success/error path, and a recovery check before any later agent dispatch. On startup/recovery, an unexpected policy identity blocks for operator inspection; Lattice must not overwrite an uncertain operator file automatically.

An abrupt host/process death may still occur between temporary replacement and restoration. Evidence must state this boundary. The template does not claim arbitrary kill safety and cannot start another agent until restoration is confirmed.

A flat/regressing post-probe leaves candidate edits for operator inspection. Automatic rollback is prohibited because the working tree may contain unrelated user changes.

## Authority, Budget, and Effect Policy

Both orchestration definitions require:

- explicit operator start and explicit approval of each successor proposal;
- positive named limits for rounds, agent invocations, tokens, and duration;
- a per-round upper bound of exactly one agent invocation;
- no fallback target, automatic retry, self-approval, or agent-selected runtime;
- fixed declared discovery/probe/validation commands visible in preview;
- bounded stdin/stdout/stderr/deadline/cancellation through existing `bounded-exec` integration;
- redaction-safe summaries and artifact identities without prompts, model responses, credentials, or unrestricted command output;
- no automatic commit, push, force-push, PR operation, credential creation, or trigger mutation.

Operator selection of a template and repository root is an authority grant, not sandboxing. Lattice validates declared boundaries but does not prove arbitrary command or agent behavior safe.

## Fixtures and Validation

Deterministic fake adapters cover:

- no active Cairn changes and zero agent calls;
- one selected change, successful archive/validation, and strict count reduction;
- missing selected change, unchanged/increased count, validation failure, malformed list, and duplicate/conflicting observations;
- Octet strict reduction with exact restoration;
- equal/increased/missing/malformed findings, probe/validation timeout or truncation, restoration mismatch, and recovery mismatch;
- exhausted budgets, stale approval, revision drift, cancellation, attempted second agent call, fallback request, and agent output requesting commit/push/authority changes.

A focused real smoke test may validate command parsing and bundle admission without contacting a model or mutating a real user repository. Repository-wide checks remain authoritative for implementation acceptance.

## Portfolio Search Decision

| Family | Mechanism | Decision |
|---|---|---|
| Leaf agent template | Add one preconfigured Pi/Animus node | Rejected for this goal: useful but does not model cross-round progress, approval, or recovery. |
| Opaque Steel wrapper | Execute either sibling script inside one shell node | Rejected: hides agent rounds, budgets, evidence, and authority inside a child process. |
| Unbounded cyclic workflow | Add a feedback edge from measurement to agent | Rejected by DAG admission and unsafe for effects/recovery. |
| Guarded orchestration bundles | One acyclic typed round plus strict progress and approved successor | Selected: ports both references' useful invariants into existing Lattice-owned boundaries. |

Adversarial review focused on false progress, agent self-approval, nested agent invocation, command substitution, budget omission, policy restoration, crash windows, stale revisions, working-tree rollback, auto-commit/push, and overclaiming. The selected design blocks progression before another agent effect when any required fact is absent or contradictory.

## Risks / Trade-offs

- Coding agents retain broad filesystem/process authority granted by their configured route; templates expose and bound that grant but do not sandbox it.
- Strict improvement can reject valuable neutral refactors and can reward superficial count reduction; operator review remains mandatory.
- Cairn active-count reduction is a lifecycle observation, not evidence that implementation or proofs are correct.
- Octet policy replacement has a crash window unless a non-mutating probe interface is available; recovery blocks rather than guessing.
- Exact bundle identities create regeneration work when prompts, commands, workflows, budgets, or references change.
- Sequential one-agent rounds are slower than parallel agents but make ownership, evidence, and stop conditions reviewable.
