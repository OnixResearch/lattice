# Tasks: Admit exact reviewed UCAN snapshot revisions

## Phase 1: Importer contract

- [x] [serial] r[ucan_token_flow.vendored_source.exact_revision] Add exact-revision source parity and import modes.
- [x] [serial] r[ucan_token_flow.vendored_source.exact_revision] Make snapshot checks use the validated recorded revision.
- [x] [parallel] r[ucan_token_flow.vendored_source.exact_revision] Add positive tests for a full exact revision.
- [x] [parallel] r[ucan_token_flow.vendored_source.exact_revision] Add negative tests for moving and malformed revisions.

## Phase 2: Pilot evidence

- [x] [serial] r[ucan_token_flow.vendored_source.exact_revision] Document the exact import and recovery procedure.
- [x] [serial] r[ucan_token_flow.vendored_source.exact_revision] Run source parity, snapshot, focused tests, and release checks.
- [x] [serial] r[ucan_token_flow.vendored_source.exact_revision] Review sync and archive dry runs before lifecycle mutation.
