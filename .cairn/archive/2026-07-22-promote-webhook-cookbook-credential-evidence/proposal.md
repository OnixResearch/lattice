# Promote webhook cookbook credential evidence

## Summary

Promote exactly `n8nclone.credentials.webhook_cookbook_secret_safety` from the reviewed credential inventory into maintained Tracey evidence now that its exact source and assertion boundaries are recorded.

## Motivation

The remaining-credential audit classified this row as exact maintained evidence but intentionally added no marker. A separate bounded promotion is required so the marker is attached to the complete accepted and rejected cookbook smoke assertions and the coverage transition is proven independently.

## Scope

- Attach the reviewed requirement marker directly to the exact accepted and rejected cookbook tests.
- Remove only that reviewed row from current debt policy.
- Prove the missing set changes from 341 to 340 with no additions or dangling markers.
- Preserve global failing coverage and all runtime behavior.

## Non-goals

This change does not alter webhook authentication, credential providers, cookbook behavior, n8n parity, or any of the 13 partial classifications. It does not claim complete traceability, release readiness, or production readiness.
