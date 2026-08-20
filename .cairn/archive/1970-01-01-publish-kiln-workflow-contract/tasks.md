# Tasks: Publish the Kiln workflow contract

## Contract and identity

- [x] [serial] Add the independent `lattice-workflow-contract` repository crate and document its ownership boundary. r[lattice.workflow_interchange.contract]
- [x] [serial] Define versioned profile, request, graph, acceptance, observation, control, and log values. r[lattice.workflow_interchange.request] r[lattice.workflow_interchange.observation] r[lattice.workflow_interchange.controls_logs]
- [x] [serial] Implement deterministic framed BLAKE3 identities for profiles, graphs, requests, and observations. r[lattice.workflow_interchange.request] r[lattice.workflow_interchange.observation]

## Pure admission

- [x] [serial] Implement bounded profile and request admission with capability, ordering, reference, and DAG checks. r[lattice.workflow_interchange.graph]
- [x] [serial] Implement linked lifecycle observation admission with explicit unknown, pause, terminal, receipt, and log rules. r[lattice.workflow_interchange.observation]
- [x] [parallel] Implement exact control and bounded log-read admission. r[lattice.workflow_interchange.controls_logs]

## Contracts and fixtures

- [x] [serial] Add the typed Nickel contract and positive plus negative Nickel fixtures. r[lattice.workflow_interchange.contract] r[lattice.workflow_interchange.evidence]
- [x] [serial] Add canonical JSON producer fixtures and independent decode checks. r[lattice.workflow_interchange.evidence]
- [x] [parallel] Add positive and negative Rust tests for identity, graph, capability, lifecycle, reconciliation, control, and log behavior. r[lattice.workflow_interchange.graph] r[lattice.workflow_interchange.observation] r[lattice.workflow_interchange.controls_logs]

## Closeout

- [x] [serial] Document the consumer workflow, source pin, compatibility boundary, and non-claims. r[lattice.workflow_interchange.contract] r[lattice.workflow_interchange.evidence]
- [x] [serial] Run formatting, tests, strict Clippy, `no_std`, Nickel, Octet, Cairn, Tracey, and Nix checks. r[lattice.workflow_interchange.evidence]
- [x] [serial] Record evidence, sync the accepted specification, archive the change, publish the branch, and integrate remote `main`. r[lattice.workflow_interchange.evidence]
