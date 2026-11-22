# Design: Typed parameters and a declared mode for workflow-template bundles

## Context

Baseline: Lattice `0862c017`.

- `BundleManifest` (`schema_version = 1`) had no parameter or mode field and denied unknown fields. `validate_bundle` checked files, identities, workflow, compatibility, orchestration binding and guarded authority, and returned a `ValidatedBundle` that `save_draft` and activation used directly.
- Shell nodes run `sh -c command` with an optional `cwd` and an `env` map. There is no argv form.
- Nickel import renumbers node identities, so an authored `node_id` does not survive loading.
- `authority inspect` derives one grant per effectful node from its kind: process, network, agent, or file read, write or append.

## Decisions

### Decision: Environment values are the only binding target

**Choice:** A parameter binds to `{ node = <label>, shell_env = "LATTICE_PARAM_*" }` targets. The label must match exactly one node, and that node must be a shell node. Binding inserts the rendered value into that node's `env`. At load, every expansion of the key in the command must be exactly `"${KEY}"`. A bare `$KEY`, an unquoted `${KEY}`, or an operator form such as `${KEY:-x}` fails.

**Rationale:** Shell nodes have no argv form, and adding one would change the runtime and its authority payload. An environment value is a structured input that the shell expands as one argument when it is double-quoted. Values never enter command text, so command bytes, node kinds and effect grants stay as reviewed. The prefix stops a parameter from setting `PATH`, `LD_PRELOAD` or another ambient variable. Labels are used because node identities do not survive import.

### Decision: A closed, metacharacter-free value alphabet

**Choice:** The types are `bool`, `integer {min, max}`, `enum {values}` and `relative_path {max_bytes}`. Enum tokens and relative paths use ASCII letters, digits, `.`, `_`, `-` and, for paths, `/`. Path components are nonempty, never `..`, and never start with `-`. A required parameter has no default, and an optional one has an in-bounds default.

**Rationale:** This is a second barrier behind the quoting check. A value cannot contain whitespace, quotes, glob characters, `$` or command separators. A relative path cannot escape upward or look like an option. Requiring defaults on optional parameters keeps every resolved set total.

### Decision: Defaults reproduce the template

**Choice:** The template workflow must hold each optional parameter's rendered default under its key and must not hold a required parameter's key.

**Rationale:** Binding the defaults then yields byte-identical workflow content and revision. The previewed, BLAKE3-pinned workflow is the one that runs when no value is supplied.

### Decision: One pure resolver and a bound bundle

**Choice:** `params::resolve(declared, supplied)` rechecks the declaration shapes and returns `params::Resolved`. It holds the values sorted by name and a domain-separated BLAKE3 identity over each name, value kind and rendered value. Each field is NUL-terminated, which is unambiguous because checked names and values never contain NUL. `bind(template, supplied)` resolves the values, applies them, and rebinds any orchestration's `workflow` to the new revision. It re-runs `admit_content`, the same workflow, compatibility, mode, orchestration and guarded checks the template passed. It returns a `ValidatedBundle` whose `identity` is BLAKE3 over the template identity and the resolved identity, and whose `params` (`params::Bound`) records both. `save_draft` refuses `params: None` with `ParamsUnbound`. `plan_application` and `plan_activation` return `ParamsUnbound` or `ModeViolation` before any store or dispatch step. The activation receipt's bundle identity is therefore the bound identity.

**Rationale:** Lattice #28 can call `resolve` directly. A template cannot be saved or started with values nobody chose. The receipt commits to the values without a schema change.

### Decision: Conservative mode classification

**Choice:** `classification::classify_node` matches every `NodeConfig` variant exhaustively. Condition, merge and approval nodes are pure. File reads and image inputs are read-only. Shell, operation, HTTP, agent, subworkflow and file write or append nodes are mutating. A `read_only` manifest whose workflow needs `mutating` fails with `mode_violation` at load and in save and start planning.

**Rationale:** No shell command is provably read-only, HTTP `GET` can change remote state, and a subworkflow's child is outside the bundle. The exhaustive match makes any new node kind take an explicit classification. As a result every built-in bundle is `mutating`.

### Decision: Schema 1 is a documented compatibility path

**Choice:** `CURRENT_BUNDLE_SCHEMA_VERSION` is 2, and `mode` and `params` are required there. A schema 1 manifest with neither field loads as `mutating` with no parameters. A schema 1 manifest with either field fails with `unsupported_schema`, and a schema 2 manifest without them fails with `invalid_manifest`.

**Rationale:** `mutating` with no parameters is exactly what a schema 1 manifest already allowed, so no authority is invented. Unchanged bytes keep identities and catalogs that an earlier `--init` copied. A hard version error would make operators re-review bundles whose meaning did not change.

### Decision: Built-in migration

**Choice:** All four built-ins declare schema 2 and `mutating`. The catalog review's command becomes `--root "${LATTICE_PARAM_CATALOG_ROOT}"`, with a `catalog_root` relative-path parameter that defaults to `builtins/agent-evaluations`, and its workflow pin is regenerated. The guarded bundles and the shell preview declare no parameters.

**Rationale:** The guarded authority check pins the root, commands, empty environment and agent workspace. A root parameter there would change where an agent may edit and would need a separate authority review.

## Tests

- `tests/test_workflow_template_params.rs` covers defaults, a stable identity that covers every value and not how it was supplied, unknown, wrong-type, out-of-bounds and missing values, and 16 hostile paths, none of them echoed. It also covers rejected declaration shapes and closed serde shapes.
- `tests/test_workflow_template_binding.rs` covers built-in migration, defaults that reproduce each workflow and orchestration revision, and binding that changes only the environment. It checks identity and revision drift with the value, equal `authority inspect` grants, safe bundle diagnostics, and a real `sh -c` probe that receives each value as one unchanged argument.
- `tests/test_workflow_template_modes.rs` covers classification, read-only admission, schema 1 and mixed-schema handling, and save and start refusals for unbound or mode-violating bundles. It also edits manifests and workflows at load: read-only drift, default drift, a missing label, identity drift, and an unquoted reference that still fails after its workflow is re-pinned.
- `src/workflow_template/params/binding.rs` unit-tests the quoting check.
- The existing template, TUI and activation tests now bind before save and start.

## Risks / Trade-offs

- The quoting check is lexical. It cannot see `eval` or a program that re-parses its argument. The value alphabet keeps such re-parsing inert, and command bytes remain operator-reviewed.
- `relative_path` is lexical. It does not resolve symbolic links and does not grant a filesystem root. The command that receives it keeps its own boundary.
- Every shell bundle is `mutating`, so `read_only` is available only to file-read and pure workflows.
- The TUI cannot enter values. A bundle with a required parameter is refused in TUI preview, and integrations must call `bind`.
