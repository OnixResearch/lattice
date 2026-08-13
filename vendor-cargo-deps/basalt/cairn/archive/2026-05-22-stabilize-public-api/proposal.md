# Stabilize public library API

## Why

Other applications need a clear, documented, semver-conscious API instead of relying on incidental public fields and minimal examples.

## What Changes

- Define the supported public API for policy parsing, requests, capabilities, receipts, and errors.
- Add constructors/accessors/serde derives where they improve downstream ergonomics without exposing unstable internals unnecessarily.
- Document semver expectations and examples for application integration.

## Scope

In scope: library API shape, docs, examples, tests for public behavior. Out of scope: changing UCAN authorization semantics or adding network/token issuance.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
