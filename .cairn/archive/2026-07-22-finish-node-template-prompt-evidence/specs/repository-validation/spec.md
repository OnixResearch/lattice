## ADDED Requirements

### Requirement: Exact node-template prompt evidence completion
r[lattice.repository_validation.node_template_prompt_completion] Lattice MUST maintain direct assertion-bound evidence for every clause in the three reviewed node-template prompt requirements before any marker promotion.

#### Scenario: Required prompt rendering is directly asserted
r[lattice.repository_validation.node_template_prompt_completion.scenario.rendered]
- GIVEN an add-node flow positioned at a required template parameter before node creation
- WHEN the public bounded-buffer renderer exposes the pure parameter-line core's user-visible output
- THEN maintained assertions MUST inspect the parameter sequence, name, type, required label, description, current value, and controls, while a negative companion MUST reject a required label for an optional parameter

#### Scenario: Complete prompt tranche becomes exact without promotion
r[lattice.repository_validation.node_template_prompt_completion.scenario.complete]
- GIVEN direct maintained collection, rendering, cancellation, and no-partial-state assertions
- WHEN the completion inventory is generated
- THEN exactly the three node-template prompt rows MUST be `exact_maintained_evidence`, bind current safe file-byte identities, and remain absent from source markers

#### Scenario: Completion drift fails closed
r[lattice.repository_validation.node_template_prompt_completion.scenario.fail_closed]
- GIVEN missing or unexpected rows, non-exact classification, rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion
