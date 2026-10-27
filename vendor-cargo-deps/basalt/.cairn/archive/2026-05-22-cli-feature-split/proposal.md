# Split CLI dependencies behind a feature

## Why

Applications pulling the library should not be forced to compile or audit CLI-only dependencies. The crate currently exposes a library but always depends on `clap` for the binary.

## What Changes

- Introduce a `cli` feature for the binary surface and gate CLI-only dependencies behind it.
- Keep the library API available through default consumer paths without requiring command-line parsing dependencies.
- Ensure binary builds/tests still work when the CLI feature is enabled.

## Scope

In scope: Cargo feature layout, binary target gating, README commands, focused CLI/library verification. Out of scope: changing CLI behavior or removing the binary.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
