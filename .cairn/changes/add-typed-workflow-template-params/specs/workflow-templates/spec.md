# Workflow Templates Specification

## ADDED Requirements

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
- THEN the bound workflow MUST differ from the template only in the declared environment values, and each shell command MUST receive a value as one unchanged argument
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
