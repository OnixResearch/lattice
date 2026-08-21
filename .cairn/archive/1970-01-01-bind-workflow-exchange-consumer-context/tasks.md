## Contract

- [x] [serial] Add and validate optional source context. r[lattice.workflow_interchange.source_context]
- [x] [serial] Bind source context into deterministic request identity. r[lattice.workflow_interchange.source_context]

## Application

- [x] [serial] Add explicit route source-binding policy. r[lattice.workflow_interchange.source_binding]
- [x] [serial] Instantiate exact mapped shell nodes with reserved source environment keys. r[lattice.workflow_interchange.source_binding]
- [x] [serial] Persist and authorize the instantiated workflow revision. r[lattice.workflow_interchange.source_authority]

## Evidence

- [x] [parallel] Add positive and negative contract and application tests. r[lattice.workflow_interchange.source_context] r[lattice.workflow_interchange.source_binding] r[lattice.workflow_interchange.source_authority]
- [x] [serial] Run focused contract, transport, application, Clippy, Octet, Cairn, Tracey, and Nix checks. r[lattice.workflow_interchange.source_authority]
- [x] [serial] Record evidence, sync accepted specs, archive the change, and publish the verified revision. r[lattice.workflow_interchange.source_authority]
