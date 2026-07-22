# Design: finish node-template prompt evidence

## Success contract

A maintained positive test MUST inspect the rendered required parameter name, type, required label, description, empty value, sequence, and controls before node creation. A negative companion MUST prove optional parameters are not mislabeled required. Existing state-machine tests MUST continue to prove collection, creation, cancellation, and invalid-input recovery.

All three inventory rows MUST become `exact_maintained_evidence`, bind current safe file-byte BLAKE3 identities, retain all 327 missing IDs, retain zero dangling markers, and remain unpromoted.

## Functional core

`render_add_node_parameter_lines` is the deterministic functional core: it receives immutable app state and returns rendered lines without I/O or external mutation. Integration tests inspect its output through the public bounded-buffer rendering shell. Existing app methods remain the imperative state-transition shell.

## Evidence boundary

The required-prompt row needs both the state-transition evidence in `src/tui/app.rs` and user-visible rendering evidence in `src/tui/panels.rs`. Parent collection and cancellation remain bound to their direct app assertions. Test names, implementation shape without assertions, archives, and generated inventory are excluded.

## Validation

Focused tests run before and after the assertion change. Completion validation checks the exact three-row classification, byte identities, no selected markers, unchanged missing set, and deterministic negative mutations.

## Non-claims

Exact evidence is bounded to the accepted clauses and maintained tests. It creates no Tracey evidence until a separate promotion is validated.
