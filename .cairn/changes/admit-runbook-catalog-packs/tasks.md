# Tasks

## Phase 1: Manifest, identity and policy

- [x] [serial] 1.1 Add the `workflow_bundle` content role and `workflow_bundles` declarations, validate placement, bundle manifests and ids in the pure `catalog` core, allow catalog-only packs, and join the declarations to the manifest identity only when present (`src/operation_pack/{model,identity}.rs`, `validation/mod.rs`, `catalog/mod.rs`, `policy/operation-packs.ncl`). Tests: `tests/test_runbook_catalog_core.rs::seed_manifest_satisfies_the_contract_and_identity_covers_every_bundle_byte`, `::pack_manifests_reject_traversal_duplicates_misplacement_and_symlinks`. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog.scenario.manifest]

## Phase 2: Fetch and installation

- [x] [serial] 2.1 Fetch one full commit through bounded `git` with a cleared environment, parse the tree listing in the pure `catalog::source` core, refuse symlink and submodule members, verify with the strict pack loader and publish create-only (`catalog/source.rs`, `shell/catalog/fetch.rs`, `cli/catalog.rs`). Tests: `tests/test_runbook_catalog_cli.rs::fetch_lock_install_and_list_from_an_exact_commit_without_network_on_load`, `::fetch_refuses_inexact_sources_wrong_identity_and_symlink_members`, `tests/test_runbook_catalog_core.rs::tree_listings_admit_only_regular_files_inside_the_selected_pack`. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog.scenario.install] r[lattice.operation_packs.runbook_catalog.scenario.fetch_denied]
- [x] [serial] 2.2 Refuse the whole pack on bundle byte drift while naming every bundle, decide each admitted bundle independently through `workflow_template::load_catalog`, and stage, re-verify and rename each admitted bundle create-only (`catalog/decision.rs`, `shell/catalog.rs`, `shell/catalog/placement.rs`). Tests: `tests/test_runbook_catalog_cli.rs::tampered_bytes_refuse_the_pack_and_name_each_drifted_bundle`, `tests/test_runbook_catalog_core.rs::one_bad_bundle_is_rejected_with_its_diagnostic_while_the_other_installs`. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog.scenario.drift] r[lattice.operation_packs.runbook_catalog.scenario.partial]
- [x] [serial] 2.3 Keep installation free of effect authority. Tests: `tests/test_runbook_catalog_core.rs::installation_grants_no_effect_authority`. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog.scenario.authority]

## Phase 3: Documentation and verification

- [x] [serial] 3.1 Retain the seed catalog fixture and describe the catalog layout, installation contract, authority boundary and publishing in `docs/operation-packs.md`, `docs/workflow-template-bundles.md` and the changelog. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog]
- [x] [serial] 3.2 Run the focused tests, clippy, fmt, the Nickel policy check, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks, and record the results. Evidence: `evidence/verification.json`. r[lattice.operation_packs.runbook_catalog]

Only checked tasks are complete.
