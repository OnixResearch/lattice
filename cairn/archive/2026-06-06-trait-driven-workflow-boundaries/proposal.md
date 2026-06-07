# Proposal: Trait-driven workflow boundaries

## Summary

Refactor n8nclone's workflow engine boundaries around narrow Rust traits for node runtimes, command execution, node-kind metadata, persistence, and runner events. Keep persisted workflow data as closed serializable enums, while moving behavior behind testable ports that preserve the functional-core / imperative-shell split.

## Problem

Several execution paths still branch directly on `NodeConfig` or concrete services. The executor owns runtime selection, command spawning, output mapping, and error shaping in one place; the TUI, validation, and format layers repeat node-kind knowledge; headless and scheduling code depend on concrete storage; and runner progress has no common event sink. This makes new node kinds costly, mixes behavior with representation, and forces tests to stand up more infrastructure than the logic requires.

## Goals

- Route node side effects through a runtime registry backed by `NodeRuntime` implementations.
- Route shell-like and agent-like command spawning through a shared process-runner trait with production and fake adapters.
- Centralize node-kind display, editable-field, validation, and format metadata behind explicit node-kind provider traits.
- Split concrete redb storage behind small repository/cache/history traits at runner, headless, and scheduler boundaries.
- Emit runner lifecycle facts through a common event-sink trait usable by TUI, headless, scheduling, and tests.
- Preserve current persisted workflow syntax, existing node behavior, cache semantics, and run-history compatibility.

## Non-Goals

- Replace `NodeConfig`, `FileOperationKind`, `AgentOutputMode`, or `TypeTag` with trait objects.
- Introduce dynamic plugin loading or runtime-loaded Rust code.
- Change the Nickel workflow format except for internal handler structure.
- Remove existing production HTTP, file, image, credential, or agent adapters.
- Rewrite the whole executor in one step.

## Success Criteria

- Existing shell, HTTP, file, image, and generic agent workflows still run through TUI, headless, scheduler, and tests.
- Runtime dispatch, process execution, event delivery, storage access, validation, TUI metadata, and format handlers each have positive and negative tests.
- Adding a built-in node kind requires adding a provider/runtime implementation rather than editing broad `NodeConfig` matches across executor, TUI, validation, and format code.
- Fake implementations can exercise runner behavior without spawning processes, opening redb, making network calls, or reading files.
- Focused tests and `./scripts/check.rs` pass, or blockers are documented with the smallest failed command.
