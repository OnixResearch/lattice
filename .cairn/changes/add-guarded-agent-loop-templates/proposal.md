## Why

Lattice has a bounded Animus leaf example and a durable LLM orchestration example, but no UI-loadable examples for guarded coding-agent work loops. Two reviewed sibling scripts demonstrate complementary mechanics: `../../mc/scripts/ralph-drain-cairns.scm` uses plan-first bounded work selection and one sequential Pi invocation per Cairn change, while `../aspen/scripts/octet-burndown.scm` accepts a candidate iteration only after validation and a strict finding-count reduction.

Copying either script into one shell node would make the loop opaque to Lattice and would collapse per-round budgets, approvals, evidence, and failure boundaries. Lattice should instead encode their useful invariants as typed acyclic workflow rounds under the existing durable orchestration layer.

## What Changes

- Add two built-in UI-loadable workflow/orchestration bundles after the workflow-template catalog is available: `cairn-agent-drain` and `octet-agent-burndown`.
- Model each iteration as one normal acyclic workflow round with explicit discovery/probe, one bounded agent invocation, validation, post-measurement, and progress-gate stages.
- Add a pure monotonic-progress decision that admits a successor round only when the configured integer metric strictly improves and rejects missing, malformed, flat, or regressing observations.
- Use the Ralph reference for deterministic Cairn change discovery, bounded selection, plan-first prompts, sequential execution, and end-to-end validation reporting.
- Use the Octet reference for baseline/final probes, exact policy restoration, validation-before-acceptance, and strict no-disabled-finding reduction.
- Require explicit operator start and successor approval, positive round/agent/token/duration budgets, one agent invocation per round, fixed declared commands, and no automatic commit or push.
- Add redaction-safe result/progress evidence and explicit non-claims for agent correctness, external effect uniqueness, arbitrary crash safety, provider truth, release eligibility, or successful repository drainage.
- Record both sibling scripts as design references only; neither path becomes a runtime dependency.

## Success Criteria

- Both guarded agent-loop bundles load from the UI catalog, preview without effects, and validate against their exact workflow and orchestration identities.
- The Cairn bundle performs zero agent calls when no active change exists; otherwise it selects only the admitted change, invokes one agent, validates, and proposes another round only after the active-change count strictly decreases.
- The Octet bundle records a baseline, invokes one agent candidate, validates, restores the exact original policy bytes before further agent work, and proposes another round only after the no-disabled finding count strictly decreases.
- Flat/regressing metrics, malformed probe output, command failure, timeout, truncation, missing authority, exhausted budget, failed policy restoration, or revision drift stop progression without automatic commit, push, approval, or fallback.
- Positive and negative fixtures exercise clean completion, strict improvement, no work, command failure, missing metrics, flat/regressing metrics, restoration failure, budget exhaustion, stale approval, cancellation, and reference-boundary non-claims.

## Non-Goals

- Running either reference Steel script as an opaque child loop or depending on sibling workspace paths at runtime.
- Granting agents ambient permission to select executables, credentials, validation policy, budgets, successors, commits, pushes, or approvals.
- Automatically reverting an unsuccessful agent's working-tree edits, which could destroy unrelated operator work.
- Claiming a lower finding count proves semantic improvement, or that an archived Cairn change proves implementation correctness or release readiness.
- Supporting arbitrary optimization directions, unbounded loops, parallel coding agents, remote workers, or hidden workflow cycles in the first slice.

## Impact

- **Dependency**: implementation follows `add-ui-workflow-template-catalog` and reuses accepted workflow-template, agent-runtime, orchestration, bounded-process, and local-file boundaries.
- **Files**: built-in Nickel bundles, role prompts and bounded helper assets, pure progress core, example documentation, focused fixtures/tests, README references, and Cairn traceability artifacts.
- **Compatibility**: additive examples only; existing workflows, node templates, agent targets, and orchestration definitions retain their current behavior.
- **Testing**: baseline relevant tests before core edits; deterministic fake-adapter tests for every positive and negative route; focused real CLI smoke tests without provider calls; repository, Octet, Cairn, and bounded Nix validation.
