# Design: Production image I/O runtime wiring

## Overview

The prior image support work introduced a typed image value model and an `ImageIo` trait, but production executor construction still uses a no-op implementation. This change adds a local filesystem implementation and wires it into the default executor used by TUI and headless runs.

## Architecture

### Functional core

Keep deterministic image rules independent of filesystem access:

- path allow-list checking
- byte budget validation
- PNG/JPEG media-type sniffing
- image metadata construction from an already-loaded byte buffer
- stable content hashing through `ImageValue`

These functions should take plain values and return plain results. Tests should cover them with in-memory inputs and no mocks.

### Imperative shell

Add `LocalImageIo` as the thin shell that handles local side effects:

1. validate the requested path against configured roots before reading;
2. inspect file metadata when useful for an early size check;
3. read file bytes;
4. map OS errors to `ImageError` variants;
5. call the pure image planning/validation helpers;
6. return `ImageValue` with source-path metadata.

Default executor construction should use `LocalImageIo`. Existing specialized constructors should continue accepting injected `ImageIo` for deterministic tests.

## Runtime behavior

- `Executor::new()` should construct a real local image I/O client.
- TUI background runs call `Executor::new()`, so they pick up local image support automatically.
- Headless runs call `Executor::new()`, so saved/imported image workflows can run from the CLI.
- Tests that need no filesystem should continue using `MemoryImageIo` or explicit runtime-client constructors.

## Error mapping

`LocalImageIo` should map side effects to existing node-scoped diagnostics:

- missing file -> `ImageError::MissingImage`
- permission denied -> `ImageError::PermissionDenied`
- path outside allowed roots -> `ImageError::UnsafePath`
- bytes over budget -> `ImageError::OversizedImage`
- unsupported media type or corrupt bytes -> `ImageError::UnsupportedFormat`
- other read/metadata failures -> `ImageError::Io`

## Safety and determinism

- Validate path roots before reading file bytes.
- Use named constants for test image fixtures, byte budgets, and file names.
- Do not hide filesystem errors behind generic adapter failures.
- Do not read images during workflow import/export; image bytes are runtime data.
- Keep image content hashing based on actual loaded bytes and media type.

## Tests

Add positive and negative tests:

- local PNG loads through `LocalImageIo`;
- local JPEG loads through `LocalImageIo`;
- `Executor::new()` can execute an image input node backed by a temp local file;
- missing file fails clearly;
- unsupported bytes fail clearly;
- oversized file fails before downstream execution;
- outside-root path remains rejected;
- injected `MemoryImageIo` tests remain unchanged.

## Follow-up

After this change, the next production step is implementing real Pi and Clankers agent adapters. That should be a separate Cairn change because it introduces runtime credentials, sessions, and external process/API behavior.
