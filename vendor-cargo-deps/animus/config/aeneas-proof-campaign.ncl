# Animus-owned typed Aeneas proof-campaign recipe profile.
# r[impl agent_harness_runtime.aeneas_proof_campaign.profile]
# r[impl agent_harness_runtime.aeneas_proof_campaign.skill_source]
let contracts = import "../policy/contracts/primitives.ncl" in

let campaign_schema = "animus.aeneas-proof-campaign.v1" in
let bundle_schema = "animus.aeneas-proof-campaign-bundle.v1" in
let manifest_schema = "animus.safe-skill-manifest.v1" in
let catalog_schema = "animus.proof-campaign-catalog.v1" in
let campaign_identifier = "aeneas-proof-campaign" in
let skill_identifier = "aeneas-proof-campaign" in
let skill_source_path = "skills/aeneas-proof-campaign/SKILL.md" in
let skill_source_blake3 = "f8f4d41384958e468c952289e8a548d78b7f6e69616abedd0ed629dfec327492" in
let maximum_label_chars = 128 in
let maximum_non_claims = 32 in
let maximum_stages = 16 in
let maximum_roles = 8 in
let maximum_services = 8 in
let proof_time_ms_limit = 1800000 in
let stage_call_limit = 64 in
let tool_call_limit = 128 in
let output_byte_limit = 1048576 in
let retrieval_item_limit = 128 in
let log_byte_limit = 262144 in
let retained_artifact_limit = 64 in

let LowerLabel = contracts.stable_validator "AENEAS_CAMPAIGN_LABEL" (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= maximum_label_chars
  && std.string.is_match "^[a-z0-9][a-z0-9._:/-]*[a-z0-9]$|^[a-z0-9]$" value
) in
let LowerHash = contracts.lowercase_blake3 "AENEAS_CAMPAIGN_BLAKE3" in
let Positive = contracts.stable_validator "AENEAS_CAMPAIGN_POSITIVE_BUDGET" (fun value =>
  std.is_number value && value > 0
) in
let SafeText = contracts.stable_validator "AENEAS_CAMPAIGN_SAFE_TEXT" (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "RAW_PROMPT|HIDDEN_REASONING|BEGIN PRIVATE KEY|API_KEY=|/home/|provider_payload|tool_output" value
) in
let RoleClass = contracts.enum "AENEAS_CAMPAIGN_ROLE_CLASS" ["implementation", "review", "verification"] in
let Mutation = contracts.enum "AENEAS_CAMPAIGN_MUTATION" ["statement", "maintained_theorem", "none"] in
let ArtifactRole = contracts.enum "AENEAS_CAMPAIGN_ARTIFACT_ROLE" [
  "source",
  "statement_candidate",
  "generated_model",
  "generated_manifest",
  "maintained_theorem",
  "statement_review",
  "proof_receipt",
  "verification_receipt",
  "bounded_log",
] in
let Outcome = contracts.enum "AENEAS_CAMPAIGN_OUTCOME" [
  "proved",
  "blocked",
  "counterexample",
  "invalid_statement",
  "invalid_model",
  "budget_exhausted",
  "tool_failure",
  "denied",
  "cancelled",
  "abstained",
] in
let Role = {
  assignment | LowerLabel,
  semantic_role_ref | contracts.enum "AENEAS_CAMPAIGN_ROLE_REF" ["implementer", "reviewer", "verifier"],
  class | RoleClass,
  mutation | Mutation,
  model_profile_ref | contracts.exact "AENEAS_CAMPAIGN_ROLE_MODEL_REF" "compat-default",
  capability_ceiling | Array LowerLabel,
  input_contract | LowerLabel,
  output_contract | LowerLabel,
  service_ceiling | Array LowerLabel,
  evidence_ceiling | Array ArtifactRole,
  separation_policy_ref | LowerLabel,
} in
let Stage = {
  stage_id | LowerLabel,
  assignment | LowerLabel,
  depends_on | Array LowerLabel,
  service_ref | LowerLabel,
  input_contract | LowerLabel,
  output_contract | LowerLabel,
  writes | Array ArtifactRole,
} in
let Service = {
  service_id | LowerLabel,
  host_owned | std.contract.Equal true,
  network_authority | contracts.enum "AENEAS_CAMPAIGN_NETWORK" ["denied", "host_policy"],
  claims_verifier_soundness | std.contract.Equal false,
} in
let Budget = {
  proof_time_ms | Positive,
  stage_calls | Positive,
  tool_calls | Positive,
  output_bytes | Positive,
  retrieval_items | Positive,
  log_bytes | Positive,
  retained_artifacts | Positive,
} in
let ProfileShape = {
  schema | contracts.exact "AENEAS_CAMPAIGN_SCHEMA" campaign_schema,
  campaign_id | contracts.exact "AENEAS_CAMPAIGN_ID" campaign_identifier,
  authority | contracts.exact "AENEAS_CAMPAIGN_AUTHORITY" "none",
  recipe_workflow_ref | contracts.exact "AENEAS_CAMPAIGN_RECIPE_REF" "goose.recipe.workflow",
  model_runtime_profile_ref | contracts.exact "AENEAS_CAMPAIGN_MODEL_REF" "compat-default",
  tool_catalog_ref | contracts.exact "AENEAS_CAMPAIGN_TOOL_REF" "animus.tool-catalog.v1",
  safe_receipt_ref | contracts.exact "AENEAS_CAMPAIGN_RECEIPT_REF" "animus.safe-receipt.v1",
  completion_gate_ref | contracts.exact "AENEAS_CAMPAIGN_COMPLETION_REF" "host.authoritative-completion",
  skill | {
    skill_id | contracts.exact "AENEAS_CAMPAIGN_SKILL_ID" skill_identifier,
    source_path | contracts.exact "AENEAS_CAMPAIGN_SKILL_PATH" skill_source_path,
    source_blake3 | contracts.exact "AENEAS_CAMPAIGN_SKILL_HASH" skill_source_blake3,
    canonical | std.contract.Equal true,
    progressive_disclosure | std.contract.Equal true,
    replaces_runtime_policy | std.contract.Equal false,
  },
  roles | Array Role,
  stages | Array Stage,
  services | Array Service,
  budgets | Budget,
  artifact_roles | Array ArtifactRole,
  terminal_outcomes | Array Outcome,
  receipt_fields | Array LowerLabel,
  forbidden_receipt_fields | Array LowerLabel,
  non_claims | Array SafeText,
} in

