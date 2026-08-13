## Why

Basalt's policy artifact workflow is structurally correct: people author `contracts/policy.ncl`, the release rail checks the generated JSON for freshness, and Rust performs runtime enforcement. The Nickel contract itself remains shallow. Schema version, backend ids, contract ids, resource prefixes, and abilities are mostly non-empty strings; arrays may be empty or duplicate values; map keys and embedded ids are not source-validated as one whole policy; and backend metadata is not constrained to the Rust-supported vocabulary.

Rust tests catch some malformed external JSON, and the active `iddqd-policy-identity-maps` package will strengthen runtime identity-map admission. The authoring source can still export ambiguous or unsupported policy and discover the problem only after projection. That is a distinct layered gap: Nickel should reject invalid reviewed intent, while Rust must continue rejecting untrusted projections.

## What Changes

- Add a reusable Basalt Nickel policy prelude for exact schema literals, supported backend ids, normalized identifiers, non-empty unique collections, and deterministic whole-policy diagnostics.
- Harden `contracts/policy.ncl` so contract/backend map keys align with embedded ids, required maps and allowlists are non-empty, exact duplicates are rejected, and backend metadata matches the supported Rust vocabulary.
- Add lexical resource-prefix and ability contracts that match Rust/UCAN parser preconditions without reimplementing UCAN containment, wildcard, caveat, proof, replay, or authorization semantics in Nickel.
- Extend policy artifact checks with contract/source/projection/Rust DTO parity and positive/negative Nickel fixtures.
- Preserve the packaged generated JSON and public parse/enforcement API shape unless a schema-versioned migration is explicitly reviewed.

## Impact

- **Authoring**: `contracts/policy.ncl`, new shared policy contracts, and source-level fixtures.
- **Generated artifact**: existing freshness remains; parity evidence becomes structural and DTO-aware.
- **Runtime**: `parse_policy_json` and the iddqd identity-map conversion remain fail-closed for external artifacts even if they did not originate in Nickel.
- **Active-change boundary**: `iddqd-policy-identity-maps` owns runtime map identity, `ucan-semantics-contract` owns UCAN compatibility behavior, and `ucan-basalt-authority-core` owns composed authority DTOs. This package owns only Nickel source shape and projection parity.
- **Claims**: source contract success does not authorize a request, validate a token, prove backend behavior, or establish runtime authority.
