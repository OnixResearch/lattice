# UCAN token flow

## ADDED Requirements

### Requirement: Admit an exact vendored UCAN source revision
r[ucan_token_flow.vendored_source.exact_revision] The Basalt vendored UCAN importer MUST accept a full reviewed Git object ID for source parity and import operations.

The importer MUST reject moving references, abbreviated object IDs, malformed values, and unavailable commits before it replaces the vendored snapshot.

#### Scenario: Exact reviewed revision is admitted
r[ucan_token_flow.vendored_source.exact_revision.admitted]
- GIVEN an explicit UCAN repository and a full lowercase Git object ID that exists in that repository
- WHEN the operator runs exact source parity or import
- THEN the importer archives that exact revision and records it in the snapshot manifest

#### Scenario: Moving or malformed revision is rejected
r[ucan_token_flow.vendored_source.exact_revision.rejected]
- GIVEN a branch name, abbreviated object ID, uppercase object ID, malformed value, or unavailable commit
- WHEN the operator requests exact source parity or import
- THEN the importer fails before it replaces the vendored snapshot