let required_roles = ["statement-author", "statement-reviewer", "prover", "verifier"] in
let required_services = ["octet-aeneas-generation", "octet-aeneas-proof", "octet-artifact-verify"] in
let required_artifact_roles = [
  "source",
  "statement_candidate",
  "generated_model",
  "generated_manifest",
  "maintained_theorem",
  "statement_review",
  "proof_receipt",
  "verification_receipt",
  "bounded_log",
] in
let required_outcomes = [
  "proved",
  "blocked",
  "counterexample",
  "invalid_statement",
  "invalid_model",
  "budget_exhausted",
  "tool_failure",
  "denied",
  "cancelled",
  "abstained",
] in
let required_non_claim_markers = [
  "theorem truth",
  "verifier soundness",
  "reviewer independence",
  "source/model equivalence",
  "requirement adequacy",
  "product correctness",
  "authoritative completion",
] in
let profile_valid = fun profile =>
  profile.authority == "none"
  && profile.recipe_workflow_ref == "goose.recipe.workflow"
  && profile.model_runtime_profile_ref == "compat-default"
  && profile.tool_catalog_ref == "animus.tool-catalog.v1"
  && profile.safe_receipt_ref == "animus.safe-receipt.v1"
  && profile.completion_gate_ref == "host.authoritative-completion"
  && profile.skill.skill_id == skill_identifier
  && profile.skill.source_path == skill_source_path
  && profile.skill.source_blake3 == skill_source_blake3
  && profile.skill.canonical
  && profile.skill.progressive_disclosure
  && !profile.skill.replaces_runtime_policy
  && std.array.length profile.roles == std.array.length required_roles
  && std.array.length profile.roles <= maximum_roles
  && contracts.unique_by (fun role => role.assignment) profile.roles
  && std.array.all (fun role => std.array.any (fun required => required == role.assignment) required_roles) profile.roles
  && std.array.length profile.stages > 0
  && std.array.length profile.stages <= maximum_stages
  && contracts.unique_by (fun stage => stage.stage_id) profile.stages
  && std.array.all (fun stage =>
    std.array.any (fun role => role.assignment == stage.assignment) profile.roles
    && (stage.service_ref == "none" || std.array.any (fun service => service.service_id == stage.service_ref) profile.services)
    && std.string.length stage.input_contract > 0
    && std.string.length stage.output_contract > 0
    && std.array.length stage.writes > 0
  ) profile.stages
  && std.array.length profile.services == std.array.length required_services
  && std.array.length profile.services <= maximum_services
  && contracts.unique_by (fun service => service.service_id) profile.services
  && std.array.all
    (fun service => std.array.any (fun required => required == service.service_id) required_services)
    profile.services
  && profile.budgets.proof_time_ms > 0
  && profile.budgets.stage_calls > 0
  && profile.budgets.tool_calls > 0
  && profile.budgets.output_bytes > 0
  && profile.budgets.retrieval_items > 0
  && profile.budgets.log_bytes > 0
  && profile.budgets.retained_artifacts > 0
  && profile.artifact_roles == required_artifact_roles
  && profile.terminal_outcomes == required_outcomes
  && std.array.length profile.receipt_fields > 0
  && contracts.unique_by (fun field => field) profile.receipt_fields
  && std.array.length profile.forbidden_receipt_fields > 0
  && contracts.unique_by (fun field => field) profile.forbidden_receipt_fields
  && std.array.length profile.non_claims > 0
  && std.array.length profile.non_claims <= maximum_non_claims
  && std.array.all
    (fun marker => std.array.any (fun claim => std.string.is_match marker claim) profile.non_claims)
    required_non_claim_markers
  && std.array.all (fun role =>
    if role.assignment == "statement-author" then
      role.class == "implementation"
      && role.mutation == "statement"
      && role.model_profile_ref == "compat-default"
      && role.capability_ceiling == ["source.read", "statement.write"]
      && role.input_contract == "requirement-and-source"
      && role.output_contract == "statement-candidate"
      && role.service_ceiling == []
      && role.evidence_ceiling == ["statement_candidate"]
    else if role.assignment == "statement-reviewer" then
      role.class == "review"
      && role.mutation == "none"
      && role.model_profile_ref == "compat-default"
      && role.capability_ceiling == ["source.read", "statement.review"]
      && role.input_contract == "statement-candidate"
      && role.output_contract == "statement-approval"
      && role.service_ceiling == []
      && role.evidence_ceiling == ["statement_review"]
    else if role.assignment == "prover" then
      role.class == "implementation"
      && role.mutation == "maintained_theorem"
      && role.model_profile_ref == "compat-default"
      && role.capability_ceiling == ["source.read", "theorem.write", "proof.request"]
      && role.input_contract == "approved-statement-and-model"
      && role.output_contract == "proof-candidate"
      && role.service_ceiling == ["octet-aeneas-generation", "octet-aeneas-proof"]
      && role.evidence_ceiling == ["generated_model", "generated_manifest", "maintained_theorem", "proof_receipt", "bounded_log"]
    else
      role.assignment == "verifier"
      && role.class == "verification"
      && role.mutation == "none"
      && role.model_profile_ref == "compat-default"
      && role.capability_ceiling == ["artifact.read", "verification.request"]
      && role.input_contract == "proof-receipt"
      && role.output_contract == "verification-report"
      && role.service_ceiling == ["octet-artifact-verify"]
      && role.evidence_ceiling == ["verification_receipt"]
  ) profile.roles
