## ADDED Requirements

### Requirement: Portable workflow authoring parity

r[lattice.tui_ux.portable_authoring_parity] The TUI MUST provide typed create, edit, remove or reset where meaningful, validation, cancellation, persistence, and Nickel round-trip paths for every field in the current portable workflow schema rather than relying on preservation of unexposed imported state.

#### Scenario: Complete portable workflow is authored

r[lattice.tui_ux.portable_authoring_parity.scenario.complete]
- GIVEN the current portable schema includes workflow metadata, graph nodes/ports/edges, all built-in node configs, credentials, runner settings/policies, and triggers
- WHEN an operator authors a workflow through TUI typed flows
- THEN every portable field MUST have a discoverable create/edit/remove-or-reset path with validation and cancellation
- AND the resulting in-memory workflow MUST be representable by typed Nickel without hidden manual source edits.

#### Scenario: Imported preservation does not satisfy authoring

r[lattice.tui_ux.portable_authoring_parity.scenario.preservation_only]
- GIVEN an imported workflow contains a portable field that whole-workflow store serialization would preserve
- WHEN the parity catalog finds no typed TUI authoring action for that field
- THEN parity validation MUST fail with the missing field identity
- AND display-only or untouched save/load behavior MUST NOT satisfy the requirement.

### Requirement: Checked authoring capability catalog

r[lattice.tui_ux.portable_authoring_parity.catalog] Lattice MUST maintain a pure checked capability catalog that covers every current portable field and binds field owner, value kind, create/edit/remove-or-reset support, parser, validation, TUI action, persistence, and Nickel round-trip evidence.

#### Scenario: Catalog covers schema exactly

r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.complete]
- GIVEN the portable schema and authoring catalog from the same source revision
- WHEN completeness validation runs
- THEN every schema field MUST map to exactly one owned capability row with all required evidence classes
- AND duplicate, stale, or unknown rows MUST fail.

#### Scenario: Missing capability blocks closeout

r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.missing]
- GIVEN a portable field is added or an existing capability row is removed or marked display-only
- WHEN parity validation runs
- THEN it MUST report the exact uncovered field and fail closeout.

### Requirement: All built-in node kinds are creatable

r[lattice.tui_ux.portable_authoring_parity.node_kinds] The guided TUI add flow MUST create Shell, HTTP, File, ImageInput, Agent, Condition, Merge, and Subworkflow nodes from validated typed drafts and MUST create no node when a draft is cancelled or invalid.

#### Scenario: HTTP and file nodes are created

r[lattice.tui_ux.portable_authoring_parity.node_kinds.scenario.http_file]
- GIVEN the operator opens the guided add flow
- WHEN valid HTTP or file node fields are entered and confirmed
- THEN the TUI MUST create the selected node kind with matching portable configuration and typed ports
- AND mark the workflow unsaved.

#### Scenario: Invalid or cancelled creation preserves workflow

r[lattice.tui_ux.portable_authoring_parity.node_kinds.scenario.cancel]
- GIVEN any built-in node creation draft is active
- WHEN validation fails or the operator cancels
- THEN the workflow graph, configs, credentials, runner settings, triggers, and dirty state MUST preserve their prior values.

### Requirement: Every built-in node field is editable

r[lattice.tui_ux.portable_authoring_parity.node_fields] Node-kind provider metadata and typed editors MUST cover every current portable Shell, HTTP, File, ImageInput, Agent, Condition, Merge, and Subworkflow configuration field with pure parsing/validation before atomic mutation.

#### Scenario: Valid node field edit commits

r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.edit]
- GIVEN one selected built-in node and a supported configuration field
- WHEN the operator supplies a valid typed scalar, enum, optional, list, map, mapping, policy, path, or bounded number value
- THEN the TUI MUST update only that field, preserve unrelated imported state, mark the workflow unsaved, and display the new typed value.

#### Scenario: Invalid node field edit is recoverable

r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.invalid]
- GIVEN a node field edit contains malformed syntax, unsupported enum, duplicate key, unsafe path, empty required value, overflow, or out-of-range number
- WHEN the operator attempts to save
- THEN the pure validator MUST reject it without mutating any workflow field
- AND the TUI MUST preserve the prior value and a recoverable edit state.

