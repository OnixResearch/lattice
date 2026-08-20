# Tasks

## Phase 1: Contract and core

- [x] [serial] Add the independently consumable `lattice-workflow-transport` crate with the pinned contract dependency. r[lattice.workflow_interchange.transport_envelope]
- [x] [serial] Define closed versioned request, response, rejection, and failure values. r[lattice.workflow_interchange.transport_envelope]
- [x] [serial] Implement deterministic tagged BLAKE3 request and response identities. r[lattice.workflow_interchange.transport_identity]
- [x] [serial] Implement pure request admission for every exchange operation. r[lattice.workflow_interchange.transport_handler]
- [x] [serial] Implement pure response and log-byte admission for every exchange operation. r[lattice.workflow_interchange.transport_response]
- [x] [serial] Define the narrow application-owned `WorkflowExchangeHandler` port. r[lattice.workflow_interchange.transport_handler]

## Phase 2: Shell adapters

- [x] [depends:lattice.workflow_interchange.transport_identity] Implement bounded length-prefixed frame reading and writing. r[lattice.workflow_interchange.transport_bounds]
- [x] [depends:lattice.workflow_interchange.transport_bounds] Implement explicit Unix peer admission and finite server service limits. r[lattice.workflow_interchange.transport_peer]
- [x] [depends:lattice.workflow_interchange.transport_bounds] Implement the matching Unix client and disconnect ambiguity classification. r[lattice.workflow_interchange.transport_ambiguity]

## Phase 3: Evidence and publication

- [x] [parallel] Add positive protocol, handler, client, and server tests. r[lattice.workflow_interchange.transport_evidence]
- [x] [parallel] Add negative malformed, oversized, stale, peer, timeout, disconnect, and handler tests. r[lattice.workflow_interchange.transport_evidence]
- [x] [parallel] Add producer fixtures, documentation, and consumer integration guidance. r[lattice.workflow_interchange.transport_evidence]
- [x] [serial] Add strict Octet deny-all, Clippy, format, and Nix verification rails. r[lattice.workflow_interchange.transport_evidence]
- [x] [serial] Run repository checks, record evidence, sync the accepted specification, and archive the change. r[lattice.workflow_interchange.transport_evidence]
