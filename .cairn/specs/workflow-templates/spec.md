# Workflow Templates Specification

## Purpose

Defines the `workflow-templates` capability.

## Requirements

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

### Requirement: Typed workflow-template parameters
r[lattice.workflow_templates.params] A schema 2 workflow-template manifest MUST declare a closed list of typed parameters. Each parameter MUST name a type from `bool`, `integer`, `enum` and `relative_path` with its bounds, either be required without a default or optional with an in-bounds default, and name 1 to 8 shell-node environment targets prefixed `LATTICE_PARAM_`. A pure resolution function MUST reject unknown names, wrong types, out-of-bounds values and missing required values with a diagnostic that names at most a declared parameter and never a supplied value. It MUST return every value with a BLAKE3 identity. Binding MUST write values only as declared environment values, MUST NOT change command text, node kinds or effect grants, and MUST readmit the bound workflow and any orchestration rebound to its exact revision. The bound identity MUST cover the template identity and the resolved-value identity.

#### Scenario: Values outside declared bounds are rejected
r[lattice.workflow_templates.params.scenario.bounds]
- GIVEN a manifest that declares typed parameters with bounds and defaults
- WHEN a caller supplies an undeclared name, a value of another type, a value outside its bounds, a relative path with a parent, absolute, option-like or metacharacter component, or omits a required value
- THEN resolution MUST fail with the matching safe diagnostic and MUST NOT echo the supplied value
- AND equal resolved values MUST have equal identities, and any changed value MUST change the identity

#### Scenario: Unsafe declarations fail at load
r[lattice.workflow_templates.params.scenario.declaration]
- GIVEN a manifest parameter with a duplicate or invalid name, inconsistent bounds, a required default, a missing or out-of-bounds optional default, an open or unknown type shape, or a target key outside `LATTICE_PARAM_`
- OR a target shell node whose command is not one simple command of space-separated safe literals and whole-word `"${KEY}"` references to its bound keys, whose first word is a reference, an assignment, a shell, an argument-executing builtin or a denied wrapper, any of whose later literal words names a shell (all by basename and ignoring ASCII case), or that contains `-exec`, `-execdir`, `-ok`, `-okdir` or `-c`
- OR a node that binds a `LATTICE_PARAM_*` key as a credential, or a template environment that does not hold the rendered default
- OR a target shell node with a `{{…}}` expression in its `cwd` or `env`
- OR a target shell node whose static env or credential bindings have a `BASH_FUNC_*` or `%%` key or a `BASH_ENV`, `SSH_CLIENT`, `SSH2_CLIENT`, `SHLVL`, `HOME`, `SHELLOPTS`, `BASHOPTS` or `PS4` key, or whose static env has a value containing `$` or a backtick; `ENV` MUST stay allowed
- WHEN the loader validates the bundle, including after its workflow is re-pinned, or the template is bound, or a bound bundle is re-verified
- THEN the bundle MUST be rejected with `invalid_param_declaration`, `param_target_expression` for the expression case or `param_target_env` for the environment case (`bound_content_drift` when re-verified), and other bundles MUST remain available
- AND a value MUST NOT take the command-name position of the bound command and MUST NOT be parsed by the shell as shell syntax; the deny list is defense in depth and MUST NOT be claimed to stop a program that executes its own arguments

#### Scenario: Values bind as environment values without widening grants
r[lattice.workflow_templates.params.scenario.binding]
- GIVEN a validated catalog template and accepted values
- WHEN the template is bound
- THEN the bound workflow MUST differ from the template only in the declared environment values and, when those values change the expanded graph, in its source association's state, which becomes `MaterializedFork`; each shell command MUST receive a value as one unchanged argument
- AND authority inspection of the bound workflow MUST list the same grants as the template under the bound workflow revision
- AND save and start MUST refuse an unbound template
- AND a bound bundle whose workflow, values, revision or mode differ from what `bind` admitted MUST be refused at save, activation planning and start before any store write or dispatch, with `mode_violation` for a mode failure and `bound_content_drift` for any other failure at every entry point, and bundle content MUST NOT be writable outside the crate

#### Scenario: Built-in bundles are migrated
r[lattice.workflow_templates.params.scenario.migration]
- GIVEN the four built-in bundles
- WHEN they are loaded and bound to their defaults
- THEN every bundle MUST use schema 2 and `mutating`, the agent-evaluation catalog review MUST declare `catalog_root` for its `--root`, the guarded bundles MUST keep their pinned root without parameters, and binding the defaults MUST reproduce each reviewed workflow and orchestration revision

