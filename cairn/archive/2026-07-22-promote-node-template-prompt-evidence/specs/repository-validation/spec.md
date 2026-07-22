## ADDED Requirements

### Requirement: Exact node-template prompt evidence promotion
r[lattice.repository_validation.node_template_prompt_promotion] Lattice MUST promote exactly the three completed node-template prompt requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact three-ID promotion delta
r[lattice.repository_validation.node_template_prompt_promotion.scenario.exact_delta]
- GIVEN the reviewed completion baseline with 327 missing requirements
- WHEN node-template prompt evidence is promoted
- THEN the current result MUST contain 324 missing requirements, remove exactly the three allowlisted prompt IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Promoted markers remain assertion-local
r[lattice.repository_validation.node_template_prompt_promotion.scenario.local_binding]
- GIVEN the exact collection, required rendering, and cancellation tests
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur exactly once immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Promotion drift fails closed
r[lattice.repository_validation.node_template_prompt_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, unrelated removals, additions, dangling markers, or successful coverage
- WHEN deterministic promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion
