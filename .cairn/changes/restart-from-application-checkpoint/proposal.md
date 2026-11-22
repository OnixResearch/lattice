## Why

Lattice #12 asks that a supported workflow resume on another host from a committed application
checkpoint after source loss, that unsupported workloads stay explicit, and that restarts never
blindly replay effects, reset budgets, deadlines or cancellation, or blur application restart,
VM continuation and fresh retry. The relocation plan (slice S3) fixes the first profile as
`workflow-node-boundary-v1` on the workflow exchange route and the contract C8
(`lattice.application-checkpoint-manifest.v1`, `lattice.application-restart-receipt.v1`).

## What Changes

- Pure C8 manifest construction and admission: identity and canonical bytes, schema, runtime
  revision (runtime and checkpoint source digests) and environment, architecture or a declared
  portable profile, crossed request, execution and binding, stale admission, objects present and
  verified, commit marker present, deadline, elapsed budget and cancellation carried forward
  unchanged, and Kiln's cancel decision from the restart request OR-ed in.
- A node-boundary hook on the exchange worker, absent unless the route's opaque C1 profile
  declares a checkpoint. It publishes at completed-node boundaries with the
  `checkpoint_from_results` sensitive-output rule, and synchronously before every effect
  reservation so any started effect is named as uncertain. A host-requested drain parks the next
  effect after a quiescent manifest.
- Restore under an opaque Kiln admission reference into a fresh store: target route and binding
  re-verification (deadline and attempt binding included), `Session::new_with_checkpoint`, and
  one atomic adoption committed after every check and the workspace write, keeping the execution
  identity and adapter identities, with uncertain nodes adopted as `Unknown` and never replayed.
- The C8 restart receipt and a pure classification of application restart, VM continuation and
  fresh retry.
- `lattice application-checkpoint restore|inspect`, the process boundary Kiln consumes (OD-1),
  plus positive and negative JSON fixtures; `lattice workflow-exchange serve
  --checkpoint-binding` publishes checkpoints and runs adopted executions, observable and
  cancellable through the same exchange.

## Owner decisions applied

- **OD-1:** Kiln consumes this only through the `lattice` process and published fixtures.
- **OD-2:** no placement authority is introduced; the admission reference stays opaque and the
  single-host evidence uses two state roots only. No local-lock fallback.
- **OD-3:** no live model credentials; the exchange route here runs scripted shell effects.
- **OD-5:** the four #12 addenda (recurrent state, per-call recovery, Archil workspace survival,
  coding-agent continuity) stay with their producers and are not addressed.

## Impact

- **Files:** new module `src/checkpoint/` (`mod.rs`, `cli.rs`, `core.rs` with `core/{manifest,
  admission,adoption}.rs`, `model.rs` with `model/{adoption,refusal}.rs`, `publish.rs` with
  `publish/hook.rs`, `restore.rs` with `restore/files.rs`, `tests.rs`); the exchange core
  adoption (`src/workflow_exchange/core/restore.rs`: `adopt_checkpoint`,
  `node_receipt_identity`); the handler restore and adopted-execution start
  (`src/workflow_exchange/application/shell/checkpoint.rs`, `service.rs`); adoption storage
  (`src/store/exchange/adoption.rs`); the snapshot continuation
  `SnapshotRunRequest::run_from_checkpoint` (`src/headless/shell/checkpoint.rs`) sharing the
  snapshot persistence sequence (`src/headless/shell/record.rs`); the executor entry
  `run_authorized_from_checkpoint` (`src/engine/executor/choice.rs`); the checkpoint source
  digest (`build.rs`); worker hook; small hunks in the route authority (reservation fence),
  handler shell, re-exports, `src/lib.rs` and `src/main.rs`; `docs/application-checkpoint.md`;
  `fixtures/application-checkpoint/`; `tests/application_checkpoint_restart.rs`.
- **Wire compatibility:** no v1 schema Kiln consumes changes (plan D6). New behaviour uses new
  versioned schemas; optional fields are absent when unset.
- **Non-goals:** cross-host or production placement claims, durable cross-host content
  transfer (Kiln #53/#54, Aspen), live-model runs, the repair-step and per-call profiles.
