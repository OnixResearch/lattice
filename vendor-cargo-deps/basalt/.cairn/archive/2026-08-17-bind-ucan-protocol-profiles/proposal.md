## Why

Basalt binds its vendored UCAN source revision and support-catalog identity, but its raw enforcement request and receipt do not identify the credential protocol profile. The reviewed snapshot currently implements the local three-part compact format, not final UCAN `1.0.0`.

When UCAN adds explicit local and final protocol profiles, Basalt must reject profile confusion without taking ownership of wire parsing or authorization semantics.

## What Changes

- Add an exact UCAN protocol-profile identity to Basalt UCAN semantics fixtures, raw enforcement inputs, policy declarations, and receipts.
- Bind the current vendored snapshot to the local compact profile.
- Reject missing, unknown, stale, unsupported, and mismatched profiles before authority or replay admission.
- Keep UCAN wire parsing, cryptography, proof traversal, command attenuation, and policy evaluation in the reviewed UCAN component.
- Define the published input contract that a later final UCAN `1.0.0` adapter must satisfy.
- Keep existing Basalt action, policy, authority-core, and live-capability boundaries.

## Impact

Basalt can state which UCAN protocol produced each verified authority input and receipt. Existing compact behavior remains available through an explicit compatibility profile.

A matching profile proves nominal cohort alignment only. It does not prove token correctness, key trust, replay durability, policy completeness, or effect safety.

## Dependencies

This change consumes the archived output and exact source snapshot from UCAN change `declare-ucan-protocol-profiles`. A separate Basalt change will adopt final UCAN `1.0.0` after the UCAN wire and authorization changes archive.

## Non-goals

- Do not parse DAG-CBOR, Varsig, CID, Delegation, Invocation, or compact wire formats in Basalt policy code.
- Do not duplicate command, proof-chain, policy, caveat, or replay semantics.
- Do not silently choose a profile from credential bytes.
- Do not implement AWAKE, transport, DID resolution, proof storage, or revocation storage.
- Do not promote final UCAN `1.0.0` support before exact upstream evidence exists.