in
let CampaignProfile = contracts.stable_validator "AENEAS_CAMPAIGN_PROFILE_INVARIANT" (fun profile =>
  profile_valid (profile | ProfileShape)
) in
let SkillManifest = {
  schema | contracts.exact "AENEAS_CAMPAIGN_MANIFEST_SCHEMA" manifest_schema,
  skill_id | contracts.exact "AENEAS_CAMPAIGN_MANIFEST_ID" skill_identifier,
  source_path | contracts.exact "AENEAS_CAMPAIGN_MANIFEST_PATH" skill_source_path,
  source_blake3 | contracts.exact "AENEAS_CAMPAIGN_MANIFEST_HASH" skill_source_blake3,
  source_count | contracts.exact "AENEAS_CAMPAIGN_SOURCE_COUNT" 1,
  canonical | std.contract.Equal true,
  safe_metadata_only | std.contract.Equal true,
  runtime_policy_authority | contracts.exact "AENEAS_CAMPAIGN_SKILL_AUTHORITY" "none",
} in
let Catalog = {
  schema | contracts.exact "AENEAS_CAMPAIGN_CATALOG_SCHEMA" catalog_schema,
  campaigns | Array {
    campaign_id | contracts.exact "AENEAS_CAMPAIGN_CATALOG_ID" campaign_identifier,
    profile_schema | contracts.exact "AENEAS_CAMPAIGN_CATALOG_PROFILE" campaign_schema,
    skill_id | contracts.exact "AENEAS_CAMPAIGN_CATALOG_SKILL" skill_identifier,
    activation | contracts.exact "AENEAS_CAMPAIGN_ACTIVATION" "explicit",
    authority | contracts.exact "AENEAS_CAMPAIGN_CATALOG_AUTHORITY" "none",
  },
} in
let Bundle = {
  schema | contracts.exact "AENEAS_CAMPAIGN_BUNDLE_SCHEMA" bundle_schema,
  campaign_profile | CampaignProfile,
  skill_manifest | SkillManifest,
  campaign_catalog | Catalog,
} in

