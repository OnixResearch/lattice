# Use remote UCAN dependency

## Why

The crate cannot be packaged or consumed outside this checkout while `ucan` is path-only. Consumers need Cargo to resolve UCAN from the authoritative remote rather than from `../ucan`.

## What Changes

- Replace the path-only UCAN dependency with a remote-pinned dependency shape suitable for downstream applications.
- Keep local development ergonomic through an optional path override or documented patch workflow without making package consumers depend on sibling checkout layout.
- Prove `cargo package` no longer fails on missing UCAN version/source metadata.

## Scope

In scope: Cargo dependency shape, lockfile update, README/developer workflow notes, package verification. Out of scope: publishing UCAN itself, changing UCAN APIs, or changing runtime authorization semantics.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
