# Node-template prompt evidence promotion

## Result

Exactly three reviewed node-template prompt requirements were promoted through assertion-local markers:

- `n8nclone.node_templates.parameters.prompt` on `add_node_flow_prompts_and_renders_template_parameters`
- `n8nclone.node_templates.parameters.prompt.cancel` on `cancel_template_parameter_entry_preserves_workflow`
- `n8nclone.node_templates.parameters.prompt.required` on `required_template_parameter_prompt_is_visible_before_node_creation`

The rows were removed from reviewed debt policy. Historical audit and completion fixtures remain pinned; their byte checks remove only the three expected marker-only lines before hashing.

## Exact transition

- baseline requirements/referenced/missing: `765 / 438 / 327`
- current requirements/referenced/missing: `769 / 445 / 324`
- removed missing IDs: exactly the three allowlisted prompt IDs
- added missing IDs: none
- dangling markers: 0
- `valid: false`
- missing-set BLAKE3: `f33a71ef4aa5bb871a48a6d8214500238153898df53779d7706dc4a56433a7ee`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `98d7096fa39a4c421b8e7411aa7d4dee9cc6f877599a0395f5af7286a8bc6f73`
- receipt BLAKE3: `162f9430750682f7945877a22bf3814b0dc9408b8576a1d0b745ee51f0a165ca`

## Inventory

- total: 324
- reviewed: 2
- exact: 1
- no exact identified: 1
- unassessed: 322
- review-policy BLAKE3: `70b2f7fdf29e23130a6c2613ec0f5396b581f05abeb024be33c72b2e836dd741`
- inventory BLAKE3: `8b27f3cbf0d24448a1e78b090cad6da47a4a6d32b3a245a6f64e476d2c39030e`

## Octet identity migration

Adding marker-only lines shifted one pre-existing Octet finding identity without changing its semantic finding multiset. The reviewed migration accepted exactly:

- stale: `b3:744d0ad32c7cdc75`
- replacement: `b3:84a3de636a5e0179`

The migration self-test, dry run, and execute path passed with all 912 findings semantically matched. The new baseline is `b3:4c4af2654b0a9650fcdd0da7a70b5a9a46c12da60970a9fb92424f1e43bb223c`; accepted/current is `912 / 912` with zero new or stale findings. The underlying Cargo subprocess remains classified as exit code 101 while the wrapper accepts the reviewed zero-delta result.

## Validation

Positive and negative rendering, historical audit, completion, promotion, inventory, identity, and template-parameter tests pass. `scripts/check.rs`, Cairn validation and gates, deterministic inventory checks, Octet zero-delta validation, and `nix flake check -L --option secret-key-files ''` pass.

## Non-claims

Promotion establishes only exact Tracey marker coverage for the accepted prompt clauses. It does not prove arbitrary TUI behavior, complete traceability, release readiness, or production readiness.
