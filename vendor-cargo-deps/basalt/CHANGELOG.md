# Changelog

## Unreleased

### Added

- `oidc_exchange::exchange_to_recipient` issues the UCAN to a holder rather
  than to the policy's Mantle service. It takes a `HolderDelivery`: the
  recipient `AudienceDid`, the typed `mantle::Request` the holder will sign,
  and the receiver's `BasaltPolicyRef`. Before key load and replay it requires
  that:
  - the request's subject is the recipient;
  - its audience and project are the policy's;
  - its ability matches the operation;
  - `ExchangeRequest.resource` equals the request's canonical resource.

  The UCAN carries exactly that resource. A subject that is not the recipient
  is denied with `RejectReason::CapabilityHolder` (`capability-holder`). A
  recipient equal to the issuing signer is denied with `IssueFailure::Audience`.
  The legacy `exchange` still issues to the policy's service audience.
- `mantle::canonical_enforcement(request, policy_ref)` builds the canonical
  signing request without receiver policy or trust roots.
  `Verifier::enforcement` checks its deployment bindings and then uses it.
- `MantleOperation::{EventRead, ResultRead}` can be selected explicitly in OIDC
  policy. The shipped default operation set is unchanged.

### Changed

- The OIDC exchange is one-time per OIDC token (issue #7). The replay key now
  covers only the token issuer and `jti`, under the new domain tag
  `basalt.oidc.replay.v2`. A second exchange of the same token is a duplicate,
  even for another operation, resource or holder. Consumers mint a fresh OIDC
  token for each exchange.
- The shell reserves the replay key until
  `max(exp, iat + max_token_age_seconds) + clock_skew_seconds`, and no longer
  only until the issued UCAN expires. `IssuancePlan` gains
  `replay_expires_at_seconds`, which is the value passed to `ReplayPort::commit`.
- The OIDC Nickel contract rejects a duplicated capability operation, as
  `parse_policy` already did. New fixtures cover:
  - an explicit `event-read` and `result-read` selection;
  - an unsupported operation;
  - an empty operation list;
  - a duplicated operation.

### Fixed

- The invalid OIDC policy fixtures failed because their helper recursed
  infinitely, not because the contract rejected them. Nickel record fields are
  recursive, so `algorithm = algorithm` referred to the field itself. The helper
  now gives its parameters names that differ from the fields, and the policy
  contract rejects every invalid fixture.

### Documented

- Mantle prefix grants must end at a field boundary, because `generation` has
  no length prefix (issue #10).

### Upgrade notes

- Reservations made under the old `basalt.oidc.replay.v1` keys do not carry
  over. A token exchanged just before the upgrade can be exchanged once more
  under the new key. The token's own lifetime bounds that window: verification
  rejects it after `max(exp, iat + max_token_age_seconds) + clock_skew_seconds`.
  Under the shipped policy that is 630 seconds after `iat` for a token that
  expires within its 600-second age limit.
