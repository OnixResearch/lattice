let Contracts = import "traceability-inventory-contracts.ncl" in

let workflow_validation = { path = "tests/test_workflow_validation.rs", blake3 = "2fe0adea9d5b36d2d09d57fd511ecfe1479f16667ff3fe4a5fd964d0496e4add" } in
let n8n_json = { path = "tests/test_n8n_json.rs", blake3 = "fb156b96a063f69f111be98be294e4bd6557f9c971540dbae2b45d2daa3ba776" } in
let replay = { path = "tests/test_replay.rs", blake3 = "e964d72cb04cf1664274c5ae6a1ccc3e2812987a48bdba29987aa7b1463bd6f7" } in
let credentials = { path = "tests/test_credentials.rs", blake3 = "cc2675977f742a1dfda395e00379b22e41a5f50c3858eb8b1e0153ddc5f17835" } in
let control_flow = { path = "tests/test_control_flow.rs", blake3 = "608b027f0450f7956b728a1e9779479ce4948182cca155b3d721866df2497e75" } in
let node_kind_provider = { path = "tests/test_node_kind_provider.rs", blake3 = "8b0f312b594587ddc129760ed053cf79db1667fcdd50c7c22f3743a840bab975" } in
let runner_core = { path = "tests/test_runner_core.rs", blake3 = "09cfa296c3c660c521a7b635e4274b3322c7bc916cc232c7252b161b39948a36" } in
let completion = { path = "tests/test_workflow_validation_completion.rs", blake3 = "6e265bdeb0ee4cd425c0d357f355496ac8d36f47ae577c9fc4f692d826976f8d" } in
let runner = { path = "src/engine/runner.rs", blake3 = "65d1a6f3d65c4d13d6b0558e10ba144bef7fab64e8b05b10a3808a85980990c5" } in
let validation = { path = "src/workflow_validation.rs", blake3 = "f4e8bb19c0b122ec89c6c733e2f6f304836eaa93039e1a726649dcedde3e613d" } in
let exact = "exact_maintained_evidence" in
let row = fun id decision rationale refs => {
  requirement_id = id,
  classification = decision,
  rationale_code = rationale,
  evidence = refs,
} in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 290,
  expected_missing_hash = "ba797fa2f3f60697a1941f1b61927af134df835f8f6454bb1febda85ebcfcc7d",
  reviewed = [
    row "n8nclone.workflow_validation.credentials.providers.config" exact "assertions.missing-provider-fatal-pre-effect" [completion],
    row "n8nclone.workflow_validation.credentials.providers.field" exact "assertions.unsupported-field-fatal-pre-effect" [completion],
    row "n8nclone.workflow_validation.credentials.providers.key" exact "assertions.malformed-key-fatal-safe-context" [completion],
    row "n8nclone.workflow_validation.diagnostics.core.empty_command" exact "assertions.empty-command-fatal-code" [workflow_validation],
    row "n8nclone.workflow_validation.diagnostics.core.missing_config" exact "assertions.missing-config-fatal-code" [workflow_validation],
    row "n8nclone.workflow_validation.diagnostics.core.valid" exact "assertions.valid-workflow-no-fatal" [workflow_validation],
    row "n8nclone.workflow_validation.diagnostics.negative.invalid_edge" exact "assertions.invalid-edge-stable-code" [workflow_validation],
    row "n8nclone.workflow_validation.generic_runner_admission.effects" exact "assertions.unsupported-effect-policy-fatal" [runner, validation, completion],
    row "n8nclone.workflow_validation.generic_runner_admission.graph" exact "assertions.all-bounded-graph-classes-pre-effect" [completion],
    row "n8nclone.workflow_validation.generic_runner_admission.terminal_state" exact "assertions.terminal-transition-preserves-workflow" [completion],
    row "n8nclone.workflow_validation.n8n_json.export_limits.node" exact "assertions.unsupported-export-refused" [n8n_json],
    row "n8nclone.workflow_validation.n8n_json.unsupported.connection" exact "assertions.malformed-connection-refused" [n8n_json],
    row "n8nclone.workflow_validation.n8n_json.unsupported.expression" exact "assertions.unsupported-expression-refused" [n8n_json],
    row "n8nclone.workflow_validation.n8n_json.unsupported.node_kind" exact "assertions.unsupported-node-refused" [n8n_json],
    row "n8nclone.workflow_validation.n8n_parity_diagnostics.malformed_connection" exact "assertions.parity-malformed-golden-diagnostic" [n8n_json],
    row "n8nclone.workflow_validation.n8n_parity_diagnostics.plaintext_credential" exact "assertions.parity-plaintext-safe-diagnostic" [n8n_json],
    row "n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_expression" exact "assertions.parity-expression-golden-diagnostic" [n8n_json],
    row "n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_node" exact "assertions.parity-node-golden-diagnostic" [n8n_json],
    row "n8nclone.workflow_validation.node_kind_provider" exact "assertions.all-builtins-provider-and-runtime-admission" [completion],
    row "n8nclone.workflow_validation.replay_records.artifact" exact "assertions.missing-artifact-recoverable-placeholder" [replay],
    row "n8nclone.workflow_validation.replay_records.order" exact "assertions.duplicate-out-of-order-fatal-codes" [replay],
    row "n8nclone.workflow_validation.replay_records.terminal" exact "assertions.terminal-replay-mutation-rejected" [replay],
    row "n8nclone.workflow_validation.run_gate.block" exact "assertions.fatal-blocks-tui-and-headless-effects" [workflow_validation],
    row "n8nclone.workflow_validation.run_gate.warning" exact "assertions.warning-allows-run-gate" [workflow_validation],
    row "n8nclone.workflow_validation.subworkflows.mapping" exact "assertions.unknown-mapping-fatal-preserves-config" [completion],
    row "n8nclone.workflow_validation.subworkflows.missing" exact "assertions.missing-subworkflow-rejected" [control_flow],
    row "n8nclone.workflow_validation.subworkflows.recursive" exact "assertions.recursive-subworkflow-rejected" [control_flow],
    row "n8nclone.workflow_validation.tui.panel.empty" exact "assertions.ready-message-rendered" [completion],
    row "n8nclone.workflow_validation.tui.panel.open" exact "assertions.severity-location-action-rendered" [completion],
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "partial evidence is not a near-complete correctness claim",
    "no exact evidence identified is bounded to the reviewed maintained roots and is not proof of global absence",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
