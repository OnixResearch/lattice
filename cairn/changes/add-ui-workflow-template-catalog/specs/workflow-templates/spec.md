## ADDED Requirements

### Requirement: Typed workflow-template bundle catalog
r[lattice.workflow_templates.catalog] Lattice MUST model UI-loadable workflow templates as typed Nickel bundles, separate from shell node templates, with stable bundle metadata and BLAKE3 identities for the manifest, workflow, optional orchestration definition, and every declared support asset.

#### Scenario: Valid bundle enters the catalog
r[lattice.workflow_templates.catalog.scenario.valid]
- GIVEN a local bundle has a valid typed manifest, exact declared files, matching BLAKE3 identities, a valid workflow, and an optional orchestration definition bound to that exact workflow revision
- WHEN the workflow-template loader evaluates the bundle
- THEN Lattice MUST include one validated bundle in the deterministic catalog with its safe metadata and compatibility requirements

#### Scenario: Invalid bundle is isolated
r[lattice.workflow_templates.catalog.scenario.invalid]
- GIVEN one bundle contains malformed Nickel, a duplicate or unsafe path, an undeclared or missing file, content-identity drift, an invalid workflow, or an orchestration definition bound to another revision
- WHEN the loader evaluates a directory containing that bundle and unrelated valid bundles
- THEN Lattice MUST exclude only the invalid bundle, emit one bounded safe diagnostic, and MUST retain the unrelated valid bundles

### Requirement: Pure workflow-template validation and planning
r[lattice.workflow_templates.core] Lattice MUST validate bundle structure, identities, workflow/orchestration compatibility, and application conflicts through a pure deterministic core over parsed in-memory inputs and explicit filesystem/store facts.

#### Scenario: Exact inputs produce a deterministic draft plan
r[lattice.workflow_templates.core.scenario.plan]
- GIVEN a validated bundle and explicit facts report no conflicting workflow or orchestration identity
- WHEN the pure core plans template application
- THEN it MUST return the same preview and draft identities for equivalent inputs without reading files, stores, clocks, environments, networks, processes, or terminal state

#### Scenario: Conflicting content fails before mutation
r[lattice.workflow_templates.core.scenario.conflict]
- GIVEN an existing workflow or orchestration uses the selected bundle's name with different content or policy identity
- WHEN the pure core plans application
- THEN it MUST return a stable conflict diagnostic and MUST authorize no overwrite, persistence, instance creation, run, or node effect

### Requirement: Capability-bounded local bundle loading
r[lattice.workflow_templates.loading] Lattice MUST load workflow-template files only from a configured local template root through deterministic bounded reads that reject path escape, symlink escape, non-regular files, undeclared files, and hidden network retrieval.

#### Scenario: Declared local files load under bounds
r[lattice.workflow_templates.loading.scenario.local]
- GIVEN every manifest-declared relative file is a regular file beneath the opened template root and all named file, bundle, and catalog limits are satisfied
- WHEN the imperative loader captures the bundle inputs
- THEN it MUST pass only the admitted bytes and safe relative metadata to the pure validation core

#### Scenario: Escaping or oversized file is rejected
r[lattice.workflow_templates.loading.scenario.rejected]
- GIVEN a manifest path is absolute, traverses a parent, resolves through a symlink, names a non-regular file, or exceeds a configured count or byte limit
- WHEN the loader captures that bundle
- THEN it MUST reject the bundle without reading outside the admitted root, executing content, retrieving a network resource, or exposing host paths or file contents in diagnostics

### Requirement: Side-effect-free TUI workflow-template preview
r[lattice.workflow_templates.preview] Lattice MUST provide a distinct TUI workflow-template browser whose selection and preview operations produce only an in-memory unsaved draft and MUST NOT persist or execute workflow or orchestration state.