### Requirement: Workflow and graph fields are editable

r[lattice.tui_ux.portable_authoring_parity.workflow_graph] The TUI MUST expose typed editors for current portable workflow metadata, node labels/positions, input/output port declarations, compatible edges, and per-node credential references with checked identity allocation and collision rejection.

#### Scenario: Graph metadata and ports are authored

r[lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.edit]
- GIVEN a workflow draft and selected workflow or node graph owner
- WHEN the operator edits metadata, position, a typed port, edge, or opaque credential reference with valid values
- THEN Lattice MUST apply the exact typed change and preserve all unrelated graph and workflow state.

#### Scenario: Invalid graph edit is atomic

r[lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.invalid]
- GIVEN an edit would duplicate an id or label where prohibited, overflow identity allocation, create a wrong-direction/self/incompatible edge, orphan an incident edge, or introduce a malformed credential reference
- WHEN validation runs
- THEN Lattice MUST reject the complete edit without partial graph, config, credential, runner, or trigger mutation.

### Requirement: Runner policy fields are editable

r[lattice.tui_ux.portable_authoring_parity.runner] The TUI MUST expose typed per-node editing for current portable runtime kind/key, ability and effect requirements, cache policy, retry and backoff policy, artifact-reference policy, and associated named bounds.

#### Scenario: Valid runner policy is stored

r[lattice.tui_ux.portable_authoring_parity.runner.scenario.edit]
- GIVEN a selected node and a valid supported runtime/policy draft
- WHEN the operator confirms the edit
- THEN Lattice MUST store equivalent runner settings and preserve them through save/load and Nickel export/import.

#### Scenario: Unsupported runner policy is rejected

r[lattice.tui_ux.portable_authoring_parity.runner.scenario.invalid]
- GIVEN a runner draft has a missing binding, unknown policy, duplicate ability/effect, zero or excessive bound, overflow, or incompatible artifact rule
- WHEN the operator attempts to save
- THEN Lattice MUST reject it without changing the previous runner settings.

### Requirement: Trigger definitions are editable without activation

r[lattice.tui_ux.portable_authoring_parity.triggers] The TUI MUST let operators browse, add, edit, disable, and remove current portable manual, schedule, and webhook trigger definitions through typed validated drafts without starting a listener, schedule, workflow run, or other effect during authoring.

#### Scenario: Valid trigger draft commits

r[lattice.tui_ux.portable_authoring_parity.triggers.scenario.edit]
- GIVEN the current workflow and a valid manual, schedule, or webhook trigger draft
- WHEN the operator confirms the draft
- THEN Lattice MUST store the trigger definition with equivalent workflow reference, input, authentication reference, overlap, and idempotency policy
- AND it MUST perform no trigger or workflow effect.

#### Scenario: Invalid or cancelled trigger draft is inert

r[lattice.tui_ux.portable_authoring_parity.triggers.scenario.invalid]
- GIVEN a trigger draft is malformed, duplicates an identity, targets the wrong workflow, contains an invalid schedule/path/auth reference/policy, or is cancelled
- WHEN the draft closes
- THEN the prior trigger set and store MUST remain unchanged and no listener, scheduler, run, or credential resolution may start.

### Requirement: Aggregate node deletion

r[lattice.tui_ux.aggregate_node_deletion] TUI node deletion MUST use one workflow-level lifecycle mutation that removes the graph node, incident edges, node configuration, credentials, runner settings, invalid node-owned references, and matching TUI focus/selection/edit state.

#### Scenario: Configured node is deleted cleanly

r[lattice.tui_ux.aggregate_node_deletion.scenario.delete]
- GIVEN a selected node has incident edges, configuration, credential references, runner settings, and an active property draft
- WHEN the operator confirms deletion
- THEN one aggregate mutation MUST remove every state item owned by that node and clear its TUI state
- AND workflow validation, store save/load, and Nickel export/import MUST report no orphan state.

#### Scenario: Missing deletion target changes nothing

