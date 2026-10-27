# Adopt non-consuming holder observations

## Why

Ongoing consumers need current UCAN observations without admitting the same nonce again. UCAN owns that verifier and has published it at `3a2aa043068a82663cf72ec05ed5d56a7d65ecd5`.

## What Changes

Refresh the existing reviewed vendor snapshot from that exact published object. Update cohort identities and generated Nickel projections. Add a narrow `LiveUcanEnforcement::observe_holder` shell operation that forwards to UCAN using the retained private holder authorization.

## Impact

No token wire format, default process permission, replay store, current Basalt policy decision, or release authority is added. The reviewed vendor mechanism remains the private packaging boundary. Consumers still need authenticated services and atomic effect-time authority checks.
