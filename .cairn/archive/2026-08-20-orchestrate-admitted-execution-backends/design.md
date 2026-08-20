## Context

Lattice maps workflow nodes to concrete runtime adapters and persists run attempts before effects. The selection is product-specific and lacks one typed requirement and explanation boundary across embedded, guarded-process, isolated-host, and external-machine routes.

Animus will own a pure backend-neutral planner. Lattice remains the final owner of runtime selection, credentials, effects, budgets, persistence, retries, recovery, and operator actions.

## Decisions

### Decision: Consume planner advice, retain product authority

**Choice:** Validate a frozen Animus plan projection, then map it into a Lattice-owned operation attempt. A planner allow never bypasses Lattice policy, adapter, authority, credential, budget, or persistence checks.

**Rationale:** Cross-repository advice must not transfer product authority.

### Decision: Persist exact backend identity before effects

**Choice:** Reserve the run and record workflow revision, node, operation contract, requirement profile, planner cohort, backend class, adapter, backend profile, authority policy, resource bounds, and attempt identity before dispatch.

**Rationale:** Recovery must know which route can have produced an unknown external outcome.

### Decision: No fallback across attempt boundaries

**Choice:** Backend unavailability or drift denies the attempt. Retry or resume can choose another backend only through a new explicit plan and a new attempt after uncertainty policy allows it.

**Rationale:** Silent substitution changes effect and containment assumptions.

### Decision: Explain without exposing payloads

**Choice:** CLI and TUI inspection show safe requirement classes, selected backend class, identities, observations, and denial reasons. Commands, prompts, credentials, environment values, and physical paths remain omitted.

**Rationale:** Operators need route visibility without another secret-bearing log.

## Risks / Trade-offs

- A planner cohort can drift from Lattice adapter support. Frozen compatibility fixtures must fail closed.
- Persisted backend facts can be mistaken for current availability. Inspection must distinguish planned, admitted, dispatched, and observed states.
- Changing a backend after unknown outcomes can duplicate effects. Recovery policy must preserve uncertainty.
