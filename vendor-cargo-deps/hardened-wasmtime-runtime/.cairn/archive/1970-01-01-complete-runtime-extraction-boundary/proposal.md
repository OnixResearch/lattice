# Proposal: Complete the executable runtime extraction boundary

## Why

The published extraction at `8d88e00` archives its requirements, but an executed
baseline discovers only ten core tests and one shell identity test. Nested
admission/boundary files are not Cargo entrypoints. The engine accepts arbitrary
Wasmtime configuration; the shell accepts only core modules, ignores artifact
size and materialization facts, and cannot bind admitted consumer imports.
Molten cannot adopt this as a completed component-runtime mechanism.

## What Changes

Enforce explicit admitted features and cumulative bounds at engine construction;
validate artifact identity, class and exact imports/exports before instantiation;
provide actual component execution with bounded owned output before automatic
guest cleanup; bind only explicit consumer adapters; require an injected, engine-bound epoch service and
positive deadline; exercise both module and component consumers. Make meaningful
existing admission regressions executable and replace obsolete shell fixtures.

## Impact

The product-neutral runtime owns mechanism, not policy or authorization. Molten
and independent consumers retain worlds, effect meaning, materialization trust,
capability decisions and canonical receipts. No ambient WASI, fallback execution,
sibling-path dependency, timer thread or product cutover is introduced. Existing
historical archives are preserved, not rewritten as fresh execution evidence.
