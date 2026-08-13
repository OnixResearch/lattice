# Proposal: Add exact OnixOS target enforcement actions

## Why

The raw UCAN adapter returns private live authority, but its closed action enum has only document operations.

OnixOS cannot use document actions for activation, persistence, rollback, or state-sync mutation. Such aliases would weaken action attenuation and create false authority.

The reviewed Basalt flake also stopped exporting its repository-owned UCAN source. OnixOS needs an identity-bound source export for offline packaging.

## What Changes

- Add six closed target action variants with exact contract and ability mappings.
- Add reviewed Nickel policy contracts for OnixOS target and state-sync resource prefixes.
- Preserve the existing request, operation-profile, holder-signature, policy, and replay bindings.
- Export `lib.ucanSource` from the checked-in reviewed UCAN snapshot.
- Add positive mapping tests and negative action-substitution tests.
- Document the downstream source and claim boundaries.

## Impact

Basalt can verify raw UCAN invocations for candidate activation, generation persistence, rollback, state-sync publish, state-sync delete, and state-sync administration.

A live result grants only the exact mapped request. It does not execute an effect or prove effect correctness.

## Dependencies

This change extends the archived `verify-ucan-enforcement-requests` boundary. It does not replace or weaken that boundary.

## Non-goals

- Do not map target mutations to document actions.
- Do not add free-form caller-defined enforcement actions.
- Do not accept preverified booleans or receipts as authority.
- Do not add ambient network, clock, trust, storage, or effect execution.
- Do not claim that an unpushed Basalt revision is remotely available.
