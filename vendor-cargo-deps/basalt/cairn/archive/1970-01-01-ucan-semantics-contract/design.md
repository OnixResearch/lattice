## Context

Basalt is intentionally small: Nickel owns policy shape, Rust owns runtime enforcement, and UCAN owns token/capability semantics. The current dependency pin gives reproducible bytes but not a reviewer-readable contract for the semantics Basalt expects.

## Design

### Semantics contract

Add a Nickel-authored contract and fixture matrix that models Basalt's consumed UCAN behavior as data:

- resource and ability strings;
- requested operation;
- direct and delegated capability grants;
- caveat policy availability and outcome;
- proof traversal status;
- replay admission status;
- expected Basalt enforcement result and reason class;
- required non-claims.

### Runtime adapter check

Basalt's Rust tests should load exported semantics fixtures and run them through the existing enforcement core. The pure core takes parsed policy, request, and UCAN decision facts; the CLI/test shell handles JSON/Nickel export and file reads.

### UCAN compatibility boundary

The contract names the UCAN semantic version or manifest digest when available. If UCAN later publishes an authorization-semantics manifest, Basalt should bind fixtures to that digest instead of only a Git revision.

## Alternatives

### Rely only on end-to-end token tests

Rejected. End-to-end token tests are useful, but they do not isolate the policy/authorization semantic boundary or make fixture drift reviewable.

### Reimplement UCAN authorization in Basalt contracts

Rejected. Basalt should not duplicate UCAN semantics. It should consume UCAN decisions and verify the compatibility envelope around those decisions.

## Risks

- **Contract duplicates UCAN behavior**: keep the contract at the adapter boundary, not as a second implementation.
- **Fixture sprawl**: group fixtures by behavior class and keep mandatory positive/negative coverage small.
- **Overclaiming**: require non-claims for backend freshness, key resolution, cryptography, and application policy completeness.
