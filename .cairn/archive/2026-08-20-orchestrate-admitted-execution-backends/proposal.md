## Why

Lattice owns workflow runtime selection, effects, persistence, retries, triggers, diagnostics, and operator authority. It supports native agents, bounded processes, and selected adapters, but workflow nodes cannot declare backend-neutral execution requirements and receive one exact admitted backend plan.

Animus is planning a pure execution backend selector inspired by agentOS's lightweight-to-heavy execution ladder. Lattice needs a consumer change that revalidates this advice, owns final product selection, persists exact attempt identity, and denies hidden fallback.

## What Changes

- Add workflow execution requirement and backend selection references to typed operation contracts.
- Consume one exact published Animus planner cohort through frozen fixtures before any direct dependency.
- Revalidate the selected plan against Lattice adapters, credentials, effects, budgets, authority, persistence, and recovery policy.
- Persist the exact backend, adapter, profile, operation, attempt, and current-enforcement identities before effects.
- Deny silent backend substitution during initial dispatch, retry, recovery, or resume.
- Add operator explanation and safe run-inspection fields without exposing commands, credentials, prompts, or host paths.

## Impact

- **Files**: workflow schemas, operation contracts, runtime selection, persistence, recovery, CLI/TUI diagnostics, fixtures, docs, and workflow execution requirements.
- **Dependencies**: Local typed operation contracts and a published Animus execution planner cohort.
- **Testing**: Positive backend selections plus negative drift, missing adapter, stale plan, authority denial, credential mismatch, fallback, retry substitution, and unknown-outcome cases.
- **Boundary**: Lattice admission chooses one product runtime route. It does not prove sandbox containment, provider availability, external effect success, or result correctness.
