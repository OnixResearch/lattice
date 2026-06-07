# Proposal: Webhook trigger cookbook

## Summary

Add an end-to-end webhook trigger cookbook with a runnable Nickel workflow, operator documentation, and smoke tests that exercise listener dry-run, valid request admission, deterministic rejection, idempotency, and credential redaction.

## Problem

Webhook triggers and the listener server are implemented, but operators do not yet have a small, copyable example that shows how a webhook trigger is represented in Nickel, how it is imported into a store, how the listener is dry-run before serving, and how accepted and rejected requests behave. Without a cookbook, the newest trigger surface is harder to discover and more likely to drift from examples and README guidance.

## Goals

- Provide a runnable webhook-triggered workflow example under the examples tree.
- Document the import, dry-run, listen, request, and inspection flow in README or examples documentation.
- Keep cookbook authentication examples credential-reference based and free of plaintext secret material.
- Add smoke coverage proving the cookbook imports, validates, and exercises both accepted and rejected webhook paths.
- Ensure example failures produce deterministic, safe diagnostics.

## Non-Goals

- Add a hosted webhook gateway or deployment service.
- Add TLS certificate management.
- Expand the webhook listener protocol beyond the accepted trigger model.
- Replace existing scheduler or listener tests with cookbook tests.

## Success Criteria

- A new user can follow the cookbook from a clean config directory and see a webhook-triggered workflow run.
- The cookbook includes at least one accepted request path and one rejected request path with stable, documented outcomes.
- Cookbook workflows and commands are covered by automated smoke tests.
- No cookbook file, diagnostic, test output, or history assertion contains plaintext webhook secret material.
