# Design: promote exact webhook authentication evidence

## Completion evidence

Baseline missing coverage MUST equal 329, current missing coverage MUST equal 327, baseline minus current MUST equal the exact two-ID allowlist, current minus baseline MUST be empty, both receipts MUST remain unsuccessful with zero dangling markers, both markers MUST be locally attached, and current debt policy MUST contain neither ID.

## Bindings

The valid marker attaches to the admitted execution/history/raw-store test. The redaction marker attaches to the rejected response/history/raw-store and fingerprint-context test.

Indirect lifecycle tests, implementation names, generated inventory, and file scope are rejected.

## Fail-closed checks

Deterministic checks reject duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, successful-coverage, missing-binding, and stale-policy mutations.

## Non-claims

Promotion proves linkage only, not remote authenticity, fingerprint confidentiality, arbitrary secret confidentiality, complete traceability, release readiness, or production readiness.
