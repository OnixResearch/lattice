## Why

Machine callers need to reopen exact retained workflow source and save a validated draft without blessing a newer catalog head or publishing it. The existing draft CLI snapshots current head itself; explicit admission is a different operation.

## What Changes

- Expose a closed JSON owner protocol to inspect the catalog head and view exact bytes. r[lattice.source_edit.view]
- Compare caller-provided head and draft revisions in one draft-only write transaction after native validation. r[lattice.source_edit.save]
- Return bounded machine errors and a metadata-only receipt with the exact committed draft and compiler/candidate identity. r[lattice.source_edit.receipt]

## Impact

- **Files:** source authoring CLI and redb catalog authoring storage; existing publication and workflow execution are unchanged.
- **Testing:** focused real CLI and store transaction tests; positive native evaluation uses the existing host-qualified captured worker.
