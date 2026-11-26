## Phase 1: Implementation

- [x] [serial] Implement versioned inspect/view and bind the selected member to the exact requested workflow head. r[lattice.source_edit.view]
- [x] [serial] Validate the exact edit using native source preview and atomically save only the draft against both caller tokens. r[lattice.source_edit.save]
- [x] [serial] Return an exact native receipt on commit and bounded machine rejections; document the wire and add focused CLI and transaction coverage. r[lattice.source_edit.receipt]

## Evidence Boundary

Native proposal, design and tasks gates check package structure, not runtime
behavior. The accompanying focused tests exercise durable comparison and the
actual CLI; selected-worker receipt qualification is a host-run check.