### Requirement: Declared bundle mode
r[lattice.workflow_templates.mode] A schema 2 manifest MUST declare `mode` as `read_only` or `mutating`. Lattice MUST classify effects conservatively: only condition, merge and approval nodes are pure, only file reads and image inputs are read-only, and every other node, including shell, operation, HTTP of any method, agent, subworkflow and file write or append, is mutating. A `read_only` bundle whose workflow needs `mutating` MUST fail admission at load, at save and at start. The TUI MUST mark every bundle's mode.

#### Scenario: Effects are classified conservatively
r[lattice.workflow_templates.mode.scenario.classification]
- GIVEN shell, HTTP, file read, file write, file append and image input nodes
- WHEN Lattice classifies them
- THEN only the file read and image input MUST be read-only

#### Scenario: A read-only bundle with a mutating effect is refused
r[lattice.workflow_templates.mode.scenario.enforced]
- GIVEN a `read_only` bundle whose workflow contains a mutating node
- WHEN it is loaded, or a bound bundle whose mode no longer admits its workflow is planned for save or start
- THEN load MUST fail with `mode_violation`, and planning MUST return a `ModeViolation` conflict or block before any store write or dispatch
- AND the same workflow under `mutating`, and a file-read-only workflow under `read_only`, MUST load

#### Scenario: Schema 1 manifests keep a compatibility path
r[lattice.workflow_templates.mode.scenario.compatibility]
- GIVEN a schema 1 manifest without `mode` or `params`
- WHEN it is loaded
- THEN it MUST load as `mutating` with no parameters and MUST bind to its defaults
- AND a schema 1 manifest that has either key, including with a `null` value, MUST fail with `unsupported_schema`, a schema 2 manifest without either field MUST fail with `invalid_manifest`, and another schema version or mode value MUST be rejected

### Requirement: Typed run inputs for saved template workflows
r[lattice.workflow_templates.run_inputs] `lattice run NAME` and `lattice authority inspect NAME` MUST accept repeatable `--input NAME=VALUE` and one `--input-file PATH` holding a JSON object. A name repeated within or across both sources MUST be refused. Inputs MUST be typed by the declarations of the exact template the saved workflow was retained from, checked by `params::resolve`, screened by the secret rule, and bound by `workflow_template::bind`, which repeats full bundle admission. Every input failure MUST happen before a run id is reserved, any run record is written, or any node runs, and its diagnostic MUST NOT echo a supplied value or an undeclared name.

#### Scenario: Each declared type binds from either source
r[lattice.workflow_templates.run_inputs.scenario.typed]
- GIVEN a saved workflow retained from a template declaring `bool`, `integer`, `enum` and `relative_path` parameters
- WHEN `--input` text values and `--input-file` JSON values are supplied
- THEN text MUST take the declared type (`true`/`false`, a canonical decimal integer, or text) and JSON values MUST keep their JSON type
- AND the bound workflow MUST hold each value in its declared environment target under a bound workflow revision, with the same effect grants as the saved workflow
- AND equal values from different sources MUST give the same input identity

#### Scenario: Wrong, unknown, missing and duplicate inputs are refused before any effect
r[lattice.workflow_templates.run_inputs.scenario.refused]
- GIVEN a parameterized saved workflow
- WHEN a value has the wrong type or is out of bounds, a name is undeclared, a required value is missing, a name repeats, an argument is not `NAME=VALUE`, or the file is not one JSON object
- THEN the command MUST fail with a safe diagnostic
- AND no node MUST run, no run id MUST be reserved, and no run or input record MUST be written

#### Scenario: Inputless runs are unchanged
r[lattice.workflow_templates.run_inputs.scenario.inputless]
- GIVEN a saved workflow run without `--input` or `--input-file`
- WHEN it runs, is inspected with `runs show`, or is inspected with `authority inspect`
- THEN its run identity, run record, summary bytes, report shape and authority output MUST equal those of the same workflow saved without a template, and no input record MUST exist

### Requirement: Retained template binding
r[lattice.workflow_templates.run_inputs.retained] Saving a bound bundle MUST retain, keyed by the saved workflow name, the bundle id, the template identity and the saved workflow revision. A run with inputs MUST load only the bundle directory the binding names. Inputs MUST be refused with a distinct missing-binding refusal when no binding is retained, and MUST be refused when the template declares no parameters, when the saved workflow revision no longer equals the retained revision, when that bundle fails to load (surfacing its value-free diagnostic) or no longer has the retained template identity, or when the workflow has an approval gate. `--input-file` MUST be opened without blocking and refused unless it is a regular file of at most 64 KiB, and `lattice run` MUST read its inputs before opening the store.

