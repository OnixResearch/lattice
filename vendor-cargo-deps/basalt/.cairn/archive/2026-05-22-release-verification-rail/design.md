# Add release verification rail Design

## Context

The repo currently relies on several manual commands and has an environment pitfall around shared target directories. A usable crate needs one documented release-readiness rail for formatting, tests, lint, policy freshness, and packaging.

## Decisions

### 1. Package verification is mandatory

**Choice:** A crate is not consumer-ready unless `cargo package` or equivalent package validation passes.

**Rationale:** A crate is not consumer-ready unless `cargo package` or equivalent package validation passes.

### 2. Environment noise must be controlled

**Choice:** The rail should avoid the known shared target-dir/sccache failure mode rather than asking maintainers to debug phantom dependency errors.

**Rationale:** The rail should avoid the known shared target-dir/sccache failure mode rather than asking maintainers to debug phantom dependency errors.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
