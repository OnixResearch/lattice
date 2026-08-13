# Preserve AGPL license boundary

## Why

The crate is intended to remain AGPL-3.0-or-later. That license choice must be explicit in package metadata and downstream documentation so applications understand the adoption boundary.

## What Changes

- Keep `AGPL-3.0-or-later` as the package license.
- Document the license boundary in consumer-facing README/package metadata without adding alternative licensing claims.
- Ensure verification prevents accidental license metadata removal or weakening.

## Scope

In scope: preserving license metadata and documentation. Out of scope: relicensing, dual licensing, legal advice, or compatibility exceptions.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
