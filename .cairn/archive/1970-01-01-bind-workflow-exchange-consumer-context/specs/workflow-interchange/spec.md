# Workflow Exchange Consumer Context Delta

## MODIFIED Requirements

### Requirement: Typed source context

r[lattice.workflow_interchange.source_context] Workflow request schema version 2 MAY carry one bounded provider, repository, and revision source context, and its version 2 identity MUST bind every supplied source field.

#### Scenario: Source revision changes
r[lattice.workflow_interchange.source_context.scenario.identity]
- GIVEN two otherwise equal workflow requests with different source revisions
- WHEN request identity framing runs
- THEN the requests MUST receive different identities.

## ADDED Requirements

### Requirement: Explicit route source binding

r[lattice.workflow_interchange.source_binding] A handler route MUST explicitly reject source context or bind it to fixed environment keys on every exact mapped shell node.

#### Scenario: Source-bound route is admitted
r[lattice.workflow_interchange.source_binding.scenario.admitted]
- GIVEN a source-bound request and a route whose mapped workflow nodes are eligible shell nodes
- WHEN handler admission runs
- THEN Lattice MUST create one instantiated workflow with the exact source values.

#### Scenario: Reserved environment key exists
r[lattice.workflow_interchange.source_binding.scenario.collision]
- GIVEN a saved workflow node already defines one reserved source environment key
- WHEN source binding runs
- THEN Lattice MUST reject before durable acceptance or effects.

### Requirement: Instantiated revision authority

r[lattice.workflow_interchange.source_authority] Durable acceptance and route effect authority MUST bind the exact instantiated workflow revision produced from the saved revision and admitted source context.

#### Scenario: Source context changes after acceptance
r[lattice.workflow_interchange.source_authority.scenario.drift]
- GIVEN one accepted source-bound request
- WHEN another request changes a source fact
- THEN it MUST require a different request, instantiated workflow, acceptance, and authority identity.