#### Scenario: Only the exact retained template accepts inputs
r[lattice.workflow_templates.run_inputs.retained.scenario.exact]
- GIVEN a saved workflow whose template binding was retained when it was saved
- WHEN inputs are supplied for it
- THEN only the exact retained parameterized template MUST accept them
- AND a template without parameters, a drifted saved workflow or a changed template MUST be refused before any run id is reserved

### Requirement: Run inputs re-verify at the start boundary
r[lattice.workflow_templates.run_inputs.start] A run with inputs is a start path. Before its run id is reserved or its input record is written, the bound bundle MUST pass the same `reverify` as save and start, and the recorded input identity MUST name exactly that bundle id, template, bound identity, workflow revision and value identity. Any difference MUST refuse the run with `bound_content_drift` and MUST leave no run id, run record or input record.

#### Scenario: Drifted bound content never reserves a run
r[lattice.workflow_templates.run_inputs.start.scenario.drift]
- GIVEN an admitted bound bundle whose workflow content was tampered with, or which was swapped for the same template bound to other values
- WHEN the run with inputs starts
- THEN it MUST be refused with `bound_content_drift`
- AND no run id MUST be reserved and no run or input record MUST be written

### Requirement: Recorded input identity
r[lattice.workflow_templates.run_inputs.record] A run with inputs MUST reserve its run id and record its input identity in one store transaction after every input check passed. The identity MUST hold the bundle id, template identity, bound identity, bound workflow revision, the `params::Resolved` value identity, and each declared name with its kind and whether it was supplied. It MUST NOT hold a value.

#### Scenario: A bound run records its input identity without values
r[lattice.workflow_templates.run_inputs.record.scenario.recorded]
- GIVEN a run with admitted inputs
- WHEN it starts
- THEN its input record MUST hold the bundle, template, bound and value identities, the bound workflow revision, and each declared name with its kind and supplied flag
- AND it MUST NOT hold a supplied value

### Requirement: Content-omitting input inspection
r[lattice.workflow_templates.run_inputs.inspection] `runs show` MUST show a recorded input identity as declared names, kinds, supplied flags and BLAKE3 identities only, without a value, workflow name or bundle name, with the report's own identity prefix. An input record that does not belong to the run MUST make the report invalid with `inputs_identity_mismatch`. The identities are unsalted, so the documentation MUST state that values are omitted, not hidden.

#### Scenario: Recorded inputs are shown without values
r[lattice.workflow_templates.run_inputs.inspection.scenario.omitted]
- GIVEN a recorded run with inputs
- WHEN `runs show --output json` reports it
- THEN its `inputs` object MUST show only declared names, kinds, supplied flags and identities, without a value, workflow name or bundle name

### Requirement: Authority inspection of bound inputs
r[lattice.workflow_templates.run_inputs.authority] `authority inspect` with the same inputs as a run MUST admit them the same way without writing state and MUST report the grants of the bound workflow revision with the content-omitting input identity, rendering the bound workflow revision exactly as the report's top-level `workflow_revision`, so a profile written from it authorizes exactly that bound revision.

#### Scenario: Authority inspection reports the bound revision that runs
r[lattice.workflow_templates.run_inputs.authority.scenario.bound]
- GIVEN `authority inspect` with the same inputs as a run
- WHEN an operator profile is written from its output
- THEN that profile MUST admit the run with those inputs
- AND a run with different values MUST be denied

### Requirement: Secret-like inputs are refused
r[lattice.workflow_templates.run_inputs.secrets] Run inputs MUST NOT carry secrets; credential references remain the only secret path. For `relative_path` parameters only, a name or binding key suffix with a secret word or a qualified `key` as an `_`-separated word, a supplied value component (or its stem before `.`, for formats without `.`) in a known provider token format (its prefix and at least its minimum suffix from its own alphabet, with the format's required digits or classes), and a supplied value with a `.`-separated piece that, without `-` and `_`, has at least 20 letters and digits of mixed classes whose lowercase runs average under 3 bytes MUST be refused naming only the parameter. `bool`, `integer` and `enum` parameters MUST NOT be screened. Ordinary refs, repository paths and CamelCase identifier file names whose words are three or more letters SHOULD be accepted; names built from short pieces joined by `-` or `_`, or from one- and two-letter words with digits, MAY be refused, and a token joined behind long lowercase words MAY be accepted. The rule is a pattern check, not a proof.

#### Scenario: Token-shaped values are refused and ordinary refs admit
r[lattice.workflow_templates.run_inputs.secrets.scenario.screened]
- GIVEN a `relative_path` parameter
- WHEN a supplied value matches a listed provider token format or the token-run rule
- THEN the input MUST be refused naming only the parameter
- AND ordinary branch refs and repository paths whose words are three or more letters MUST be admitted
