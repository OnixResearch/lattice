# Add consumer integration tests Design

## Context

Unit tests prove local logic, but applications need evidence that the crate works when consumed through its public API and configured features.

## Decisions

### 1. Tests should use the public surface only

**Choice:** Consumer tests must not reach into private modules or rely on repository-local implementation details.

**Rationale:** Consumer tests must not reach into private modules or rely on repository-local implementation details.

### 2. Negative tests are first-class

**Choice:** Deny/error paths are required because application integrators need fail-closed behavior evidence.

**Rationale:** Deny/error paths are required because application integrators need fail-closed behavior evidence.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
