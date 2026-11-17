# Authoring and acquisition evidence

- Producer source acquisition: anonymous HTTPS codeload of Aspen revision `afc8308bff9a8c50b2315cb4a17c9454252d7c45` succeeded.
- Exact downloaded gzip archive BLAKE3: `a6307998de7d69f737e1ce39465c63f6742aa4612979321597a304b89771363d`.
- Maintained importer executed against those acquired bytes and imported 26 selected files/metadata entries into the new `vendor-cargo-deps/aspen-render` tree. `SOURCE.json` is the exact selection/license/file inventory; only the explicitly owned sparse workspace wrapper is synthesized.
- Parent-performed LSP reference queries identified preparation/classification callers in `src/engine/executor.rs` and `src/operation/shell.rs`; dispatch signatures/callers were migrated to explicit optional host execution context. The worker's `xd://lsp` device was unavailable, not silently replaced with guessed references.
- No build, test, lint, formatter or qualification fixture was run by the implementation worker. Native regression definitions and the Nix-driven real-route campaign are authored sources, not passing qualification evidence.

## Precise unavailable producer contracts

`molten-batch-core::JobSpec`/`ExecutionRequest` exposes per-attempt fuel,
`timeout_ms` and retry bounds, but not a durable job-level absolute-deadline,
aggregate-budget or current-Lattice-fence admission port. The consumer preserves
original reservations/deadline and current-owner rechecks, rejects late acceptance
and requests cancellation while observing; detached remote deadline enforcement
needs that producer/managed-owner API. `execution-result.v1` has no metered usage
fields. These missing producer facts remain unavailable, not fabricated telemetry
or a second local budget/scheduling authority.

Actual retained Lattice graph/component/fault/repetition outcomes must be produced
by the integration owner's commands in `docs/render-workflow.md` before claiming
issue #18's real-route qualification. No producer-local passing test or historical
Aspen receipt is substituted for those results.
