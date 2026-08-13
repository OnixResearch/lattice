# Tasks: Bind verified Nostr identities to Basalt subjects

## Policy and subject core

- [x] [serial] Wait for the reviewed UCAN `VerifiedNostrCredential` contract and pin its exact source. r[nostr_identity_bindings.verified_input]
- [x] [serial] Add typed Nickel subject and authenticator contracts with generated artifact freshness. r[nostr_identity_bindings.policy_contract]
- [x] [serial] Add pure canonical-key indexing, subject resolution, and deterministic diagnostics. r[nostr_identity_bindings.subject_resolution]
- [x] [parallel] Add valid single-key, rotated-key, and canonical-order fixtures. r[nostr_identity_bindings.final_validation]
- [x] [parallel] Add malformed, duplicate, ambiguous, inactive, revoked, and stale-policy fixtures. r[nostr_identity_bindings.final_validation]

## Authorization composition

- [x] [serial] Add opaque `AuthenticatedSubject` construction from reviewed verified inputs. r[nostr_identity_bindings.subject_resolution]
- [x] [serial] Add pure request-bound composition with existing Basalt UCAN decisions. r[nostr_identity_bindings.authority_separation]
- [x] [parallel] Prove authentication cannot bypass denied, missing, stale, or mismatched authorization. r[nostr_identity_bindings.authority_separation]
- [x] [parallel] Prove caller-supplied validity and NIP-05 identity claims fail closed. r[nostr_identity_bindings.verified_input]

## Lifecycle and evidence

- [x] [serial] Add active, retired, and revoked key lifecycle handling. r[nostr_identity_bindings.key_lifecycle]
- [x] [parallel] Add rotation continuity and revoked-key denial fixtures. r[nostr_identity_bindings.key_lifecycle]
- [x] [serial] Add redacted BLAKE3 receipts with typed Nostr SHA-256 references. r[nostr_identity_bindings.receipts]
- [x] [parallel] Add secret-field, digest-substitution, and weakened non-claim rejection fixtures. r[nostr_identity_bindings.receipts]

## Closeout

- [x] [serial] Update public API, policy, authority, rotation, and non-claim documentation. r[nostr_identity_bindings.final_validation]
- [x] [serial] Run policy, no-default, CLI, downstream, Cairn, and release verification. r[nostr_identity_bindings.final_validation]

## Dependency evidence

The dependency is resolved. Basalt vendors integrated UCAN revision
`277ab768eb532ac0c715a3bbff2a7dc37a702f11` and accepts only the reviewed
`ucan-nostr::VerifiedNostrCredential` contract introduced by revision
`c9d43a7c11dd2fab4d5cb3d100012fa9a2e5b026`.