let campaign_profile_value = {
  schema = campaign_schema,
  campaign_id = campaign_identifier,
  authority = "none",
  recipe_workflow_ref = "goose.recipe.workflow",
  model_runtime_profile_ref = "compat-default",
  tool_catalog_ref = "animus.tool-catalog.v1",
  safe_receipt_ref = "animus.safe-receipt.v1",
  completion_gate_ref = "host.authoritative-completion",
  skill = {
    skill_id = skill_identifier,
    source_path = skill_source_path,
    source_blake3 = skill_source_blake3,
    canonical = true,
    progressive_disclosure = true,
    replaces_runtime_policy = false,
  },
  roles = [
    {
      assignment = "statement-author",
      semantic_role_ref = "implementer",
      class = "implementation",
      mutation = "statement",
      model_profile_ref = "compat-default",
      capability_ceiling = ["source.read", "statement.write"],
      input_contract = "requirement-and-source",
      output_contract = "statement-candidate",
      service_ceiling = [],
      evidence_ceiling = ["statement_candidate"],
      separation_policy_ref = "subject-bound",
    },
    {
      assignment = "statement-reviewer",
      semantic_role_ref = "reviewer",
      class = "review",
      mutation = "none",
      model_profile_ref = "compat-default",
      capability_ceiling = ["source.read", "statement.review"],
      input_contract = "statement-candidate",
      output_contract = "statement-approval",
      service_ceiling = [],
      evidence_ceiling = ["statement_review"],
      separation_policy_ref = "independent-review",
    },
    {
      assignment = "prover",
      semantic_role_ref = "implementer",
      class = "implementation",
      mutation = "maintained_theorem",
      model_profile_ref = "compat-default",
      capability_ceiling = ["source.read", "theorem.write", "proof.request"],
      input_contract = "approved-statement-and-model",
      output_contract = "proof-candidate",
      service_ceiling = ["octet-aeneas-generation", "octet-aeneas-proof"],
      evidence_ceiling = ["generated_model", "generated_manifest", "maintained_theorem", "proof_receipt", "bounded_log"],
      separation_policy_ref = "subject-bound",
    },
    {
      assignment = "verifier",
      semantic_role_ref = "verifier",
      class = "verification",
      mutation = "none",
      model_profile_ref = "compat-default",
      capability_ceiling = ["artifact.read", "verification.request"],
      input_contract = "proof-receipt",
      output_contract = "verification-report",
      service_ceiling = ["octet-artifact-verify"],
      evidence_ceiling = ["verification_receipt"],
      separation_policy_ref = "independent-verifier",
    },
  ],
  stages = [
    { stage_id = "author-statement", assignment = "statement-author", depends_on = [], service_ref = "none", input_contract = "requirement-and-source", output_contract = "statement-candidate", writes = ["statement_candidate"] },
    { stage_id = "review-statement", assignment = "statement-reviewer", depends_on = ["author-statement"], service_ref = "none", input_contract = "statement-candidate", output_contract = "statement-approval", writes = ["statement_review"] },
    { stage_id = "generate-model", assignment = "prover", depends_on = ["review-statement"], service_ref = "octet-aeneas-generation", input_contract = "approved-source", output_contract = "generated-model", writes = ["generated_model", "generated_manifest"] },
    { stage_id = "maintain-theorem", assignment = "prover", depends_on = ["generate-model"], service_ref = "none", input_contract = "approved-statement-and-model", output_contract = "maintained-theorem", writes = ["maintained_theorem"] },
    { stage_id = "request-proof", assignment = "prover", depends_on = ["maintain-theorem"], service_ref = "octet-aeneas-proof", input_contract = "fresh-proof-candidate", output_contract = "proof-receipt", writes = ["proof_receipt", "bounded_log"] },
    { stage_id = "verify-proof", assignment = "verifier", depends_on = ["request-proof"], service_ref = "octet-artifact-verify", input_contract = "proof-receipt", output_contract = "verification-report", writes = ["verification_receipt"] },
  ],
  services = [
    { service_id = "octet-aeneas-generation", host_owned = true, network_authority = "denied", claims_verifier_soundness = false },
    { service_id = "octet-aeneas-proof", host_owned = true, network_authority = "denied", claims_verifier_soundness = false },
    { service_id = "octet-artifact-verify", host_owned = true, network_authority = "denied", claims_verifier_soundness = false },
  ],
  budgets = {
    proof_time_ms = proof_time_ms_limit,
    stage_calls = stage_call_limit,
    tool_calls = tool_call_limit,
    output_bytes = output_byte_limit,
    retrieval_items = retrieval_item_limit,
    log_bytes = log_byte_limit,
    retained_artifacts = retained_artifact_limit,
  },
  artifact_roles = required_artifact_roles,
  terminal_outcomes = required_outcomes,
  receipt_fields = [
    "profile_identity",
    "skill_identity",
    "plan_identity",
    "role_identity",
    "assignment_identity",
    "subject_identity",
    "statement_identity",
    "model_identity",
    "budget_identity",
    "stage_identity",
    "bounded_counts",
    "outcome",
    "evidence_identities",
    "issue_classes",
    "redaction_markers",
  ],
  forbidden_receipt_fields = [
    "raw_prompt",
    "hidden_reasoning",
    "credential",
    "private_path",
    "source_body",
    "proof_body",
    "tool_arguments",
    "tool_output",
    "provider_payload",
    "unbounded_log",
  ],
  non_claims = [
    "campaign conformance does not prove theorem truth",
    "campaign conformance does not prove verifier soundness",
    "supplied separation dimensions do not prove reviewer independence",
    "generated and maintained identities do not prove source/model equivalence",
    "statement approval does not prove requirement adequacy",
    "a proved candidate does not prove product correctness or release readiness",
    "campaign receipts do not grant authoritative completion",
  ],
} | CampaignProfile in

let bundle_value = {
  schema = bundle_schema,
  campaign_profile = campaign_profile_value,
  skill_manifest = {
    schema = manifest_schema,
    skill_id = skill_identifier,
    source_path = skill_source_path,
    source_blake3 = skill_source_blake3,
    source_count = 1,
    canonical = true,
    safe_metadata_only = true,
    runtime_policy_authority = "none",
  },
  campaign_catalog = {
    schema = catalog_schema,
    campaigns = [{
      campaign_id = campaign_identifier,
      profile_schema = campaign_schema,
      skill_id = skill_identifier,
      activation = "explicit",
      authority = "none",
    }],
  },
} | Bundle in

{
  campaign_profile_contract = CampaignProfile,
  campaign_bundle_contract = Bundle,
  aeneas_proof_campaign_bundle = bundle_value,
}
