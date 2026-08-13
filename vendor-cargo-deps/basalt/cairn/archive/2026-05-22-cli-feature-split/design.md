# Split CLI dependencies behind a feature Design

## Context

Applications pulling the library should not be forced to compile or audit CLI-only dependencies. The crate currently exposes a library but always depends on `clap` for the binary.

## Decisions

### 1. Library first for downstream apps

**Choice:** Default dependency use should favor the library surface; the CLI remains available explicitly for operators.

**Rationale:** Default dependency use should favor the library surface; the CLI remains available explicitly for operators.

### 2. No CLI behavior change in the split

**Choice:** The split should be dependency-surface work, not a command redesign.

**Rationale:** The split should be dependency-surface work, not a command redesign.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
