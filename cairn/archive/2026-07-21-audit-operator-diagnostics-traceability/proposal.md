## Why

The maintained traceability inventory still contains eight unassessed `operator-diagnostics` gaps. Current doctor source and tests provide materially different evidence strengths: some requirements have complete implementation evidence, some have only partial assertions, and redaction claims are not wired to injected sensitive values. Leaving all eight unassessed hides that distinction, while adding markers during review would conflate classification with coverage.

## What Changes

- Review all eight missing operator-diagnostics identifiers against current `src/main.rs` and `tests/test_doctor.rs` evidence.
- Record two exact maintained-evidence rows, four partial-evidence rows, and two no-exact-evidence-identified rows in typed Nickel policy.
- Bind exact and partial rows to current file-byte BLAKE3 identities and deterministic rationale codes.
- Add positive and fail-closed tranche tests for completeness, classification drift, evidence-path drift, and unchanged coverage.
- Regenerate the debt inventory with eight reviewed rows and 353 unassessed rows without adding any operator-diagnostics marker.

## Impact

- **Files**: typed inventory policy/generated report, bounded tranche-review tests and coverage fixture, repository-validation spec, and lifecycle evidence.
- **Testing**: focused doctor/inventory tests, positive and negative tranche validation, exact missing-set comparison, repository checks, Cairn gates, Octet, and Nix checks.