#### Scenario: Selection opens an unsaved workflow draft
r[lattice.workflow_templates.preview.scenario.select]
- GIVEN the catalog contains a validated bundle and the current canvas can be replaced under the normal dirty-workflow confirmation policy
- WHEN the operator selects and previews that bundle
- THEN the TUI MUST show the exact workflow draft, bundle identity, compatibility requirements, optional orchestration presence, and non-claims without creating a store record, instance, run, agent call, credential lookup, or external effect

#### Scenario: Cancellation preserves prior state
r[lattice.workflow_templates.preview.scenario.cancel]
- GIVEN the operator is browsing or previewing a workflow template
- WHEN the operator cancels before an explicit save or start action
- THEN Lattice MUST preserve the prior workflow, orchestration, store, configuration, and template bytes and MUST perform no node effect

### Requirement: Explicit workflow-template activation admission
r[lattice.workflow_templates.activation] Lattice MUST save or start content originating from a workflow-template bundle only through explicit operator actions that reuse current workflow and orchestration admission against the exact current identities.

#### Scenario: Exact admitted draft may start
r[lattice.workflow_templates.activation.scenario.start]
- GIVEN the operator explicitly saved the exact validated workflow revision and requests start of its exact optional orchestration draft with admitted actor authority, runtimes, effects, credentials, triggers, and positive budgets
- WHEN activation admission succeeds
- THEN Lattice MAY create one durable orchestration instance and MUST record the exact bundle, workflow, orchestration, policy, and actor authority identities before dispatch

#### Scenario: Drift or missing authority blocks start
r[lattice.workflow_templates.activation.scenario.blocked]
- GIVEN the saved workflow, orchestration, runtime, effect, credential, trigger, budget, or authority identity differs from the previewed and admitted bundle or a required admission fact is missing
- WHEN the operator requests activation
- THEN Lattice MUST reject the request without creating an instance, run, budget reservation, agent call, or other node effect

#### Scenario: Selection cannot auto-start
r[lattice.workflow_templates.activation.scenario.no-auto-start]
- GIVEN a bundle contains an otherwise valid orchestration definition
- WHEN the operator only opens, selects, previews, or saves its workflow draft
- THEN Lattice MUST NOT infer start authority, approve a proposal, reserve budget, or dispatch a workflow round

### Requirement: Non-destructive workflow-template initialization
r[lattice.workflow_templates.init] Lattice MUST install built-in workflow-template bundle files only during fresh create-only initialization and MUST NOT replace an existing config, node template, workflow template, manifest, workflow, orchestration, role, or support asset path.

#### Scenario: Fresh initialization writes complete bundles
r[lattice.workflow_templates.init.scenario.fresh]
- GIVEN the requested configuration and workflow-template target paths do not exist
- WHEN initialization succeeds
- THEN Lattice MUST create the default config, existing node-template catalog, and every built-in workflow-template bundle with exact reviewed bytes

#### Scenario: Existing target remains unchanged
r[lattice.workflow_templates.init.scenario.existing]
- GIVEN any target path already contains operator-authored bytes
- WHEN initialization attempts exclusive creation
- THEN Lattice MUST fail visibly before truncating or replacing that path and MUST preserve all pre-existing bytes

### Requirement: Existing template and execution compatibility
r[lattice.workflow_templates.compatibility] Lattice MUST preserve existing shell node-template loading/rendering, stored workflow opening, workflow import/export, headless execution, and orchestration CLI semantics when the workflow-template catalog is unused.

#### Scenario: Existing node templates retain shell behavior
r[lattice.workflow_templates.compatibility.scenario.nodes]
- GIVEN a current shell node template and valid parameter values
- WHEN the existing `Ctrl-N` template flow creates the node after workflow-template support is added
- THEN it MUST render and create the same shell node configuration without workflow-bundle or orchestration semantics

#### Scenario: Existing CLI routes remain unchanged
r[lattice.workflow_templates.compatibility.scenario.cli]
- GIVEN an existing workflow or orchestration definition is imported, run, validated, started, inspected, or replayed through current CLI commands
- WHEN no workflow-template bundle is selected
- THEN Lattice MUST retain the same validation, persistence, execution, evidence, and exit behavior
