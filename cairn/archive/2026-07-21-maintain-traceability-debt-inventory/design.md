## Context

The maintained Cairn coverage receipt reports 698 accepted requirements, 331 referenced requirements, 367 missing identifiers, zero dangling identifiers, and `valid: false`. The missing array is authoritative linkage debt, but not a semantic statement that implementation is absent. The first audited tranche contains six safe-inspection/recovery scenarios whose assertions are maintained under `tests/test_run_inspection.rs` and `tests/test_run_store_recovery.rs` but are currently linked only through broader parent markers.

## Goals

- Inventory every current missing identifier exactly once.
- Partition debt deterministically by namespace subsystem and explicit review state.
- Bind the missing set, exported review policy, rows, summaries, issues, and non-claims with BLAKE3.
- Fail closed on duplicate, stale, unsafe, malformed, or contradictory review data.
- Preserve the exact missing array, zero dangling markers, and unsuccessful coverage.

## Non-Goals

- Adding markers for existing missing requirements.
- Claiming that lexical similarity proves implementation or verification.
- Claiming that `unassessed` means behavior or evidence is absent.
- Proving semantic correctness, complete traceability, release readiness, or production readiness.
- Turning archived change material into maintained evidence.

## Portfolio Search Contract

**Exact goal:** produce a maintained, deterministic inventory of the 367 current missing identifiers without changing their linkage state.

**Completion evidence:** every baseline missing ID appears once; no other ID appears; the missing-set hash matches typed policy; the report validates its own inventory hash; six explicitly reviewed candidates carry safe relative evidence paths and current file-byte BLAKE3 hashes; all other rows are `unassessed`; the before/after missing arrays are equal; dangling remains zero; coverage remains invalid.

**False completion:** adding debt markers, inferring semantic evidence from token overlap, omitting or duplicating IDs, treating archive text as current evidence, describing unreviewed rows as absent behavior, or accepting a green process exit without deterministic row/hash checks.

**Budgets:** repository-owned accepted specs, coverage JSON, current evidence roots, and the exact two safe-inspection test files; no network retrieval; one implementation round plus one adversarial audit and validation round.

**Allowed outcomes:** validated inventory, exact fail-closed blocker, or user decision if semantic classification requires authority not present in maintained artifacts.

## Approach Registry

| Family | Mechanism | State | Reason |
|---|---|---|---|
| marker promotion | Add markers while inventorying | falsified | It changes the measured debt and can manufacture coverage. |
| lexical inference | Classify evidence from matching names/tokens | falsified | Similar names do not establish scenario assertions. |
| absence default | Mark every unreviewed row as absent | falsified | A bounded search cannot prove behavioral absence. |
| reviewed registry | Explicit typed decisions plus `unassessed` default | selected | It preserves uncertainty, supports exact candidate paths, and is mechanically complete. |

## Decisions

### Decision: typed Nickel owns review decisions

`cairn-policy/traceability-inventory.ncl` defines the supported classifications, expected missing count/hash, safe evidence paths, evidence-file BLAKE3 hashes, rationale codes, and explicit reviewed decisions. The Rust tool receives exported JSON, hashes the current non-symlink evidence files, and revalidates every invariant rather than trusting Nickel alone.

### Decision: pure core plus imperative shell

`scripts/traceability_inventory/core.rs` parses in-memory DTOs, validates review policy, derives sorted rows and summaries, and computes domain-separated BLAKE3 identities. `scripts/traceability-inventory.rs` only parses arguments, invokes Nickel export, reads/writes bounded files, and chooses the process exit status.

### Decision: uncertainty remains first-class

The report supports `exact_maintained_evidence`, `partial_evidence_only`, `no_exact_evidence_identified`, and `unassessed`. Only explicit policy rows may select the first three. Missing identifiers without reviewed policy remain `unassessed`; this is not evidence absence.

### Decision: JSON is the generated receipt format

Nickel remains the human-authored policy. JSON is used only for the Cairn coverage input and deterministic generated inventory because those tools require machine interchange. The generated report is checked in under `cairn-policy/generated/` and is not a Tracey evidence source.

### Decision: baseline identity is the sorted missing set

The core hashes a domain tag followed by each sorted identifier's checked byte length and bytes. Policy pins this BLAKE3 value and exact count. Coverage receipt input/receipt/policy hashes remain provenance fields but are not substituted for missing-set identity.

## Risks and Mitigations

- **Risk:** a reviewed candidate is promoted into a coverage claim. **Mitigation:** the inventory contains no `r[...]` marker and labels candidates separately from Tracey linkage.
- **Risk:** report or reviewed evidence drift after source changes. **Mitigation:** the tool has `generate` and `check` modes; the checked report pins coverage, missing-set, and reviewed evidence-file identities.
- **Risk:** malformed policy paths disclose or escape repository scope. **Mitigation:** reject absolute, parent, empty, and non-normal relative paths before report construction.
- **Risk:** self-hash instability. **Mitigation:** hash a typed payload that excludes only the final `inventory_hash` field.

## Validation

- Evaluate the Nickel policy and reject a malformed negative fixture.
- Run positive and negative Rust core/shell tests.
- Generate then check the inventory byte-for-byte.
- Compare the exact ordered before/after missing arrays and dangling arrays.
- Run policy freshness, repository checks, Cairn gates, and `nix flake check -L --option secret-key-files ''`.
