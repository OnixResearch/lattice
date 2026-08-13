# Stabilize public library API Design

## Context

Other applications need a clear, documented, semver-conscious API instead of relying on incidental public fields and minimal examples.

## Decisions

### 1. Semver flexibility over raw-field convenience

**Choice:** Prefer constructors and documented access patterns where direct field exposure would make later policy evolution breaking.

**Rationale:** Prefer constructors and documented access patterns where direct field exposure would make later policy evolution breaking.

### 2. Receipts stay deterministic

**Choice:** Any API additions must preserve stable receipt semantics for allow/deny outcomes.

**Rationale:** Any API additions must preserve stable receipt semantics for allow/deny outcomes.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