r[lattice.tui_ux.aggregate_node_deletion.scenario.missing]
- GIVEN the requested node identity is absent or the delete plan is stale
- WHEN deletion is applied
- THEN Lattice MUST reject or no-op deterministically without removing state owned by another node or changing dirty state.

### Requirement: Condition topology reconciliation

r[lattice.tui_ux.condition_topology_reconciliation] Condition branch editing MUST reconcile output ports by validated branch label identity, preserving matching port ids and edges, creating ports for new labels, removing deleted ports and incident edges, treating rename as remove-plus-add, and applying no mutation on failure or cancellation.

#### Scenario: Branch cardinality changes atomically

r[lattice.tui_ux.condition_topology_reconciliation.scenario.change]
- GIVEN a condition node has valid branches, output ports, and outgoing edges
- WHEN a valid edit adds, removes, renames, or reorders branches
- THEN unchanged labels MUST retain port identities and incident edges, new labels MUST receive fresh collision-free ports, deleted labels MUST lose ports and incident edges, and output order MUST match the edited branch order
- AND workflow validation MUST observe one matching output per branch.

#### Scenario: Invalid branch edit preserves topology

r[lattice.tui_ux.condition_topology_reconciliation.scenario.reject]
- GIVEN a branch edit has an empty or duplicate label, malformed predicate, identifier exhaustion, stale node, or is cancelled
- WHEN reconciliation runs
- THEN it MUST reject the edit without changing branch config, ports, edges, focus, or dirty state.

### Requirement: TUI-to-Nickel parity round trip

r[lattice.tui_ux.portable_authoring_parity.round_trip] Lattice MUST maintain exhaustive fixtures that author every current capability through TUI actions and prove equivalent in-memory, store, exported Nickel, and re-imported workflow meaning.

#### Scenario: Complete authored workflow round trips

r[lattice.tui_ux.portable_authoring_parity.round_trip.scenario.complete]
- GIVEN a TUI-authored workflow exercises all eight node kinds, every portable field family, typed ports/edges, credentials, runner policies, control-flow mappings, and all trigger kinds
- WHEN it is saved, loaded, exported to Nickel, and imported
- THEN the resulting workflow MUST preserve equivalent portable meaning and validate without fatal diagnostics.

#### Scenario: Invalid authored workflow does not partially persist

r[lattice.tui_ux.portable_authoring_parity.round_trip.scenario.invalid]
- GIVEN an authoring draft, deletion plan, or condition reconciliation would make the workflow invalid
- WHEN validation or persistence admission runs
- THEN Lattice MUST preserve the prior in-memory workflow and store bytes and MUST NOT emit a misleading importable export.

### Requirement: Complete authoring is discoverable

r[lattice.tui_ux.portable_authoring_parity.discoverability] Command palette, contextual help, and owner-specific property surfaces MUST make every authoring field family discoverable while remaining usable at constrained terminal dimensions.

#### Scenario: Operator finds every editor family

r[lattice.tui_ux.portable_authoring_parity.discoverability.scenario.help]
- GIVEN the TUI is focused on the canvas, workflow, node, runner, credential, or trigger owner
- WHEN contextual help or the command palette is rendered
- THEN it MUST expose the corresponding create/edit/remove actions without requiring undocumented key knowledge.

### Requirement: Authoring parity verification includes failures

r[lattice.tui_ux.portable_authoring_parity.verification] Completion MUST pair positive field-family, aggregate deletion, condition reconciliation, store, Nickel, and TUI/PTY fixtures with negative malformed, duplicate, overflow, stale, cancelled, orphan, dangling-edge, hidden-field, save-failure, and constrained-render fixtures.

#### Scenario: Closeout evidence covers the parity matrix

r[lattice.tui_ux.portable_authoring_parity.verification.scenario.closeout]
- GIVEN maintainers intend to archive the parity change
- WHEN closeout validation runs
- THEN evidence MUST show a complete capability matrix, focused positive and negative tests, repository checks, Octet result, Cairn validation/gates, and selected Nix checks
- AND every skipped rail MUST include an exact blocker and next-best check.
