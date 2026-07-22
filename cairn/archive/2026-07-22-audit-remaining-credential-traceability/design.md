## Context

The maintained inventory contains 341 missing IDs and zero dangling markers. Exactly 14 unassessed rows remain in the `credentials` subsystem after the core 12-ID promotion. Existing parent or child markers, archived task claims, implementation names, and lexical similarity are not evidence for these rows.

## Goals

- Classify exactly the 14 remaining credential rows.
- Bind exact and partial decisions to safe current repository paths and file-byte BLAKE3 hashes.
- Preserve the exact 341-item missing set and zero dangling markers.
- Fail closed on row, classification, rationale, evidence, ordering, duplication, marker, or coverage drift.

## Non-Goals

- Changing credential behavior.
- Adding any of the 14 credential markers.
- Treating partial evidence as near-complete correctness.
- Inferring absence from `no_exact_evidence_identified` or `unassessed`.
- Claiming n8n parity, arbitrary secret confidentiality, complete traceability, release readiness, or production readiness.

## Portfolio Search Contract

**Exact goal:** produce a complete conservative classification of the 14-row remaining credential tranche without changing behavior or linkage.

**Observable completion evidence:** one row per allowlisted ID; no additional credential row; exact classification/rationale/evidence paths; current file-byte hashes; baseline/current missing sets equal 341; zero dangling; both receipts invalid; no selected marker in maintained source/test roots; deterministic positive and negative tests.

**False completion:** trusting parent/child markers, archived evidence, generated inventory, implementation shape for user-visible or persisted claims, a binary exact/absent partition, stale file hashes, changed missing IDs, added credential markers, or green global coverage.

**Audit risks:** n8n fixtures intentionally containing a rejection sentinel; cross-test inference replacing a direct boundary observation; direct provider tests being promoted to execution-boundary claims; object construction being treated as rendering/persistence; and safe omission being treated as retained safe context.

**Budgets:** accepted credential spec; generated inventory; repository-owned `src`, `tests`, and `examples` evidence roots; four mechanism-family passes; one local advisory review; one adversarial synthesis; one focused audit suite; normal repository/Cairn/Octet/Nix validation.

**Allowed outcomes:** validated classification, exact blocker, exhausted audit, or user decision if accepted clauses conflict irreducibly.

## Approach Registry

| Family | Mechanism | Finding | State |
|---|---|---|---|
| marker/archive trust | infer evidence from related markers or completed changes | rejected because evidence is not attached to the reviewed ID and archives are not maintained authority | falsified |
| implementation shape | infer complete behavior from mapping/provider/webhook code | useful partial evidence but insufficient for export, persistence, rendering, and multi-surface clauses | blocked for exact claims |
| direct assertions | compare maintained observations to every scenario clause | supplies the strongest bounded partition and exact gaps | validated |
| binary exact/absent | collapse all non-exact rows into no-exact | rejected because 13 rows have concrete maintained subset evidence | falsified |

## Classification Decision

The cookbook parent row is exact because maintained tests directly inspect its workflow/docs references and accepted/rejected smoke surfaces. The other 13 rows are partial: each has maintained implementation or assertions, but at least one accepted boundary observation is absent. No row is classified no-exact because every row has concrete maintained subset evidence.

The local advisory review recommended collapsing every row to no-exact after noticing the parity sentinel. That recommendation was rejected because it discarded concrete source/assertion evidence and did not preserve the policy distinction between partial and no-exact. Deterministic repository artifacts remain authoritative.

## Functional Core and Shell

`tests/traceability_review/core.rs` remains the pure comparator over in-memory review rows and coverage snapshots. The focused test shell only decodes checked-in JSON and hashes `include_str!` source bytes. Inventory generation owns bounded filesystem access and Nickel export.
