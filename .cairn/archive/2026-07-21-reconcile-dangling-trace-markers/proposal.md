# Change: Reconcile dangling Tracey markers

## Why

Correct Lattice source discovery exposes eight marker identifiers that have no accepted requirement. Leaving them dangling hides whether each marker names an obsolete alias, an underspecified accepted behavior, or evidence that does not prove the nearest accepted scenario.

## What Changes

- Add one narrow accepted scenario for the already-tested rejection of numeric template parameters above their declared maximum.
- Replace the stale trigger persistence alias with the accepted save scenario and strengthen the test to compare the complete persisted trigger definition.
- Bind provider-backed and redaction-safe authentication tests only to the accepted parent requirement they actually support.
- Bind the headless trigger-plan implementation to the already accepted listener-admission scenario it implements, while removing stale admission and webhook method/content-type aliases instead of promoting partial tests to scenario evidence.
- Add focused repository tests that require the new accepted marker and reject every retired dangling identifier.
- Preserve all 369 pre-existing missing requirements, `valid: false`, and an unsuccessful coverage exit while reducing dangling markers from eight to zero.

## Capabilities

### Modified Capabilities

- `node-templates`: Record the exact above-maximum parameter rejection already verified by the template renderer test.
- `repository-validation`: Require reviewed marker reconciliation to remove stale identifiers without manufacturing accepted coverage.

## Impact

- **Files**: `tests/test_template_parameters.rs`, `tests/test_triggers.rs`, a focused traceability alignment test, accepted Cairn specs, and lifecycle evidence.
- **Behavior**: Runtime behavior is unchanged; one persistence assertion becomes stronger.
- **Evidence**: Tracey keeps real missing coverage fail-closed while reporting no dangling marker identifiers.
- **Non-claims**: This change does not prove the remaining requirements, semantic correctness of every retained marker, complete test adequacy, or release readiness.
