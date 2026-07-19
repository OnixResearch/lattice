## Why

The role-agent operating graph has real-model evidence only through a temporary shell wrapper. Operators need a repository-owned OpenAI-compatible transport so headless workflows can call configured model endpoints without Pi, Clankers, generated shell scripts, or plaintext API keys in Nickel.

## What Changes

- Add typed OpenAI-compatible runtime configuration keyed by the existing adapter-neutral agent target labels.
- Route each target to exactly one configured command or HTTP runtime and reject ambiguous or missing routes before I/O.
- Carry resolved node credentials through a non-serializable execution context and use an optional named secret as the bearer token.
- Add deterministic request planning and response decoding cores plus a bounded production HTTP transport with explicit timeout and redirect policy.
- Support strict text and JSON outputs; reject unsupported media outputs, malformed responses, empty content, non-success statuses, timeouts, and oversized bodies.
- Run the Markdown-backed role graph through the native adapter against a real OpenAI-compatible endpoint.

## Impact

- **Files**: `Cargo.toml`, generated `Cargo.lock`, `src/agent_node.rs`, `src/engine/executor.rs`, `src/config/mod.rs`, focused tests, example configuration, README, Cairn lifecycle artifacts, and the audited Octet baseline if source identities change.
- **Testing**: Pre/post agent-node tests, deterministic planning/decoding tests, transport failure fixtures, credential redaction tests, Nickel loading, the real role graph, full repo checks, Clippy, Octet, and Cairn gates.
