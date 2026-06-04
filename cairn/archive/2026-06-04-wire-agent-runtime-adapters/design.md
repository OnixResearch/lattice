# Design: Production agent runtime adapter wiring

## Overview

The current agent node implementation already has a clean runtime seam: executor logic builds an `AgentRequest`, calls an `AgentAdapter`, and maps `AgentResponse` values to real graph output ports. This change fills the production side of that seam with a router that dispatches requests to Pi or Clankers command adapters.

## Architecture

### Functional core

Keep deterministic adapter planning separate from process I/O:

- map `AgentTarget` to the configured runtime command;
- build the prompt passed to a headless runtime from prompt text, ordered text/JSON/image input summaries, context references, tool policy, model hints, working directory, and session labels;
- derive process arguments for Pi and Clankers text/JSON runs;
- decode stdout into `AgentResponse` for text and JSON output modes;
- reject image and artifact output modes until a file/artifact handoff contract exists.

These helpers should take plain values and return plain plans/results so tests can exercise success and failure without spawning processes.

### Imperative shell

Add a thin process runner boundary:

1. resolve the command path from app configuration or a production default;
2. reject missing command configuration before spawning;
3. spawn the command with planned args, prompt stdin, optional cwd, and inherited environment;
4. collect stdout/stderr;
5. map spawn errors, non-zero exits, invalid JSON, and unsupported output modes into `AgentError` values.

### Runtime wiring

- `Executor::new()` should use a production agent adapter router.
- Headless runs use `Executor::new()`, so saved workflows gain production agent routing automatically.
- TUI background runs use `Executor::new()`, so user-created agent nodes gain the same routing.
- `Executor::new_with_agent_adapter` and `Executor::new_with_runtime_clients` remain available for deterministic tests.

## Command contracts

Pi defaults to print mode because the Pi README documents `pi -p "prompt"`, stdin merging, `--model`, and `--no-session` for headless integration.

Clankers defaults to one-shot prompt mode because the Clankers README documents `clankers -p "prompt"`, `--mode json`, and stdin support for headless runs.

The initial production adapter supports text and JSON output modes:

- text mode returns trimmed stdout as `AgentOutput::Text`;
- JSON mode parses stdout as JSON and returns `AgentOutput::Json`;
- image and artifact modes return explicit unsupported adapter errors until future work defines output-file conventions.

## Safety and determinism

- Do not spawn a process from the pure executor branch directly; all process I/O lives in the production adapter shell.
- Do not silently fall back from one target to another.
- Include stderr and status in bounded, node-scoped errors.
- Use named constants for default commands, arguments, and truncation limits.
- Preserve raw prompt content only in process stdin/arguments needed for execution; tests assert metadata and lengths where possible.

## Tests

Add positive and negative tests:

- core planning builds Pi and Clankers commands from the same neutral request;
- text stdout decodes into a text response;
- JSON stdout decodes into a JSON response;
- unsupported image output mode rejects before spawning;
- missing runtime command rejects before spawning;
- non-zero exits include bounded stderr in the adapter error;
- default executor contains production routing behavior while injected fake adapters still work.

## Follow-up

Future changes can replace command adapters with Pi RPC/SDK or Clankers daemon/MCP clients and add image/artifact file handoff contracts without changing workflow node shape.
