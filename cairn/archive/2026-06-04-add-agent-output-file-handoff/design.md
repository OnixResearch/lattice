# Design: Agent output file handoff

## Overview

Production agent commands keep a simple interface:

- text mode: stdout is the final text value;
- JSON mode: stdout is the final JSON value;
- image mode: stdout is a JSON file handoff envelope;
- artifact mode: stdout is a JSON file handoff envelope.

The initial envelope is local-only and intentionally small:

```json
{"path":"relative-or-absolute-file", "media_type":"image/png", "metadata":{"optional":"safe metadata"}}
```

`path` is required. `media_type` is optional for images and, when present, must match sniffed bytes. `metadata` is optional and is copied under adapter metadata.

## Functional core

Pure helpers own deterministic decisions:

- parse handoff envelope JSON;
- resolve relative paths against a declared working directory string;
- derive metadata JSON from adapter label, target, stdout size, output path label, byte count, and optional envelope metadata;
- validate a declared image media type against sniffed media type.

These helpers take plain strings/values and return plain structs or errors.

## Imperative shell

The production adapter shell owns side effects:

1. run the command and capture stdout/stderr;
2. parse an envelope for image/artifact output modes;
3. canonicalize the command working directory and handoff file path;
4. reject files outside the working directory;
5. check metadata size before reading;
6. read bytes;
7. decode image bytes through existing image helpers or return artifact bytes.

## Safety

- Handoff files MUST resolve under the command working directory. If the request has no explicit cwd, `.` is the working directory boundary.
- File size MUST be bounded by `agent_runtime.max_output_bytes`.
- Image bytes MUST pass existing PNG/JPEG sniffing. Declared media type MUST match sniffed media type when provided.
- Errors MUST be returned as `AgentError::Adapter` so they remain node-scoped and do not corrupt saved workflow state.

## Tests

Add positive tests for image and artifact handoff through production command adapters. Add negative tests for invalid envelope JSON, missing file, outside-working-directory path, oversized artifact, image media mismatch, and unsupported image bytes. Add one headless/default-runtime test proving image handoff reaches the real workflow image output port.

## Follow-up

Future adapter-specific integrations may replace stdout envelopes with Pi RPC images or Clankers daemon artifact handles while keeping the workflow node shape and executor response mapping unchanged.
