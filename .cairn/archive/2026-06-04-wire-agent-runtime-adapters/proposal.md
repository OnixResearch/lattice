# Proposal: Wire production agent runtime adapters

## Summary

Wire generic agent nodes into production Pi and Clankers command adapters for default TUI and headless execution. The existing `AgentAdapter` trait remains the test seam, but production executors should route Pi and Clankers targets through configured headless commands instead of always using the placeholder no-op adapter.

## Problem

Generic agent nodes can be modeled, imported, edited, and tested through injected fake adapters, but default runtime construction still uses `NoopAgentAdapter`. A workflow author can create Pi-backed or Clankers-backed nodes, yet a TUI or headless run fails before reaching the real agent runtime.

## Goals

- Add production command adapter implementations for Pi and Clankers headless execution.
- Keep request planning, command selection, and response decoding in deterministic functional cores where possible.
- Make default TUI and headless executors use the production adapter router.
- Preserve explicit injected adapter constructors for tests.
- Cover successful text/JSON responses, unsupported image/artifact output modes, missing command configuration, non-zero process exits, and target routing.

## Non-Goals

- Embed Pi's TypeScript SDK in Rust.
- Implement a long-lived Pi RPC or Clankers daemon client.
- Add credential storage beyond existing node credential references and process environment inheritance.
- Generate image files from agent output; image and artifact modes stay explicit unsupported production outcomes until a file/artifact contract is added.
- Change Nickel workflow syntax for generic agent nodes.

## Success Criteria

- `Executor::new()` routes Pi and Clankers text/JSON requests through configurable production commands.
- Missing commands, unsupported output modes, invalid JSON, and non-zero exits become node-scoped adapter failures.
- Fake adapters and existing unit tests keep working through injected adapter constructors.
- `./scripts/check.rs` passes.
