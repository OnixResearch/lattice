## Context

The current persisted model contains metadata, graph nodes/typed ports/edges, eight `NodeConfig` variants, per-node credential maps, per-node runner settings and effect/cache/retry/artifact policies, and trigger definitions. Nickel import/export is intended to represent this model. The TUI stores the complete in-memory workflow, so untouched imported fields can survive, but direct authoring covers only shell, image, agent, condition, merge, and subworkflow creation and selected scalar properties.

Mutation currently crosses the model boundary in two places. Canvas Delete/Backspace delegates to graph state and copies only the graph back into `Workflow`. Condition branch editing rebuilds output ports by zipping branches with old ports. Both bypass workflow-level invariants.

## Success Contract

Completion requires a checked capability matrix with no unsupported field in the current portable schema, typed TUI creation/edit paths, atomic workflow-level mutation, and cross-surface round-trip evidence. Merely preserving imported bytes, adding HTTP/File menu labels, exposing raw JSON/Nickel replacement, or passing existing shell round-trip tests is insufficient.

False completion includes:

- a field displayed but not editable;
- an editor that mutates before validation or loses the prior value on cancellation;
- a generic string field whose parser accepts malformed nested policy silently;
- deleted-node config/credentials/runner state surviving in memory or store;
- branch labels and output ports having different cardinality;
- changing a branch label while silently retargeting an edge to a different semantic branch;
- tests that enumerate fields without exercising TUI input and Nickel round trip.

## Architecture

```text
PortableWorkflowCapabilityCatalog (pure)
  metadata / node kinds / config fields / ports / runners / credentials / triggers
                  |
                  v
Typed editor descriptors + pure parse/validate/apply functions
                  |
                  v
TUI shells: browse -> draft -> validate -> atomic apply | cancel
                  |
                  v
Workflow lifecycle core
  remove_node_plan() / reconcile_condition_branches() / apply_*()
                  |
                  v
persisted Workflow <-> portable Nickel parity fixtures
```

The catalog identifies field id, owner, value kind, required/optional/repeated shape, display label, edit help, parser, validator, and mutation target. The core consumes in-memory drafts and explicit current workflow facts. TUI handlers own keystrokes, focus, modals, and rendering only.

## Decisions

### Decision: make the portable schema the parity reference

**Choice:** Maintain a pure capability catalog and test matrix covering workflow metadata, graph ports/edges, all `NodeConfig` fields, credentials, runner settings/policies, and triggers currently represented by `PortableWorkflow`. Every row is classified `create`, `edit`, `remove/reset`, and `round_trip`; no `display-only` row can satisfy completion.

**Rationale:** A shared Rust struct does not prove UI authoring coverage. A checked matrix makes omissions visible when the schema evolves.

### Decision: use typed editors, not raw source replacement

**Choice:** Scalar fields use typed text/enum/boolean/positive-number editors. Maps, lists, policies, port declarations, mappings, and triggers use explicit row-oriented draft models with add/edit/remove operations and pure validation. The TUI may render compact serialized summaries, but cannot commit an unvalidated raw Nickel or JSON replacement.

**Rationale:** Raw source editing would evade recoverable field validation and turn parity into a second import command.

### Decision: route field metadata through providers and owner catalogs

**Choice:** Node-specific configuration fields come from node-kind provider metadata. Workflow metadata, graph ports, runner policy, credentials, and triggers use parallel owner-specific catalogs. One dispatcher opens the appropriate typed draft and one pure apply function performs validation before mutation.

**Rationale:** Extending scattered `match NodeConfig` presentation branches would make future parity drift likely and difficult to test.

### Decision: centralize node lifecycle mutation

**Choice:** `Workflow` exposes a pure `plan_remove_node` over current graph/config/credential/runner/trigger facts and an atomic `remove_node` shell that applies the plan only after validation. It removes the graph node and incident edges plus all per-node maps. Trigger definitions are retained only when they reference the workflow rather than a deleted node; future node-owned trigger references must be rejected or removed explicitly. TUI state clears focused/selected/property drafts for the removed identity.

