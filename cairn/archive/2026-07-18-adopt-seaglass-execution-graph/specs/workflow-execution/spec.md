## ADDED Requirements

### Requirement: Seaglass execution graph core adoption
r[n8nclone.workflow_execution.seaglass_graph_core] The system MUST use the provenance-recorded Seaglass execution-graph core for reusable DAG admission and lifecycle decisions while retaining `n8nclone` runtime, value, policy, event, and artifact ownership.

#### Scenario: Existing runtime behavior remains compatible
r[n8nclone.workflow_execution.seaglass_graph_core.scenario.compatibility]
- GIVEN an existing valid workflow contains shell, HTTP, file, image, condition, merge, subworkflow, Pi, or Clankers nodes
- WHEN runner admission and execution use the shared graph core
- THEN deterministic order and readiness MUST remain compatible and typed runtime dispatch MUST stay in the `n8nclone` shell

#### Scenario: Vendored core records exact provenance
r[n8nclone.workflow_execution.seaglass_graph_core.scenario.provenance]
- GIVEN the Seaglass owner revision is not available through an authorized remote push
- WHEN the core snapshot is synchronized into `n8nclone`
- THEN a repo-owned import tool MUST record the exact source revision and product builds MUST NOT depend on a durable sibling path

### Requirement: LLM operating graph
r[n8nclone.workflow_execution.llm_operating_graph] The system MUST support role-based LLM workflows as admitted typed DAGs with explicit runtime, state, effect, and artifact boundaries.

#### Scenario: Role graph fans out and aggregates
r[n8nclone.workflow_execution.llm_operating_graph.scenario.fanout]
- GIVEN a workflow contains Markdown-backed research, build, review, or publishing agent roles with typed dependencies
- WHEN the graph runs through an injected or production agent adapter
- THEN independent ready roles MAY execute as admitted fan-out and downstream aggregation MUST wait for required predecessors

#### Scenario: Feedback remains between runs
r[n8nclone.workflow_execution.llm_operating_graph.scenario.feedback-boundary]
- GIVEN metrics, replies, failures, budget facts, or approval outcomes suggest a graph change
- WHEN the operating graph reorganizes
- THEN the change MUST be represented as explicit next-run state or graph input and MUST NOT bypass cycle admission, authority policy, or durable event recording

#### Scenario: Production file boundaries execute explicitly
r[n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
- GIVEN a role graph reads shared state and writes its final report through file nodes with explicit allowed roots
- WHEN the graph runs through the default headless production shell
- THEN local file operations MUST execute through the filesystem interface, MUST reject paths outside canonical allowed roots, and MUST leave downstream roles and effects blocked after a failed dependency

### Requirement: Rats dependency readiness
r[n8nclone.workflow_execution.rats_dependency] The system MUST consume the newest Rats source that is complete and passes the focused consumer compile checks.

#### Scenario: Source-distribution defect fails visibly
r[n8nclone.workflow_execution.rats_dependency.scenario.source-completeness]
- GIVEN the newest Rats revision references a source file omitted by ignore or archive rules
- WHEN `n8nclone` compiles against that revision
- THEN the update MUST remain blocked until the Rats owner source includes the file and the focused consumer check passes
