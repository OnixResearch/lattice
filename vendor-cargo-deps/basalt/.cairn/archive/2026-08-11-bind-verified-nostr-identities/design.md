# Design: Bind verified Nostr identities to Basalt subjects

## Goals

- Map verified external keys to stable policy subjects.
- Keep authentication separate from authorization.
- Make rotation, revocation, ambiguity, and policy drift explicit.
- Keep network, clock, signer, and replay effects outside Basalt.

## Policy source

Nickel remains the human-authored policy source. Add a versioned identity-binding contract with stable subjects and typed authenticators.

A Nostr authenticator contains:

- a stable binding identifier;
- the canonical 32-byte x-only public key in lowercase hexadecimal;
- a closed lifecycle state;
- optional non-secret display metadata;
- required non-claims.

The generated JSON remains a deterministic runtime input. Rust parses and validates the complete policy before subject resolution.

## Stable subject model

A subject identifier is Basalt policy identity. It is not a Nostr key, NIP-05 name, DID, display name, or UCAN grant.

A subject can have multiple authenticators for rotation. Only active authenticators can satisfy a new request.

One active Nostr key cannot map to multiple subjects. Duplicate, ambiguous, empty, malformed, or conflicting bindings fail before enforcement.

Retired bindings remain visible for history but cannot authenticate. Revoked bindings deny and preserve their revocation disposition.

## Input boundary

Basalt accepts a typed `VerifiedNostrCredential` from the exact reviewed UCAN adapter contract.

Basalt does not accept caller-supplied `signature_valid`, `authenticated`, or equivalent booleans. It verifies adapter profile identity and required request bindings.

The core receives policy, credential, current policy identity, and explicit lifecycle facts as values. It performs no network, DNS, clock, signer, or replay-store effect.

## Subject resolution

The pure core performs these steps:

1. Validate the complete identity-binding policy.
2. Validate the UCAN adapter schema and profile identity.
3. Find the exact canonical Nostr key binding.
4. Reject missing, duplicate, ambiguous, inactive, or revoked bindings.
5. Bind the credential audience, purpose, challenge, and request to the expected application request.
6. Construct `AuthenticatedSubject` only after all checks pass.

`AuthenticatedSubject` contains the stable subject ID, binding ID, credential event identity, policy identity, request identity, and non-claims.

## Authorization composition

Authentication and authorization remain separate inputs.

An application can require both:

- `AuthenticatedSubject` for identity attribution;
- an existing Basalt UCAN enforcement result for the exact resource and ability.

A pure composition function verifies that both results bind the same request and configured subject policy. It cannot widen either result.

Nostr authentication alone always produces a non-authorizing result. A UCAN allow for another subject or request cannot compose.

## Receipts

Authentication and composition receipts use BLAKE3 over versioned canonical material. They retain the external Nostr SHA-256 event ID under its protocol role.

Receipts can include public subject, binding, request, decision, reason, policy, and adapter identities. They exclude credentials, private keys, proof bytes, challenges, and secret metadata.

## Verification

Positive fixtures cover one active key, rotated active keys, exact request composition, and stable canonical ordering.

Negative fixtures cover malformed keys, duplicate subjects, ambiguous keys, inactive keys, revoked keys, stale policy, forged adapter facts, and request mismatch.

Authority fixtures prove that authenticated subjects cannot bypass missing, denied, stale, or differently scoped UCAN authorization.
