# Clarify policy artifact consumption Design

## Context

The repository has Nickel policy source and an ignored generated JSON artifact. Downstream applications need a deterministic answer for whether they consume generated policy JSON, generate it themselves, or embed a sample.

## Decisions

### 1. Generated policy is explicit, not accidental

**Choice:** If generated JSON is packaged, it must be intentionally included; if not, docs must state that consumers generate or supply policy JSON.

**Rationale:** If generated JSON is packaged, it must be intentionally included; if not, docs must state that consumers generate or supply policy JSON.

### 2. Nickel source remains reviewable authority

**Choice:** Policy shape stays authored in Nickel with Rust loading the deterministic JSON export.

**Rationale:** Policy shape stays authored in Nickel with Rust loading the deterministic JSON export.

## Risks / Trade-offs

- Scope is intentionally planning-only in this Cairn; implementation must preserve current passing enforcement behavior unless a requirement explicitly changes it.
- Downstream-consumability changes can expose packaging blockers in the sibling `ucan` crate; those blockers should be reported separately if they are outside this repo.
