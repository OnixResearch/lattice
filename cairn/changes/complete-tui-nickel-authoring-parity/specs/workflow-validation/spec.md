## ADDED Requirements

### Requirement: Aggregate workflow mutation preserves ownership

r[lattice.workflow_validation.aggregate_mutation] Workflow lifecycle mutation MUST preserve aggregate ownership invariants across graph nodes, incident edges, node configuration, credential references, runner settings, trigger references, and TUI-owned selection/edit state instead of mutating the graph in isolation.

#### Scenario: Node deletion removes owned state

r[lattice.workflow_validation.aggregate_mutation.scenario.delete]
- GIVEN a workflow node owns configuration, credentials, runner settings, ports, incident edges, and matching transient editor state
- WHEN a validated workflow-level deletion plan is applied
- THEN all state owned by that node MUST be removed in one mutation
- AND unrelated node, graph, credential, runner, trigger, and editor state MUST remain unchanged.

#### Scenario: Orphan-producing mutation is rejected

r[lattice.workflow_validation.aggregate_mutation.scenario.reject_orphan]
- GIVEN a mutation would leave a config, credential reference, runner entry, port, edge, trigger reference, selection, or edit draft pointing at a missing owner
- WHEN aggregate mutation validation runs
- THEN it MUST reject the mutation before persistence or export
- AND preserve the complete prior workflow and transient state.
