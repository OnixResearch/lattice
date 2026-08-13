# Complete package metadata Design

## Context

Downstream users and registries need stable crate metadata to identify source, docs, license, package purpose, and discoverability. Cargo currently warns that key package metadata is missing.

## Decisions

### 1. Metadata must not overclaim maturity

**Choice:** Use accurate discoverability fields that describe UCAN+Nickel policy enforcement without implying production certification.

**Rationale:** Use accurate discoverability fields that describe UCAN+Nickel policy enforcement without implying production certification.

### 2. Repository URL is part of the public contract

**Choice:** Point package metadata at the authoritative source remote so consumers can audit and vendor the crate.

**Rationale:** Point package metadata at the authoritative source remote so consumers can audit and vendor the crate.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
