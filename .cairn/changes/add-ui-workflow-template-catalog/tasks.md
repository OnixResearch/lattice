## Phase 1: Baseline and typed core

- [x] [serial] Establish and record the smallest relevant pre-change baselines for node-template loading/TUI behavior, workflow import/save, orchestration admission, initialization, and repository validation. r[lattice.workflow_templates.compatibility]
- [x] [serial] Add typed Nickel manifest contracts plus Rust bundle, identity, compatibility, diagnostic, explicit-fact, and application-plan values with named count/byte/schema limits. r[lattice.workflow_templates.catalog]
- [x] [serial] Implement the pure bundle validator and conflict/application planner over in-memory manifest, file, workflow, orchestration, and store-fact inputs. r[lattice.workflow_templates.core]
- [x] [parallel] Add positive and negative pure-core tests for exact valid bundles, deterministic planning, malformed/duplicate/unsafe paths, missing/extra files, BLAKE3 drift, invalid workflow/orchestration content, revision mismatch, exact existing content, and incompatible name conflicts. r[lattice.workflow_templates.catalog.scenario.valid] r[lattice.workflow_templates.catalog.scenario.invalid] r[lattice.workflow_templates.core.scenario.plan] r[lattice.workflow_templates.core.scenario.conflict]

## Phase 2: Bounded loading and initialization

- [x] [serial] Add the thin capability-rooted loader with deterministic enumeration, regular-file/no-symlink admission, bounded reads, isolated safe diagnostics, and no network or execution path. r[lattice.workflow_templates.loading]
- [x] [parallel] Add loader tests for valid local files and negative traversal, absolute path, symlink, non-regular file, oversized file/bundle/catalog, undeclared file, and diagnostic-redaction cases. r[lattice.workflow_templates.loading.scenario.local] r[lattice.workflow_templates.loading.scenario.rejected]
- [x] [serial] Add reviewed built-in bundle source assets and extend fresh initialization with exclusive create/no-clobber behavior and explicit partial-result handling. r[lattice.workflow_templates.init]
- [x] [parallel] Add positive fresh-init and negative existing-path/race/partial-failure tests that prove pre-existing config, node-template, workflow-template, role, and asset bytes remain unchanged. r[lattice.workflow_templates.init.scenario.fresh] r[lattice.workflow_templates.init.scenario.existing]

## Phase 3: TUI preview and activation

- [x] [serial] Add a distinct workflow-template browser and preview state that displays safe bundle metadata and loads only an unsaved in-memory workflow/orchestration draft after normal dirty-canvas confirmation. r[lattice.workflow_templates.preview]
- [x] [parallel] Add positive and negative TUI tests for browse/select/preview, dirty-canvas refusal, cancellation, malformed-catalog diagnostics, and zero store/run/agent/credential/effect mutation before explicit action. r[lattice.workflow_templates.preview.scenario.select] r[lattice.workflow_templates.preview.scenario.cancel]
- [x] [serial] Wire explicit save/start actions through existing workflow and orchestration command ports with exact revision/policy/authority revalidation and stable no-auto-start behavior. r[lattice.workflow_templates.activation]
- [x] [parallel] Add activation tests for exact admitted start, exact-content idempotency, name conflict, workflow/orchestration drift, missing runtime/effect/credential/trigger/budget/authority facts, selection-only behavior, and no partial instance/run/budget/effect creation. r[lattice.workflow_templates.activation.scenario.start] r[lattice.workflow_templates.activation.scenario.blocked] r[lattice.workflow_templates.activation.scenario.no-auto-start]

## Phase 4: Compatibility, evidence, and completion

- [x] [serial] Preserve and directly test existing shell node-template parameter rendering/TUI creation plus current workflow/orchestration CLI routes when the new catalog is unused. r[lattice.workflow_templates.compatibility.scenario.nodes] r[lattice.workflow_templates.compatibility.scenario.cli]
- [x] [parallel] Update operator/example documentation with load/preview/save/start steps, authority boundaries, content-identity regeneration, and explicit non-claims. r[lattice.workflow_templates.catalog] r[lattice.workflow_templates.activation]
- [x] [serial] Run focused tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validate and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`; review any Octet identity transition with a fail-closed semantic migration before sync/archive. r[lattice.workflow_templates.catalog] r[lattice.workflow_templates.compatibility]
