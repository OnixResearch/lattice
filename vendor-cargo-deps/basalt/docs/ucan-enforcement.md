# UCAN enforcement adapter

## Purpose

`basalt::ucan_enforcement` verifies one raw UCAN holder invocation for one exact Basalt request.

Success returns `LiveUcanEnforcement`. This value is the live in-process admission capability.

A denial returns `UcanEnforcementFailure`. The failure contains a typed reason and redacted evidence only.

## Required inputs

The caller supplies these explicit inputs:

- a loaded Basalt policy and its trusted policy reference
- one normalized `EnforcementRequest`
- the raw compact token
- one closed `EnforcementAction`
- the holder nonce and signature
- key, proof, revocation, caveat, and replay services
- verification time and explicit limits

The normalized request must not contain capability grants. The adapter creates verified authority facts after UCAN authorization succeeds.

## Bearer ownership

Keep the compact bearer in its zeroizing `CompactToken` owner. Pass only borrowed `CompactToken::as_str()` views to signing and verification.

Do not copy a compact bearer into `String` or `Vec` values. Do not put bearer text in arguments, logs, receipts, or serde values.

Test failures must use wrong keys, signatures, requests, or injected services. Tests must not mutate copied bearer text.

## Request signing

Call `bind_enforcement_action` before holder signing. It maps one closed action to a contract, ability, operation profile, and request digest.

The profile and request digest use domain-separated BLAKE3. Each variable field has a fixed `u64` length prefix.

Use the returned `OperationBinding` in `HolderInvocationSigningRequest`. The signature then covers the token, audience, resource, ability, operation, request binding, and nonce.

A token, request, action, profile, or nonce substitution invalidates the invocation.

## OnixOS target actions

The closed `EnforcementAction` enum adds six OnixOS target actions. Each action has one reviewed contract, one ability, and one resource prefix.

| Action | Contract | Ability | Resource prefix |
|---|---|---|---|
| Candidate activation | `onixos-target-candidate-activation` | `onix/target-candidate-activation` | `did:key:z6Mkonix/onixos/` |
| Generation persistence | `onixos-target-generation-persistence` | `onix/target-generation-persistence` | `did:key:z6Mkonix/onixos/` |
| Rollback | `onixos-target-rollback` | `onix/target-rollback` | `did:key:z6Mkonix/onixos/` |
| State-sync publish | `onixos-state-sync-publish` | `state_sync.publish` | `onix://state-sync/` |
| State-sync delete | `onixos-state-sync-delete` | `state_sync.delete` | `onix://state-sync/` |
| State-sync admin | `onixos-state-sync-admin` | `state_sync.admin` | `onix://state-sync/` |

The adapter has no generic action constructor. A caller can select only a reviewed enum variant. Each variant has a distinct BLAKE3 operation-profile identity, and the request-binding digest includes the action, contract, resource, and ability.

`bind_enforcement_action` refuses a request whose resource falls outside the action's reviewed prefix. State-sync publish does not authorize delete or administrative authority. A target mutation is never mapped to a document action.

A live result for one target action grants only the exact mapped request. It does not execute an effect and does not prove effect correctness.

## UCAN source export

These actions depend on the checked-in reviewed UCAN snapshot under `vendor/ucan/`.

The Basalt flake exports that snapshot as `lib.ucanSource`. A downstream consumer that pins one immutable Basalt revision evaluates `basalt.lib.<system>.ucanSource` to name the checked-in vendor tree for offline packaging.

The existing revision, BLAKE3 manifest, and license checks in `tools/sync-vendored-ucan.rs` remain authoritative. A successful result does not claim that an unpushed Basalt revision is remotely available and does not recreate a remote UCAN pin.

## Verification order

The adapter uses this order:

1. Check token and request bounds.
2. Bind the typed action to the exact request.
3. Verify the compact token with the injected services.
4. Run non-consuming capability and caveat authorization.
5. Build and evaluate exact Basalt authority facts.
6. Build the allowed redacted receipt.
7. Run fresh holder-signed verification with the same services.
8. Let UCAN call replay once as its final commit.
9. Move the prebuilt values into `LiveUcanEnforcement`.

Key, proof, revocation, and caveat services can receive repeated calls. These services must not consume state.

Replay admission is the only consuming service. A Basalt policy denial makes no replay call.

After replay succeeds, Basalt performs no allocation, hashing, policy work, or other fallible work.

## Live authority

`LiveUcanEnforcement` borrows the exact `EnforcementRequest`. It privately owns these values:

- the final UCAN holder authorization
- the prebuilt Basalt `AuthorityDecision`
- the prebuilt redacted receipt

The type has no public constructor. It does not implement `Clone`, `Serialize`, or `Deserialize`.

The prebuilt `AuthorityDecision` has no public getter. Callers cannot copy or serialize the allowed decision from the live wrapper.

Use the live value during the admitted in-process operation. Do not treat a receipt as live authority.

## Denial reasons

`UcanEnforcementReason` gives stable fail-closed classes. Important classes include:

- `malformed-credential`
- `oversized-credential`
- `signature-denied`
- `proof-denied`
- `key-resolution-denied`
- `holder-denied`
- `time-denied`
- `caveat-denied`
- `scope-denied`
- `revoked`
- `replay-denied`
- `limit-exceeded`
- `operation-mismatch`
- `policy-denied`
- `backend-unavailable`

Missing or malformed proof data maps to `proof-denied`. Stale or unavailable verification services map to `backend-unavailable`.

Canonical field-size errors map to `limit-exceeded`. Invalid holder signatures map to `holder-denied`.

A failure exposes no `AuthorityDecision` and no live capability.

## Receipts

Each outcome includes a deterministic BLAKE3 receipt. Its hash uses a separate receipt domain.
It frames each component with a big-endian `u64` length.
Optional values use a framed presence tag and a fixed-width payload.
The receipt includes these redacted facts:

- the stable reason and allowed state
- the observed compact-token length
- the bounded compact-token digest, when permitted
- the exact request-binding digest
- the reviewed operation-profile digest
- the trusted policy digest
- the reviewed UCAN revision
- explicit non-claims

An oversized token is rejected before token hashing. Its receipt records the input length but omits token identity.

Receipts omit tokens, proof bytes, keys, caveat payloads, identity text, and scope text. A serialized receipt cannot construct `LiveUcanEnforcement`.

## Bounded claim

Success proves that the supplied services admitted one exact holder-signed request under the supplied time, limits, and Basalt policy.

The result does not prove these properties:

- key discovery or key custody policy
- global revocation freshness
- durable or cross-process replay storage
- transport security or caller identity
- upstream request construction correctness
- application policy truth
- downstream effect correctness
- release eligibility or whole-system security

## Focused checks

```bash
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-ucan-enforcement-target RUSTC_WRAPPER= cargo test --test ucan_enforcement --no-default-features
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-ucan-enforcement-target RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings
```
