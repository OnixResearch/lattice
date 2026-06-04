# Proposal: Add agent output file handoff

## Summary

Extend production Pi and Clankers command adapters so generic agent nodes can return image and artifact outputs through a bounded local file handoff envelope. Text and JSON modes continue using stdout directly; image and artifact modes use JSON stdout that identifies a command-produced file under the command working directory.

## Problem

Production agent runtime wiring currently supports text and JSON command outputs, but image and artifact output modes still fail explicitly. The workflow model already exposes image and artifact output ports, so production runs need a safe, deterministic file handoff contract before those ports are usable with real agent commands.

## Goals

- Define a stdout JSON envelope for command-produced image and artifact files.
- Load image handoff files into typed `ImageValue` outputs with media sniffing and content hashing.
- Load artifact handoff files into bytes outputs.
- Bound file size and restrict handoff paths to the command working directory.
- Preserve existing text/JSON stdout behavior and injected fake adapters.
- Cover success and failure paths with positive and negative tests.

## Non-Goals

- Add Pi SDK/RPC or Clankers daemon-specific binary streaming.
- Add remote URL, object-store, or clipboard artifact handoff.
- Persist generated artifacts outside normal workflow value/cache storage.
- Add terminal preview rendering beyond existing image/artifact output display.

## Success Criteria

- A production command in image mode can print `{ "path": "out.png" }` and have the adapter return a typed image output.
- A production command in artifact mode can print `{ "path": "out.bin" }` and have the adapter return bytes.
- Missing files, paths outside the command working directory, oversized files, invalid envelopes, media-type mismatches, and unsupported image bytes fail as node-scoped adapter errors.
- `./scripts/check.rs` passes.
