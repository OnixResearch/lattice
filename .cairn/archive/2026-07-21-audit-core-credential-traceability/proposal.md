## Why

The maintained traceability inventory contains 26 unassessed credential gaps. The core credential boundary already has substantial source and test evidence, but parent-level markers do not establish the exact accepted scenarios and several assertions omit required save/export, diagnostic, or multi-surface redaction clauses. A bounded review is needed before any behavior change or marker promotion.

## What Changes

- Review 12 core credential scenarios covering secret references, provider resolution, redaction, validation, cache identity, registry resolution, and environment providers.
- Record eight exact maintained-evidence rows and four partial-evidence rows in typed Nickel policy.
- Bind every reviewed row to current safe repository-relative file paths and BLAKE3 file identities.
- Add positive and fail-closed tranche tests for completeness, classifications, evidence drift, and unchanged missing coverage.
- Regenerate the debt inventory with 12 reviewed rows and 341 unassessed rows without adding credential markers.

## Impact

- **Files**: typed inventory policy/generated report, one bounded coverage fixture, one tranche-review test, repository-validation lifecycle artifacts, and audit evidence.
- **Behavior**: no credential runtime, workflow, provider, persistence, or output behavior changes.
- **Testing**: existing credential tests, positive and negative audit validation, exact missing-set preservation, repository checks, Cairn gates, Octet, and Nix checks.
