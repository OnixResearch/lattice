# Typed parameters and a declared mode for workflow-template bundles

Tracking: [Lattice #27](https://github.com/OnixResearch/lattice/issues/27). Owner-authorized round 2026-11-21.

## Why

Workflow-template bundles bake fixed values such as `--root` into their commands. A manifest cannot declare parameters or say whether the bundle may mutate a repository. Such a bundle cannot be reused across repositories or reviewed as a parameterized unit. A dispatcher such as Kiln also cannot tell a read-only audit from a mutating change. A later typed `lattice run --input` (Lattice #28) needs one pure validation function to reuse.

## What Changes

- Schema 2 manifests add a required `mode`, `read_only` or `mutating`, and a required closed `params` list. The manifest identity is the BLAKE3 identity of its bytes, so it covers both. r[lattice.workflow_templates.params] r[lattice.workflow_templates.mode]
- Each parameter has a lowercase name and a type from `bool`, `integer`, `enum` and `relative_path` with its bounds. It is either required without a default or optional with an in-bounds default. It binds to 1 to 8 shell nodes, named by label, as `LATTICE_PARAM_*` environment values. No type accepts free-form text. r[lattice.workflow_templates.params.scenario.declaration]
- The pure `workflow_template::params::resolve(declared, supplied)` rejects unknown names, wrong types, out-of-bounds values and missing required values. It never echoes a value, and it returns the values with a BLAKE3 identity. r[lattice.workflow_templates.params.scenario.bounds]
- `workflow_template::bind(template, supplied)` writes values only into declared environment entries. Each bound command must read its variable only as `"${KEY}"`. The bound workflow and any orchestration, rebound to its revision, pass full admission again. The bound identity covers the template and the values. Save and start require a bound bundle, and the TUI preview binds the defaults. Grants from `authority inspect` are unchanged. r[lattice.workflow_templates.params.scenario.binding]
- Effects are classified conservatively. Only file reads and image inputs are read-only; condition, merge and approval nodes are pure, and everything else is mutating. A `read_only` bundle with a mutating node fails at load, save and start planning. The TUI marks every bundle's mode. r[lattice.workflow_templates.mode.scenario.classification] r[lattice.workflow_templates.mode.scenario.enforced]
- The built-in bundles move to schema 2 as `mutating`. The catalog review's `--root` becomes the typed `catalog_root` parameter. The guarded bundles keep their pinned `--root .`. Schema 1 manifests load as `mutating` without parameters. r[lattice.workflow_templates.params.scenario.migration] r[lattice.workflow_templates.mode.scenario.compatibility]

## Impact

- **Files**: `src/workflow_template/{classification.rs,params.rs,params/resolution.rs,params/binding.rs,core.rs,core/binding.rs,core/planning.rs,model.rs,model/activation.rs,mod.rs,shell.rs,shell/activation.rs}`; `src/tui/app/lifecycle/{catalog,persistence}.rs`, `src/tui/app/model/dialogs.rs`, `src/tui/panels/overlays/browsers.rs`; `policy/workflow-template-bundle.ncl`; the four `builtins/workflow-templates/*/manifest.ncl` files and the catalog review's `workflow.ncl`; `tests/test_workflow_template_{params,binding,modes}.rs`, `tests/fixtures/workflow-templates/`, `tests/test_workflow_templates.rs`; `docs/workflow-template-bundles.md`, `docs/guarded-agent-templates.md`, `docs/agent-evaluation.md`; `CHANGELOG.md`.
- **Compatibility**: `ValidatedBundle` gains `params`, and `BundleManifest` gains optional `mode` and `params`. `save_draft` and activation now require a bundle returned by `bind`. Schema 1 manifest bytes and identities are unchanged. The catalog review workflow's BLAKE3 pin changes.
- **Testing**: the focused tests above, plus `test_guarded_agent_templates` and the lib unit tests, `cargo clippy -p lattice --all-targets --no-deps -- -D warnings`, `cargo fmt -p lattice --check`, the runtime Nickel typecheck, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks.

## Dependencies

- None.

## Out of Scope

- A CLI or TUI form for entering parameter values. The TUI binds defaults, and integrations call `bind`.
- `lattice run --input` (Lattice #28) and catalog search (Lattice #29).
- Parameters on the guarded bundles, other binding targets such as `cwd` or file paths, and read-only classification of any shell command.
