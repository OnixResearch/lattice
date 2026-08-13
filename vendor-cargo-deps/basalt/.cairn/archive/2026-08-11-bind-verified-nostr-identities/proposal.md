# Proposal: Bind verified Nostr identities to Basalt subjects

## Why

The planned UCAN Nostr adapter proves control of one Nostr key for one request. It does not map that key to a stable application subject.

Basalt owns reviewed Nickel policy and runtime enforcement boundaries. It is the correct layer for deterministic subject bindings and authorization composition.

## What Changes

- Add typed Nickel policy for stable subjects and external authenticators.
- Accept only `VerifiedNostrCredential` values from the reviewed UCAN adapter boundary.
- Resolve one canonical Nostr key to one active stable subject.
- Separate Nostr authentication from UCAN capability authorization.
- Support explicit key activation, retirement, revocation, and policy-version binding.
- Emit redacted BLAKE3-bound authentication and composition receipts.
- Add positive and negative policy, identity, rotation, and authority fixtures.

## Impact

Applications can identify a stable subject without treating a Nostr key as permanent account identity. Key rotation does not change the subject identifier.

A resolved subject gains no ability by itself. Existing Basalt and UCAN checks still control each requested resource and ability.

## Dependencies

Implementation depends on the reviewed `VerifiedNostrCredential` contract from `OnixResearch/ucan` change `add-nostr-identity-adapter`.

The active `iddqd-policy-identity-maps` change remains separate. This change can reuse its deterministic map boundary after that work is accepted.

## Non-goals

- Do not verify Nostr signatures, contact relays, resolve NIP-05 names, or operate NIP-46 sessions in Basalt.
- Do not mint UCAN credentials from a Nostr signature.
- Do not convert a Nostr public key into a DID without a separate reviewed standard.
- Do not let subject metadata, key possession, or a successful binding grant authority.