**Rationale:** Graph mutation alone cannot maintain workflow aggregate invariants.

### Decision: reconcile condition outputs by branch identity

**Choice:** Branch labels are the semantic identity in the current format. Reconciliation first validates non-empty unique labels and predicates. It preserves existing `PortId`, type tag, and outgoing edges for exactly matching labels; allocates deterministic fresh output ports for new labels; removes ports and incident edges for deleted labels; and treats rename as remove-plus-add so edges are not silently retargeted. Reorder changes display order while preserving matching identities. Any failure returns no mutation.

**Rationale:** Positional `zip` conflates cardinality and semantics. Label-based reconciliation is deterministic and conservative.

### Decision: allocate identifiers through one workflow core

**Choice:** New node and port ids use workflow-owned allocation based on explicit current ids with checked overflow/collision rejection. TUI code never invents ids or reuses a removed id implicitly.

**Rationale:** Port editing, condition reconciliation, and import parity require one collision-free source of identity.

### Decision: preserve imported hidden fields throughout migration

**Choice:** Existing store load/save remains whole-workflow serialization. Editors mutate only their owned field. Until every catalog row ships, unsupported action attempts fail without resetting hidden values; however the change cannot close while any current portable row remains unsupported.

**Rationale:** Incremental implementation must not create new data loss before final parity.

## Field Families

### Workflow and graph

- name, description, created/updated metadata under explicit timestamp policy;
- node label/position and typed input/output port id, label, direction, type tag;
- compatible edge add/remove;
- node credentials;
- trigger add/edit/remove for manual, schedule, and webhook variants.

### Node configuration

- Shell: command, cwd, environment map, stdin piping, timeout;
- HTTP: method, URL, headers, body, timeout;
- File: operation, path, content, allowed roots;
- Image input: path, allowed roots, maximum bytes;
- Agent: prompt/source, target, output, model, tool policy, context refs, cwd, session, credentials;
- Condition: branch labels and predicates with reconciled output topology;
- Merge: policy and minimum input count;
- Subworkflow: workflow name and input/output mappings.

### Runner metadata

- runtime kind/key;
- declared abilities/effects;
- cache mode;
- retry count and backoff policy;
- artifact reference policy and configured limits.

Numeric literals are represented by named bounds/default constants and checked conversion; invalid/overflowing input never mutates state.

## Adversarial Audit

| Family | Mechanism | Result |
|---|---|---|
| Preservation only | Continue storing the whole imported workflow | Rejected: does not provide authoring parity. |
| Raw Nickel editor | Let users replace the workflow source in a text modal | Rejected: bypasses typed/recoverable editing and duplicates import. |
| Expand ad hoc matches | Add fields directly to existing render/input matches | Rejected as the sole architecture: omissions remain uncheckable. |
| Catalog + typed drafts | Enumerate portable fields and route typed drafts through pure apply/lifecycle cores | Selected. |

The surviving route is audited against empty/duplicate ids, numeric overflow, enum drift, malformed maps/lists/mappings/triggers, cancellation at every draft stage, node deletion during selection/edit, orphan maps, dangling edges, branch add/remove/rename/reorder, edge preservation for unchanged labels, identifier exhaustion, imported hidden fields, save failure, and constrained terminal dimensions.

## Compatibility and Non-claims

Existing keyboard shortcuts, workflows, store bytes, and Nickel files remain supported. New editor state is transient and does not change persistence format. The change does not implement SecretSpec/SOPS resolution or fix upstream provider behavior.

Parity means the current portable fields can be created, edited, removed/reset where meaningful, persisted, exported, and imported with equivalent typed meaning. It does not mean identical visual layout, arbitrary future plugin support, workflow correctness, provider correctness, safe external effects, or production readiness.
