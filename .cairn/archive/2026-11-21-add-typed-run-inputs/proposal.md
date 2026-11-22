# Typed `--input` values on `lattice run`

Tracking: [Lattice #28](https://github.com/OnixResearch/lattice/issues/28). Owner-authorized round 2026-11-21. Builds on Lattice #27 (`add-typed-workflow-template-params`).

## Why

`lattice run` takes only a saved workflow name. A manual or dispatcher-initiated run, such as a Kiln runbook dispatch, cannot supply typed per-run values such as a target path or format. Lattice #27 added typed bundle parameters, the pure `params::resolve` and `workflow_template::bind`, but no route from the CLI to them.

## What Changes

- `lattice run NAME` and `lattice authority inspect NAME` accept repeatable `--input NAME=VALUE` and one `--input-file PATH` JSON object. A name repeated within or across both sources is refused. r[lattice.workflow_templates.run_inputs] r[lattice.workflow_templates.run_inputs.scenario.typed]
- Saving a bound bundle retains its template binding by identity, keyed by the saved workflow name. Inputs rebind through that exact template, loading only its bundle directory, and are refused without a retained binding (save again from the bundle), for a template without parameters, after saved-workflow or template drift, or for a workflow with an approval gate. r[lattice.workflow_templates.run_inputs.retained]
- Wrong, unknown, missing, duplicate and malformed inputs fail before a run id is reserved, any record is written, or any node runs, without echoing a value. r[lattice.workflow_templates.run_inputs.scenario.refused]
- Secret-like `relative_path` inputs are refused by a pattern rule over names, binding keys and token-shaped values. It is a heuristic, not a proof that accepted values are not sensitive. Credential references remain the only secret path. r[lattice.workflow_templates.run_inputs.secrets]
- A run with inputs reserves its run id and records its input identity in one transaction. `runs show` displays it without values. r[lattice.workflow_templates.run_inputs.record] r[lattice.workflow_templates.run_inputs.inspection]
- `authority inspect` with the same inputs reports the grants of the bound revision. r[lattice.workflow_templates.run_inputs.authority]
- Runs without inputs keep their path, identity, records and output bytes. r[lattice.workflow_templates.run_inputs.scenario.inputless]

## Impact

- **Files**: `src/workflow_template/{inputs/mod.rs,inputs/admission.rs,inputs/parsing.rs,inputs/secrets.rs,inputs/secrets/tests.rs,mod.rs,shell.rs,shell/inputs.rs,shell/activation.rs}`; `src/store/implementation/catalog/{inputs.rs,mod.rs}`; `src/headless/shell.rs`, `src/headless/shell/bound.rs`, `src/headless/shell/bound/tests.rs`; `src/run/shell.rs`, `src/run/inspection/{identity.rs,core.rs,format.rs,mod.rs,model.rs}`; `src/main.rs`; `tests/test_run_inputs.rs`, `tests/test_run_inputs_records.rs`, `tests/test_run_inputs_sources.rs`, `tests/support/run_inputs.rs`; `docs/workflow-template-bundles.md`, `docs/choregraph-effect-authority.md`, `README.md`; `CHANGELOG.md`.
- **Compatibility**: two store tables, `workflow_template_bindings_v1` and `run_inputs_v1`, are created on first write. `runs show` gains an `inputs` object and the `inputs_identity_mismatch` diagnostic only for runs with inputs; its schema version is unchanged. `save_draft` now also writes the retained binding. Workflows saved before this change have no binding and must be saved again from their bundle to accept inputs.
- **Testing**: the focused tests above, `cargo test -p lattice --lib -- run workflow_template`, `cargo clippy -p lattice --all-targets --no-deps -- -D warnings`, `cargo fmt -p lattice --check`, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks.

## Dependencies

- `add-typed-workflow-template-params` (Lattice #27).

## Out of Scope

- Inputs for workflows not saved from a parameterized bundle, inputs on triggers or schedules, and a TUI form for values.
- Inputs for workflows with approval gates: resume reloads the saved revision.
- Per-value digests in `runs show`, and storing raw input values.
