# Steel Contract Backend Design

## Context

The crate's current value is a consumable Rust/Nickel contract surface for UCAN-based authorization. Nickel is strong for static policy/config contracts and generated JSON artifacts. Steel Scheme is useful for dynamic predicates and higher-order policy logic, but only if UCAN talks to it through the same reviewed contract discipline: typed DTOs, no ambient authority, deterministic receipts, and fail-closed validation.

## Decisions

### 1. Shared backend-neutral contract envelope

**Choice:** Introduce a shared contract envelope that names the backend (`nickel` or `steel`), contract id/version, normalized source hash, expected input schema, expected output schema, and receipt schema version.

**Rationale:** UCAN should not special-case Nickel or Steel at the authority layer. A common envelope lets Rust validate backend identity, source binding, and receipt shape before any backend-specific evaluation is trusted.

### 2. Steel is a sibling backend, not a replacement for Nickel

**Choice:** Model Steel as a sibling backend/adapter surface, while preserving Nickel as the static policy and generated-artifact backend.

**Rationale:** Nickel and Steel serve different roles. Nickel is best for reviewed static policy data and export freshness. Steel is best for dynamic predicates/callables. Treating Steel as a sibling avoids coupling all contracts to an interpreter and avoids weakening Nickel's artifact-freshness story.

### 3. Rust owns authority and effects

**Choice:** Steel evaluation requests carry bounded DTOs only. Steel returns structured decisions/evidence only. Rust validates UCAN capabilities, backend policy, source hashes, schema compatibility, and receipt hashes before trusting a decision; Steel receives no ambient filesystem, process, network, credential, or mutation authority.

**Rationale:** UCAN authority is the product boundary. Scheme code may express predicates, but it must not become an unreviewed host-effect channel.

### 4. Higher-order contracts are explicit callable descriptors

**Choice:** Add higher-order contract descriptors for callable/predicate values: stable callable id, arity, argument contract list, return contract, required UCAN capabilities, determinism/effect class, source/content hash, and redaction class.

**Rationale:** Steel naturally represents higher-order logic. UCAN can safely reason about it only when callables are declared, hash-bound, capability-bound, and receipt-producing instead of opaque closures.

### 5. Fail-closed compatibility and fixtures

**Choice:** Unsupported backend ids, missing hashes, missing callable descriptors, effectful callables without declared authority, and malformed Steel receipts must produce deterministic deny/error receipts.

**Rationale:** Downstream consumers need stable failure behavior when they opt into Steel. The crate should prove both Nickel compatibility and Steel denial paths before implementing broad evaluation support.

## Risks / Trade-offs

- Adding Steel too deeply could blur this crate's contract role with downstream runtime/orchestration roles. Keep implementation behind DTOs and backend traits.
- Higher-order descriptors add schema complexity. Keep v1 narrow: predicate/callable contracts only, no general host function registry.
- Steel source normalization and hashing need a stable definition before receipts are considered durable. Treat hash and normalization rules as part of the public contract.
- Runtime sandbox claims are easy to overstate. The requirements should claim deny-by-default DTO authority, not full sandbox security.
