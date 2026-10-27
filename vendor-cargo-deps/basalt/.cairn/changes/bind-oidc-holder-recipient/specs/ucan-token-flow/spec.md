## ADDED Requirements

### Requirement: OIDC service audience and holder recipient remain independent

Explicit OIDC operation selections MAY include the native Mantle event-read and result-read actions. Shipped default policy MUST NOT acquire these actions implicitly.
r[ucan_token_flow.oidc.holder_recipient] Basalt MUST support explicit typed holder delivery after admitting the original service audience, project, operation, resource and lifetime. Recipient selection MUST NOT claim holder possession, bypass issuer admission, or permit a duplicate exchange by changing holders. Metadata-only receipt capability identities MUST bind the recipient without including bearer tokens. Existing exchange consumers MUST retain their original issuance semantics.

#### Scenario: Holder delivery preserves attenuation
- GIVEN an accepted OIDC token, admitted service policy and explicit holder DID
- WHEN the recipient exchange issues a UCAN
- THEN the signed token audience is the holder and the exact resource, operation and expiration remain bounded by the service policy

#### Scenario: Holder substitution cannot bypass service policy or replay
- GIVEN a request already exchanged to one holder or a request replacing the service audience with the holder
- WHEN another exchange is attempted
- THEN replay or service audience admission denies before another UCAN is issued

### Requirement: Shared native Mantle signing canonicalization
r[ucan_token_flow.oidc.native_signing] Basalt MUST expose pure Mantle canonicalization using an explicit request and policy artifact reference without requiring fabricated receiver policy or trust roots. The native verifier MUST retain operator-configured generation, endpoint and audience admission before invoking this canonicalization. Canonicalization MUST preserve exact length framing, all signed request fields, nonzero generation and per-field/aggregate bounds; it MUST NOT confer authority.

#### Scenario: Holder and receiver share one signed representation
- GIVEN an explicit native Mantle request and operator-pinned policy reference
- WHEN a holder constructs its signing resource and the native receiver admits the invocation
- THEN the actual signature MUST cover the same canonical fields and request substitutions MUST fail before replay consumption

