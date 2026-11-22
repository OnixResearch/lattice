## Why

OIDC service-policy audience and UCAN holder identity are independent. The existing exchange issues only to the configured service and cannot supply a holder-bound invocation client without replacing service attenuation.

A holder-usable grant raises two existing gaps to live risk. First, the exchange was not one-time (issue #7): it keyed replay on the whole request and reserved it only for the UCAN lifetime, so one OIDC token could be exchanged again for another operation or resource, or for the same request after the reservation lapsed. Second, a prefix-shaped resource could name another account or project, or end mid-field (issue #10), while the grant was signed to a caller-chosen holder.

## What Changes

Add `exchange_to_recipient` with a typed `HolderDelivery` while retaining the existing exchange contract. The delivery carries the recipient DID, the typed `mantle::Request` the holder will sign and the receiver's `BasaltPolicyRef`. Before key load and replay, the request subject must be the recipient, its audience and project must be the policy's, its ability must match the operation, and the requested resource must equal its canonical resource exactly. The UCAN carries exactly that resource. A recipient equal to the issuing signer is refused. Recipient identities are bound into redacted receipts. Receiver-side holder signatures remain mandatory; recipient selection proves no possession.

Make the exchange one-time per OIDC token. Key replay on the issuer and `jti` alone under a new domain tag, and reserve the key until `max(exp, iat + max_token_age) + skew`.

Expose the native Mantle event-read and result-read actions in OIDC policy selection. The Nickel contract accepts explicit, unique, supported operation selections; its shipped six-operation default remains unchanged and confers neither action implicitly. Fix the OIDC fixture helper so invalid fixtures exercise the contract instead of failing on infinite recursion.

Expose the native Mantle resource canonicalization as `canonical_enforcement(request, policy_ref)` so holders and the exchange can construct the real signing request without fabricating or loading receiver policy or trust-root state. The verifier retains configured generation, endpoint and audience checks before sharing the same canonicalization. Document that Mantle prefix grants must end at a field boundary.

## Impact

- **Files**: `src/oidc_exchange/{core,shell,issue,receipt,model,mod}.rs`, `src/mantle.rs`, `contracts/oidc-exchange/{contracts.ncl,fixtures/**}`, `flake.nix`, `tests/oidc_exchange.rs`, `tests/ucan_enforcement/mantle.rs`, `docs/oidc-exchange.md`, `README.md`, `CHANGELOG.md`.
- **API**: `exchange_to_recipient(token, now_seconds, policy, request, cache, documents, replay, issuer, signer, holder: &HolderDelivery<'_>)`. `IssuancePlan` gains `replay_expires_at_seconds`; `RejectReason` gains `CapabilityHolder`. Consumers mint a fresh OIDC token per exchange.
- **Upgrade**: reservations under the old `basalt.oidc.replay.v1` keys do not carry over; the token's own acceptance window bounds the exposure.
- **Testing**: see `tasks.md` for the commands that ran and their results.
- **Publication**: based on published Basalt `5e0f1f4cad925a50f7dd5cdbcae05a04e7b92463`, vendored UCAN `fca62a6ed6895689ff18112b497bc7a986516c20`. No upstream publication, acceptance, archive or deployment is claimed.
