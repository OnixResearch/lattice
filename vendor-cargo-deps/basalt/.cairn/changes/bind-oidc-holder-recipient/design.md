## Context

Basalt's OIDC core admits a configured service audience but issuance previously reused it as the holder audience. Lattice needs a separate stable holder to sign each exact invocation without claiming that a requested recipient establishes possession.

## Decisions

### Decision: Preserve the existing API and add explicit recipient delivery

**Choice:** Add a typed `AudienceDid` argument only on `exchange_to_recipient`. Share all original admission, document loading, attenuation and replay code. Select the token recipient only after the original plan is admitted and replay reserved. Preserve the original replay identity so changing holders cannot issue a second token for one request. Bind recipient identities into both success and failure receipts using domain-separated length framing.

**Rationale:** A new explicit operation preserves existing service-audience issuance consumers while making the independent delivery binding visible. No policy is rewritten to pretend the holder is the service, and no local authorization or network checks are bypassed.

### Decision: Reuse native asynchronous read actions without broadening defaults

**Choice:** Expose event-read and result-read in the closed OIDC operation vocabulary, using the exact native Mantle abilities. Admit explicit supported selections in Nickel; retain the existing generated default operation set unchanged.

**Rationale:** Events and result discovery are independent native read operations, not implied by submit, logs, status, cancellation or store-read. Operator policy must select each action.

## Risks / Trade-offs

- A typed recipient is not possession evidence; the receiving service must verify holder-signed invocation before effects.
- Issuance uncertainty retains the consumed replay reservation exactly as before.
- Publication and product verification remain separate admissions. No release, lifecycle acceptance, archive, signing, remote mutation or production activation is authorized by this change.
