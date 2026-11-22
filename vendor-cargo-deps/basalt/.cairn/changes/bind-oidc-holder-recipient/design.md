## Context

Basalt's OIDC core admits a configured service audience but issuance previously reused it as the holder audience. Lattice needs a separate stable holder to sign each exact invocation without claiming that a requested recipient establishes possession. Once a holder can use the grant, the grant itself must carry the admitted service and project, and one OIDC token must yield one grant.

## Decisions

### Decision: Bind holder delivery to one exact canonical Mantle request

**Choice:** Add `exchange_to_recipient` taking a `HolderDelivery` (recipient `AudienceDid`, typed `mantle::Request`, `BasaltPolicyRef`). Before key load and replay, require subject = recipient, request audience and project = policy service audience and project (for both the Mantle request and the `ExchangeRequest`), the Mantle ability = the requested operation's ability, and `ExchangeRequest.resource` = `canonical_enforcement(request, policy_ref).resource`. Overwrite the plan's resource with that canonical resource and issue it to the recipient. Refuse a recipient equal to the issuing signer with `IssueFailure::Audience`. Keep the legacy `exchange` issuing to the policy audience. Bind recipient identities into success and failure receipts with domain-separated length framing.

**Rationale:** The receiver's `Verifier::enforcement` checks only its own generation, endpoint and audience; the account, project and target come from the holder-signed request, and UCAN resource matching is a byte prefix. A holder grant over a policy prefix would therefore cover any project, account, target or generation under it. Issuing exactly the canonical resource closes that, and makes truncated prefixes (issue #10) unreachable from this exchange. All checks are pure and run before any effect, so a refused request spends no replay key.

### Decision: One exchange per OIDC token

**Choice:** Key replay on `(iss, jti)` under `basalt.oidc.replay.v2`, and reserve it until `max(exp, iat + max_token_age_seconds) + clock_skew_seconds` via the new `IssuancePlan::replay_expires_at_seconds`.

**Rationale:** The documented contract was a one-time exchange keyed by `jti` (issue #7). Including request fields in the key allowed one exchange per distinct request, and reserving only for the UCAN lifetime allowed a repeat while the token stayed acceptable. The new reservation covers the full window in which `validate_time_claims` can accept the token. Old v1 reservations do not carry over; the token lifetime bounds that window.

### Decision: Reuse native asynchronous read actions without broadening defaults

**Choice:** Expose event-read and result-read in the closed OIDC operation vocabulary, using the exact native Mantle abilities. Admit explicit, unique, supported selections in Nickel; retain the existing generated default operation set unchanged.

**Rationale:** Events and result discovery are independent native read operations, not implied by submit, logs, status, cancellation or store-read. Operator policy must select each action. Uniqueness matches `parse_policy`, so a policy that passes the contract also loads.

## Risks / Trade-offs

- A typed recipient is not possession evidence; the receiving service must verify a holder-signed invocation before effects.
- Issuance uncertainty retains the consumed replay reservation exactly as before.
- Consumers that exchanged one OIDC token several times must now mint one token per exchange.
- Publication and product verification remain separate admissions. No release, lifecycle acceptance, archive, signing, remote mutation or production activation is authorized by this change.
