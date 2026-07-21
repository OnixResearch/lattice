# Nickel is the source of truth for repo-owned tool configuration.
# Export the native files with scripts/config/export-nickel-configs.

let CargoAliasConfig = {
  verify | String,
} in
let CargoNetConfig = {
  "git-fetch-with-cli" | Bool,
} in
let CargoConfig = {
  alias | CargoAliasConfig,
  net | CargoNetConfig,
} in

let ClippyConfig = {
  "avoid-breaking-exported-api" | Bool,
  "cognitive-complexity-threshold" | Number,
  "too-many-arguments-threshold" | Number,
  "too-many-lines-threshold" | Number,
  "type-complexity-threshold" | Number,
} in

let OctetConfig = {
  lint_levels | { _ | String },
} in
let DylintConfig = {
  octet | OctetConfig,
} in

let PreCommitHook = {
  id | String,
  name | String,
  entry | String,
  language | String,
  pass_filenames | Bool,
  stages | Array String,
} in
let PreCommitRepo = {
  repo | String,
  hooks | Array PreCommitHook,
} in
let PreCommitConfig = {
  repos | Array PreCommitRepo,
} in

let cargo_verify_command = "test --workspace" in
let clippy_breaking_exported_api_allowed = false in
let clippy_cognitive_complexity_threshold = 15 in
let clippy_too_many_arguments_threshold = 4 in
let clippy_too_many_lines_threshold = 100 in
let clippy_type_complexity_threshold = 200 in
let git_fetch_with_cli_enabled = true in
let octet_lint_level = "deny" in
let octet_hook_id = "octet-lints" in
let octet_hook_name = "Octet lints" in
let octet_hook_entry = "scripts/hooks/octet-lints" in
let nickel_config_hook_id = "nickel-configs" in
let nickel_config_hook_name = "Nickel generated configs" in
let nickel_config_hook_entry = "scripts/config/check-nickel-configs" in
let router_policy_hook_id = "router-policy" in
let router_policy_hook_name = "Router policy conformance" in
let router_policy_hook_entry = "scripts/cairn/check-router-policy" in
let script_language = "script" in
let pre_commit_stage = "pre-commit" in
let local_repo = "local" in
{
  cargo_config | CargoConfig = {
    alias = {
      verify = cargo_verify_command,
    },
    net = {
      "git-fetch-with-cli" = git_fetch_with_cli_enabled,
    },
  },

  clippy_config | ClippyConfig = {
    "avoid-breaking-exported-api" = clippy_breaking_exported_api_allowed,
    "cognitive-complexity-threshold" = clippy_cognitive_complexity_threshold,
    "too-many-arguments-threshold" = clippy_too_many_arguments_threshold,
    "too-many-lines-threshold" = clippy_too_many_lines_threshold,
    "type-complexity-threshold" = clippy_type_complexity_threshold,
  },

  dylint_config | DylintConfig = {
    octet = {
      lint_levels = {
        compound_assertion = octet_lint_level,
        function_length = octet_lint_level,
        acronym_style = octet_lint_level,
        numeric_units = octet_lint_level,
        magic_number = octet_lint_level,
        excessive_file_length = octet_lint_level,
        module_file_count = octet_lint_level,
        underscore_in_module_filename = octet_lint_level,
        non_trait_imports = octet_lint_level,
        renamed_imports = octet_lint_level,
        path_segment_repetition = octet_lint_level,
        unbounded_loop = octet_lint_level,
        compound_condition = octet_lint_level,
        verified_purity = octet_lint_level,
        unjustified_allow = octet_lint_level,
        nested_conditionals = octet_lint_level,
        assertion_density = octet_lint_level,
        multi_lock_ordering = octet_lint_level,
        explicit_defaults = octet_lint_level,
        unbounded_channel = octet_lint_level,
        unbounded_collection_growth = octet_lint_level,
        sentinel_fallback = octet_lint_level,
        usize_in_public_api = octet_lint_level,
        borrowed_argument_types = octet_lint_level,
        deref_polymorphism = octet_lint_level,
        clone_then_borrow = octet_lint_level,
        too_many_parameters = octet_lint_level,
        negated_predicate = octet_lint_level,
        bool_naming = octet_lint_level,
        ignored_result = octet_lint_level,
        drop_must_use = octet_lint_level,
        fallible_int_fallback = octet_lint_level,
        unchecked_narrowing = octet_lint_level,
        unchecked_division = octet_lint_level,
        contradictory_time = octet_lint_level,
        ambient_clock = octet_lint_level,
        catch_all_on_enum = octet_lint_level,
        platform_dependent_cast = octet_lint_level,
        float_for_currency = octet_lint_level,
        ambiguous_params = octet_lint_level,
        raw_arithmetic_overflow = octet_lint_level,
        no_unwrap = octet_lint_level,
        no_panic = octet_lint_level,
        public_unsafe_api = octet_lint_level,
        no_todo = octet_lint_level,
        unjustified_no_todo_allow = octet_lint_level,
        no_recursion = octet_lint_level,
      },
    },
  },

  pre_commit_config | PreCommitConfig = {
    repos = [
      {
        repo = local_repo,
        hooks = [
          {
            id = nickel_config_hook_id,
            name = nickel_config_hook_name,
            entry = nickel_config_hook_entry,
            language = script_language,
            pass_filenames = false,
            stages = [pre_commit_stage],
          },
          {
            id = router_policy_hook_id,
            name = router_policy_hook_name,
            entry = router_policy_hook_entry,
            language = script_language,
            pass_filenames = false,
            stages = [pre_commit_stage],
          },
          {
            id = octet_hook_id,
            name = octet_hook_name,
            entry = octet_hook_entry,
            language = script_language,
            pass_filenames = false,
            stages = [pre_commit_stage],
          },
        ],
      },
    ],
  },
}
