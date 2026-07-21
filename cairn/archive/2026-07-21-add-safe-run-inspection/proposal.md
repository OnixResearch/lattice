## Why

Lattice persists owner-bound run summaries, redaction-safe node details, replay events, and orchestration evidence, but headless operators cannot inspect those records through the CLI. They must open redb indirectly or infer run state from separate orchestration commands. Directly serializing persisted details would expose arbitrary output and error text, including legitimate prompt echoes, so inspection needs a deliberately narrower projection.

## What Changes

- Add `lattice runs show <run-id> --output text|json` as a read-only operator command.
- Project persisted run records into a stable safe inspection report that omits node labels, port labels, output text, error text, replay summaries, orchestration summaries, objectives, actors, and trigger keys.
- Report lifecycle counts, exact owner binding, dispatch/recovery/abandonment state when available, per-dispatch budget evidence, and run/orchestration replay validity.
- Treat missing optional historical details or replay as explicitly not recorded, while failing the command for malformed run identity, mismatched detail identity, foreign owner binding, or invalid replay evidence.
- Keep unobserved dispatches outside the run-record command; operators continue to inspect those by orchestration instance because no `RunRecord` exists.

## Impact

- **Files**: `src/run/inspection/`, `src/run/shell.rs`, `src/run/mod.rs`, `src/main.rs`, focused core/CLI tests, `README.md`, and Cairn lifecycle artifacts.
- **Testing**: baseline and post-change run-history/store/orchestration tests, positive and negative safe-projection tests, CLI text/JSON/not-found fixtures, secret/prompt marker exclusion checks, repository checks, Octet comparison, Cairn gates, and bounded Nix validation.
