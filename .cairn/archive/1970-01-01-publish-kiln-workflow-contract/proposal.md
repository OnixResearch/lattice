# Proposal: Publish the Kiln workflow contract

## Why

Kiln needs a stable workflow request and observation boundary without importing Lattice runtime internals. Lattice currently exposes product and operation types only through its application crate.

A direct dependency would transfer persistence, process, UI, and provider details into a CI control-plane adapter. It would also prevent independent request validation before effects.

## What Changes

- Add a narrow `lattice-workflow-contract` crate under the repository workspace.
- Keep the crate `no_std + alloc`, pure, versioned, and independently testable.
- Define bounded capability profiles, Kiln linkage, DAG requests, acceptance records, lifecycle observations, controls, log reads, and stable error codes.
- Bind profiles, graphs, requests, and observations with deterministic BLAKE3 identities.
- Add typed Nickel contracts and accepted plus rejected producer fixtures.
- Publish the completed contract at one immutable Lattice revision for downstream consumers.

## Impact

- **Crate**: `crates/lattice-workflow-contract`
- **Contracts**: `contracts/kiln-workflow-contract.ncl`
- **Fixtures**: `tests/fixtures/kiln-workflow-contract`
- **Consumer**: Kiln can pin the published revision and build an optional adapter.
- **Runtime**: No Lattice workflow execution path changes in this package.

## Non-goals

- Do not import Kiln types into Lattice.
- Do not execute workflows, read credentials, open files, start processes, or call providers from the contract crate.
- Do not let Lattice observations replace canonical Kiln CI state.
- Do not claim workflow correctness, host confinement, effect success, credential freshness, or release readiness.
