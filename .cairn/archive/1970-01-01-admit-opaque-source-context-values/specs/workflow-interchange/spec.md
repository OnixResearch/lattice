# Opaque Source Context Value Delta

## MODIFIED Requirements

### Requirement: Typed source context

r[lattice.workflow_interchange.source_context] Workflow request schema version 2 MAY carry one bounded provider, repository, and revision source context. Its version 2 identity MUST bind every supplied source byte without provider-identity normalization.

#### Scenario: Source revision changes
r[lattice.workflow_interchange.source_context.scenario.identity]
- GIVEN two otherwise equal workflow requests with different source revisions
- WHEN request identity framing runs
- THEN the requests MUST receive different identities.

#### Scenario: Mixed-case repository identity arrives
r[lattice.workflow_interchange.source_context.scenario.opaque]
- GIVEN a nonempty bounded provider-native repository identity with mixed case
- WHEN source-context admission runs
- THEN Lattice MUST preserve and admit its exact bytes.

#### Scenario: Source identity contains a control byte
r[lattice.workflow_interchange.source_context.scenario.control]
- GIVEN a repository or revision source value with an ASCII control byte
- WHEN source-context decoding runs
- THEN Lattice MUST reject the value before request admission.
