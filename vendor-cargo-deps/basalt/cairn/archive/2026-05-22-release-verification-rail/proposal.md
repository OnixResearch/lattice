# Add release verification rail

## Why

The repo currently relies on several manual commands and has an environment pitfall around shared target directories. A usable crate needs one documented release-readiness rail for formatting, tests, lint, policy freshness, and packaging.

## What Changes

- Define a single maintained release verification command or Nix check set.
- Include fmt, tests, clippy, Nickel policy export freshness, package verification, and consumer feature checks.
- Avoid shared stale target/sccache state by documenting or enforcing a clean target directory strategy.

## Scope

In scope: Nix/Cargo checks, README verification docs, target-dir hygiene, package gate. Out of scope: CI provider setup, publishing credentials, or release tagging.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
