# Design: Add exact OnixOS target enforcement actions

## Goals

- Give each protected target effect one closed typed action.
- Preserve exact request and holder binding through the existing live adapter.
- Keep product effects outside Basalt.
- Restore a durable UCAN source output from the reviewed snapshot.

## Action mappings

The closed enum adds these mappings:

| Action | Contract | Ability | Resource prefix |
|---|---|---|---|
| Candidate activation | `onixos-target-candidate-activation` | `onix/target-candidate-activation` | `did:key:z6Mkonix/onixos/` |
| Generation persistence | `onixos-target-generation-persistence` | `onix/target-generation-persistence` | `did:key:z6Mkonix/onixos/` |
| Rollback | `onixos-target-rollback` | `onix/target-rollback` | `did:key:z6Mkonix/onixos/` |
| State-sync publish | `onixos-state-sync-publish` | `state_sync.publish` | `onix://state-sync/` |
| State-sync delete | `onixos-state-sync-delete` | `state_sync.delete` | `onix://state-sync/` |
| State-sync administration | `onixos-state-sync-admin` | `state_sync.admin` | `onix://state-sync/` |

Each variant has a distinct operation-profile BLAKE3 identity. The digest includes the action, contract, and ability with big-endian `u64` length prefixes.

## Functional core

The existing pure `bind_enforcement_action` core remains the mapping boundary. It rejects caller capability fields and any contract or ability mismatch.

The existing request-binding digest includes the selected action, contract, resource, and ability. A resource change therefore changes holder proof material.

No generic string constructor is added. Callers can select only one reviewed enum variant.

## Policy boundary

Nickel remains the policy source. Each target action gets one contract with one ability and one bounded resource prefix.

Separate contracts prevent one granted action from authorizing another action. State-sync publish does not imply delete or administrative authority.

## Source export

The Basalt repository already owns `vendor/ucan/`, `vendor/ucan-revision.txt`, and `vendor/ucan.snapshot`.

The flake exports `lib.ucanSource = ./vendor/ucan`. This output names source within the exact Basalt flake revision.

The output replaces downstream use of the removed `inputs.ucan` surface. It does not recreate a remote UCAN pin or weaken the vendored snapshot checks.

## Verification

Positive tests check every action, mapping, policy contract, and operation-profile identity.

Negative tests substitute contract, ability, action, and state-sync authority. Every substitution must fail before replay or effect dispatch.

The existing raw verification, replay ordering, redaction, and non-forgeability tests remain mandatory.

## Claim boundary

A successful result proves only bounded evaluation for the exact action, contract, resource, ability, request, and supplied verification boundaries.

It does not prove machine identity discovery, global revocation freshness, replay durability, transport security, effect execution, effect correctness, or release eligibility.
