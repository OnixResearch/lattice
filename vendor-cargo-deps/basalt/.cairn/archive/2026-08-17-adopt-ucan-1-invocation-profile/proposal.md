## Why

Basalt can bind its current compact UCAN adapter to an exact protocol profile, but final UCAN `1.0.0` uses different Delegation, Invocation, command, policy, proof, and replay semantics.

Basalt should adopt those verified facts only after UCAN publishes archived wire and authorization evidence. It must use a separate adapter and closed application mapping instead of parsing tokens or duplicating authority rules.

## What Changes

- Refresh the reviewed vendored UCAN snapshot to an exact revision with archived final-profile wire and authorization evidence.
- Add a separate adapter from UCAN's live verified final-profile Invocation into Basalt's application-owned authority input.
- Add typed Nickel mappings from admitted UCAN commands and complete subject/resource facts to closed `EnforcementAction` values.
- Reject lossy conversion, caller assertions, source/profile mismatches, and compact/final adapter substitution.
- Bind final-profile decisions and receipts to the exact UCAN and Basalt cohorts.
- Keep the compact adapter until a separate migration removes it.

## Impact

Basalt can enforce application policy over verified final UCAN `1.0.0` Invocation facts. UCAN retains wire, signature, proof-chain, command, policy, revocation-observation, and replay semantics. Basalt retains application action and contract policy.

## Dependencies

This change depends locally on `bind-ucan-protocol-profiles`. Implementation also requires archived evidence from UCAN changes `implement-ucan-1-wire-profile` and `implement-ucan-1-authorization`.

## Non-goals

- Do not parse UCAN wire formats in Basalt.
- Do not duplicate UCAN command, policy, proof-chain, signature, revocation, or replay semantics.
- Do not remove the compact profile in this change.
- Do not implement AWAKE, Promise, Revocation storage, transport, DID resolution, or proof storage.
- Do not treat a passing receipt as live authority or effect evidence.
