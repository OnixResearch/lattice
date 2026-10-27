# Preserve AGPL license boundary Design

## Context

The crate is intended to remain AGPL-3.0-or-later. That license choice must be explicit in package metadata and downstream documentation so applications understand the adoption boundary.

## Decisions

### 1. AGPL is retained

**Choice:** Do not change the license to improve proprietary adoption; make the boundary explicit instead.

**Rationale:** Do not change the license to improve proprietary adoption; make the boundary explicit instead.

### 2. No legal overclaims

**Choice:** Docs should identify the crate license but not provide legal advice about whether a particular application can use it.

**Rationale:** Docs should identify the crate license but not provide legal advice about whether a particular application can use it.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
