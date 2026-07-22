## ADDED Requirements

### Requirement: Exact workflow-example evidence completion
r[lattice.repository_validation.workflow_example_completion] Lattice MUST maintain direct positive and negative fixture evidence for both reviewed image workflow-example scenarios before promotion.

#### Scenario: Invalid image fixture remains intentional
r[lattice.repository_validation.workflow_example_completion.scenario.fixture]
- GIVEN a repository-owned typed Nickel workflow fixture whose image inputs resolve to unsupported GIF bytes
- WHEN the fixture is imported
- THEN its invalid intent and affected image nodes MUST remain explicit and reviewable

#### Scenario: Invalid image execution leaves no partial completion
r[lattice.repository_validation.workflow_example_completion.scenario.negative]
- GIVEN the invalid image fixture and deterministic in-memory image bytes
- WHEN it runs through the executor
- THEN maintained assertions MUST observe unsupported-format diagnostics, skipped downstream execution, and zero completed nodes
