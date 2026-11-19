# Typed standalone Nickel export requests used during the fail-closed dual-run cutover.
let NonEmpty = std.contract.from_predicate (fun value => value != "") in
let RelativePath = NonEmpty in
let Format = [| 'json, 'toml, 'yaml, 'raw |] in
let Request = {
  schema | NonEmpty,
  family_id | NonEmpty,
  source | RelativePath,
  dependencies | Array RelativePath,
  import_paths | Array RelativePath,
  selector | String,
  contract | String,
  format | Format,
  destination | RelativePath,
  allow_secret_material | Bool,
} in
let request = fun family_value source_value dependency_values selector_value format_value destination_value allow_secret_markers =>
  {
    schema = "onix-nickel-export-request/v1",
    family_id = family_value,
    source = source_value,
    dependencies = dependency_values,
    import_paths = ["config", "policy/contracts"],
    selector = selector_value,
    contract = "",
    format = format_value,
    destination = destination_value,
    allow_secret_material = allow_secret_markers,
  }
in
{
  requests | {
    cargo_config | Request,
    clippy_config | Request,
    dylint_config | Request,
    pre_commit_config | Request,
    recipe_workflows | Request,
    model_runtime_profiles | Request,
    agent_role_profiles | Request,
    codebase_graph_profile | Request,
    mesh_provider_profile | Request,
    spacewasm_embedded_compat | Request,
    harbor_evaluation_profile | Request,
    campaign_manager_eval_tasks | Request,
    campaign_manager_local_scorer_fixture | Request,
    campaign_manager_local_scorer_eval | Request,
    campaign_manager_frozen_policy_fixture | Request,
    campaign_manager_frozen_policy_eval | Request,
    frozen_policy_example_tree | Request,
    frozen_policy_example_mlp | Request,
    host_control_source | Request,
  } = {
    cargo_config = request "animus.repo.cargo" "config/repo.ncl" ["config/agent-check-registry.ncl"] "cargo_config" 'toml ".cargo/config.toml" false,
    clippy_config = request "animus.repo.clippy" "config/repo.ncl" ["config/agent-check-registry.ncl"] "clippy_config" 'toml "clippy.toml" false,
    dylint_config = request "animus.repo.dylint" "config/repo.ncl" ["config/agent-check-registry.ncl"] "dylint_config" 'toml "dylint.toml" false,
    pre_commit_config = request "animus.repo.pre-commit" "config/repo.ncl" ["config/agent-check-registry.ncl"] "pre_commit_config" 'yaml ".pre-commit-config.yaml" false,
    recipe_workflows = request "animus.recipe-workflows" "config/recipe-workflows.ncl" [] "recipe_workflow_catalog" 'json "policy/recipe-workflows/generated/recipe-workflows.json" false,
    model_runtime_profiles = request "animus.model-runtime-profiles" "config/model-runtime-profiles.ncl" ["policy/contracts/primitives.ncl"] "model_runtime_profile_set" 'json "config/generated/model-runtime-profiles.json" true,
    agent_role_profiles = request "animus.agent-role-profiles" "config/agent-role-profiles.ncl" ["policy/contracts/primitives.ncl"] "agent_role_catalog" 'json "config/generated/agent-role-profiles.json" true,
    codebase_graph_profile = request "animus.codebase-graph-profile" "config/codebase-graph-profile.ncl" [] "codebase_graph_profile" 'json "config/generated/codebase-graph-profile.json" false,
    mesh_provider_profile = request "animus.mesh-provider-profile" "config/mesh-provider-profile.ncl" [] "mesh_provider_profile" 'json "config/generated/mesh-provider-profile.json" false,
    spacewasm_embedded_compat = request "animus.spacewasm-embedded-compat" "config/spacewasm-embedded-compat.ncl" [] "spacewasm_embedded_compat" 'json "config/generated/spacewasm-embedded-compat.json" false,
    harbor_evaluation_profile = request "animus.harbor-evaluation-profile" "config/harbor-evaluation-profile.ncl" [] "harbor_evaluation_profile" 'json "config/generated/harbor-evaluation-profile.json" false,
    campaign_manager_eval_tasks = request "animus.campaign-manager-eval-tasks" "config/campaign-manager-eval-tasks.ncl" [] "campaign_manager_eval_tasks" 'json "config/generated/campaign-manager-eval-tasks.json" false,
    campaign_manager_local_scorer_fixture = request "animus.campaign-manager-local-scorer-fixture" "config/campaign-manager-local-scorer-fixture.ncl" [] "campaign_manager_local_scorer_fixture" 'json "config/generated/campaign-manager-local-scorer-fixture.json" false,
    campaign_manager_local_scorer_eval = request "animus.campaign-manager-local-scorer-eval" "config/campaign-manager-local-scorer-eval.ncl" [] "campaign_manager_local_scorer_eval" 'json "config/generated/campaign-manager-local-scorer-eval.json" false,
    campaign_manager_frozen_policy_fixture = request "animus.campaign-manager-frozen-policy-fixture" "config/campaign-manager-frozen-policy-fixture.ncl" ["config/frozen-policy-contract.ncl"] "campaign_manager_frozen_policy_fixture" 'json "config/generated/campaign-manager-frozen-policy-fixture.json" false,
    campaign_manager_frozen_policy_eval = request "animus.campaign-manager-frozen-policy-eval" "config/campaign-manager-frozen-policy-eval.ncl" [] "campaign_manager_frozen_policy_eval" 'json "config/generated/campaign-manager-frozen-policy-eval.json" false,
    frozen_policy_example_tree = request "animus.frozen-policy-example-tree" "config/frozen-policy-examples.ncl" ["config/frozen-policy-contract.ncl"] "frozen_policy_example_tree" 'json "config/generated/frozen-policy-example-tree.json" false,
    frozen_policy_example_mlp = request "animus.frozen-policy-example-mlp" "config/frozen-policy-examples.ncl" ["config/frozen-policy-contract.ncl"] "frozen_policy_example_mlp" 'json "config/generated/frozen-policy-example-mlp.json" false,
    host_control_source = request "animus.host-control-source" "config/host-control-source.ncl" [] "host_control_source" 'json "config/generated/host-control-source.json" false,
  },
}
