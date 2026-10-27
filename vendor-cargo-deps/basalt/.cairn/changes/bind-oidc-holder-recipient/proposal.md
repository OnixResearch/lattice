## Why

OIDC service-policy audience and UCAN holder identity are independent. The existing exchange issues only to the configured service and cannot supply a holder-bound invocation client without replacing service attenuation.

## What Changes

Add `exchange_to_recipient` with an explicit typed holder DID while retaining the existing exchange contract. Keep service audience, project, operation, resource, expiration, issuer admission and one-time replay checks. Bind the recipient into redacted receipt capability identities. Receiver-side holder signatures remain mandatory; recipient selection proves no possession.

Expose the already-reviewed native Mantle event-read and result-read actions in OIDC policy selection. The Nickel contract accepts explicit supported operation selections; its shipped six-operation default remains unchanged and confers neither action implicitly.

Expose the native Mantle resource canonicalization as `canonical_enforcement(request, policy_ref)` so holders can construct the real signing request without fabricating or loading receiver policy/trust-root state. The verifier retains configured generation, endpoint and audience checks before sharing the same canonicalization.

## Impact

- **Files**: `src/oidc_exchange/{shell,issue,receipt,model,mod}.rs`, `src/mantle.rs`, `contracts/oidc-exchange/contracts.ncl`, `tests/oidc_exchange.rs`, `tests/ucan_enforcement/mantle.rs`.
- **Testing**: focused OIDC exchange verification must cover holder token audience, exact service attenuation, duplicate replay across recipients, distinct receipt identities and bearer redaction. Existing exchange callers remain compatible. Tests are authored but deliberately unexecuted pending the integrated consumer verification owner.
- **Publication**: this is a local source correction based on published Basalt `4c5c9e919b4573a7865559db2dc98a99c0c58144`, retaining UCAN `3a2aa043068a82663cf72ec05ed5d56a7d65ecd5`. No upstream publication, acceptance, archive or deployment is claimed.
