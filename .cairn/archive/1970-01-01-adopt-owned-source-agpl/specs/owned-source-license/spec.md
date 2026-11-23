# Owned source licensing

## ADDED Requirements

### Requirement: Prospective owned-source grant
r[lattice.owned_source_license.grant] All new Lattice-owned source, including reusable cores, contracts, schemas, fixtures, templates, documentation and policy, MUST use AGPL-3.0-or-later; current package metadata, package-local grants and owned fixture license declarations MUST agree.

#### Scenario: Current reusable source is consumed
r[lattice.owned_source_license.grant.scenario.current]
- GIVEN a consumer selects the newly changed first-party source
- WHEN package metadata and its local license are inspected
- THEN both MUST identify AGPL-3.0-or-later without offering a new permissive alternative.

### Requirement: Historical grants and source boundaries
r[lattice.owned_source_license.boundary] Lattice MUST preserve prior grants, copyright attribution, immutable imported source, historical receipts, source pins, lockfiles and exact historical dependency license assertions. This preparation MUST NOT refresh Kiln's imported contract, publish a revision or imply anonymous acquisition.

#### Scenario: An old MIT grant remains effective
r[lattice.owned_source_license.boundary.scenario.historical]
- GIVEN content previously received under MIT, including byte-identical content still present
- WHEN current prospective declarations change
- THEN those previous rights MUST remain available under the original grant; imported dependencies retain their exact-source terms.
