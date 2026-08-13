# Add consumer integration tests

## Why

Unit tests prove local logic, but applications need evidence that the crate works when consumed through its public API and configured features.

## What Changes

- Add integration tests that exercise the crate as a downstream application would use it.
- Cover positive and negative enforcement flows, malformed policy/schema errors, and feature-gated CLI smoke where applicable.
- Keep fixtures deterministic and easy to audit.

## Scope

In scope: integration tests, fixtures, CLI smoke for feature-enabled builds. Out of scope: live network tests, credential tests, or UCAN issuance flows.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
