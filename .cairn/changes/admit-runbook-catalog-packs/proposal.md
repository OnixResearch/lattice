# Admit git-backed runbook catalogs through content-addressed operation packs

Tracking: [Lattice #29](https://github.com/OnixResearch/lattice/issues/29). Owner-approved round 2026-11-21. Related: Lattice #27 (bundle `params` and `mode`), Lattice #28 (`--input`), Kiln #101 (seed runbooks).

## Why

Workflow-template bundles load only from the local `workflow-templates/` directory next to `config.ncl`. Operation packs distribute operation catalogs, runtimes and skills, but not bundles. There is no reviewed, versioned way to publish a set of reusable runbooks and have a runner admit exact revisions.

## What Changes

- A pack manifest can declare workflow-template bundles: every bundle file is a `workflow_bundle` content item under `workflow-templates/<bundle-id>/`, and `workflow_bundles` pins each bundle's loader identity. A catalog-only pack may omit operations and runtimes. The pack identity covers every bundle byte; packs without bundles keep their identity. `policy/operation-packs.ncl` gains the typed `workflow_bundles` field. r[lattice.operation_packs.runbook_catalog] r[lattice.operation_packs.runbook_catalog.scenario.manifest]
- `lattice pack catalog-fetch --confirm-fetch` retrieves one full commit from a `file://`, `https://` or `ssh://` repository through bounded `git` with a cleared environment. It refuses symlink and submodule members and an unexpected `--pack-identity`, and publishes a strictly verified tree create-only. It is the only networked catalog step. r[lattice.operation_packs.runbook_catalog.scenario.install] r[lattice.operation_packs.runbook_catalog.scenario.fetch_denied]
- `lattice pack catalog-install` admits the locked pack and installs its bundles into a local `workflow-templates/` catalog. Pack byte drift refuses the whole pack and still names every drifted bundle. After admission each bundle is decided independently through the existing workflow-template loader; the good ones are staged, re-verified and renamed in create-only. `lattice pack catalog-list` lists a catalog. r[lattice.operation_packs.runbook_catalog.scenario.drift] r[lattice.operation_packs.runbook_catalog.scenario.partial]
- Installation grants no effect authority; the existing standalone authority profile still gates the first run. r[lattice.operation_packs.runbook_catalog.scenario.authority]
- A seed catalog fixture is retained in `tests/fixtures/runbook-catalog`, and `docs/operation-packs.md` describes the layout, contract and publishing.

## Impact

- **Files**: `src/operation_pack/{model,identity,mod}.rs`, `src/operation_pack/validation/mod.rs`, new `src/operation_pack/catalog/{mod,decision,source}.rs`, `src/operation_pack/shell/{mod,installation}.rs`, new `src/operation_pack/shell/catalog.rs` and `shell/catalog/{placement,fetch}.rs`, `src/operation_pack/cli/mod.rs`, new `src/operation_pack/cli/catalog.rs`, `src/inet/route/pack.rs`, `examples/generate_operation_pack_fixture.rs`, `tests/test_operation_pack_wasm_core.rs` (the new `workflow_bundles` field in a `Manifest` literal), `policy/operation-packs.ncl`, `docs/operation-packs.md`, `docs/workflow-template-bundles.md`, `CHANGELOG.md`, `tests/fixtures/runbook-catalog/`, `tests/runbook_catalog/mod.rs`, `tests/test_runbook_catalog_{core,cli}.rs`.
- **Compatibility**: Manifests without `workflow_bundles` decode, validate and identify exactly as before. The workflow-template bundle schema, loader and built-in bundles are unchanged.
- **Testing**: `cargo test -p lattice --test test_runbook_catalog_core --test test_runbook_catalog_cli`, the operation-pack and workflow-template suites, clippy with `-D warnings`, `cargo fmt -p lattice --check`, `nickel typecheck policy/*.ncl`, `octet-precommit-check`, `cairn validate` and this change's gates.

## Out of Scope

- Automatic fetching, updating, registry resolution or credentials for private repositories.
- A multi-bundle transaction across one installation.
- Removing or upgrading installed bundles; installation never replaces an existing bundle directory.
- Kiln dispatch of runbooks.
