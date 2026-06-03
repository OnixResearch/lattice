## Why

A workflow can visually connect one shell node's `stdout` output to another shell node's `stdin` input, but the downstream command still receives empty stdin. Shell execution currently returns outputs keyed by synthetic sentinel port IDs, while graph input gathering looks up values by the real upstream output port IDs.

## What Changes

- Map shell stdout, stderr, and exit outputs onto the executed node's real graph output port IDs.
- Keep direct single-node execution outputs addressable by real node ports.
- Add a regression proving an `echo hello world → grep hello` workflow passes stdout into downstream stdin.

## Impact

- **Files**: `src/engine/executor.rs`, `tests/test_executor.rs`, `tests/common/mod.rs`, `cairn/specs/tui-ux/spec.md`
- **Testing**: focused executor regression plus full locked tests and Cairn validation.
