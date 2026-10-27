# Use remote UCAN dependency Design

## Context

The crate cannot be packaged or consumed outside this checkout while `ucan` is path-only. Consumers need Cargo to resolve UCAN from the authoritative remote rather than from `../ucan`.

## Decisions

### 1. Remote is authoritative for consumers

**Choice:** Use the UCAN repository remote as the dependency source for this crate's public consumability path; any sibling checkout use must be an explicit local override, not the published package contract.

**Rationale:** Use the UCAN repository remote as the dependency source for this crate's public consumability path; any sibling checkout use must be an explicit local override, not the published package contract.

### 2. Package verification is the release gate

**Choice:** Treat `cargo package` as the proof that downstream applications can resolve the dependency surface without the current repository layout.

**Rationale:** Treat `cargo package` as the proof that downstream applications can resolve the dependency surface without the current repository layout.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
