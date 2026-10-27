# Typed, non-default Pijul CLI adapter profile.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let PijulIdentity = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^[A-Z2-7]{53}$" value) in
let Blake3Hex = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Operation = [| 'log, 'diff, 'channel_list, 'change, 'record, 'unrecord, 'apply, 'fork, 'reset, 'channel_mutation |] in
let Projection = [| 'checkpoint, 'snapshot_ledger, 'provenance, 'workspace |] in
let Profile = {
  schema | NonEmpty,
  enabled_by_default | Bool,
  cohort | {
    release | NonEmpty,
    source_repository | NonEmpty,
    source_change | PijulIdentity,
    source_state | PijulIdentity,
    crate_checksum_sha256 | Blake3Hex,
    nix_source_hash | NonEmpty,
    nix_output_basename | NonEmpty,
    change_hash_format | NonEmpty,
    change_hash_chars | Positive,
    state_id_format | NonEmpty,
    state_id_chars | Positive,
    log_output_format | NonEmpty,
    diff_output_format | NonEmpty,
    channel_output_format | NonEmpty,
    license | NonEmpty,
    cohort_identity | Blake3Hex,
  },
  operations | {
    query_allowlist | Array Operation,
    protected_mutations | Array Operation,
    unsupported | Array Operation,
    confirmation_required | Bool,
  },
  bounds | {
    max_log_entries | Positive,
    max_diff_records | Positive,
    max_record_records | Positive,
    max_path_bytes | Positive,
    max_channel_bytes | Positive,
    max_diagnostic_bytes | Positive,
    max_stdout_bytes | Positive,
    max_stderr_bytes | Positive,
    max_stdin_bytes | Positive,
    max_memory_bytes | Positive,
    max_cpu_milliseconds | Positive,
    deadline_milliseconds | Positive,
    minimum_hash_prefix_chars | Positive,
  },
  channels | {
    allowlist | Array NonEmpty,
    allow_unlisted | Bool,
  },
  identity | {
    load_local_identity_files | Bool,
    report_author_class | Bool,
    report_descriptive_timestamp | Bool,
    signature_statuses | Array NonEmpty,
    signature_authenticity_claim | Bool,
  },
  projections | Array Projection,
  receipt | {
    repository_class | NonEmpty,
    status | NonEmpty,
    redaction_class | NonEmpty,
    omission_class | NonEmpty,
    truncation_class | NonEmpty,
    projection_targets | Array Projection,
    non_claims | Array NonEmpty,
  },
  evidence | {
    metadata_only | Bool,
    raw_diffs_allowed | Bool,
    file_bodies_allowed | Bool,
    change_descriptions_allowed | Bool,
    author_details_allowed | Bool,
    key_material_allowed | Bool,
    private_paths_allowed | Bool,
    credentials_allowed | Bool,
    unrestricted_diagnostics_allowed | Bool,
    non_claims | Array NonEmpty,
  },
} in
let configured_max_log_entries = 128 in
let configured_max_diff_records = 512 in
let configured_max_record_records = 512 in
let configured_max_path_bytes = 256 in
let configured_max_channel_bytes = 64 in
let configured_max_diagnostic_bytes = 2048 in
let configured_max_stdout_bytes = 1048576 in
let configured_max_stderr_bytes = 65536 in
let configured_max_stdin_bytes = 4096 in
let configured_max_memory_bytes = 268435456 in
let configured_max_cpu_milliseconds = 30000 in
let configured_deadline_milliseconds = 30000 in
let configured_minimum_hash_prefix_chars = 8 in
{
  pijul_vcs_adapter | Profile = {
    schema = "animus.pijul-vcs-adapter.v1",
    enabled_by_default = false,
    cohort = {
      release = "1.0.0-beta.18",
      source_repository = "https://nest.pijul.com/pijul/pijul",
      source_change = "QRYOAJLYA5KO7JAYU6LMYHPU3NSZCTUMYEPCK3HNZIRIDZDWSMBAC",
      source_state = "RISR562S7FZP3NNYDFVBQUMDEGB6YSSMSH2RGZDRYBIPGVQDSLXQC",
      crate_checksum_sha256 = "0500d14ab41adea4f0d71f03b9d1be895a0638e025204f5b6ab90e46c40684ff",
      nix_source_hash = "sha256-vU41JiuxB6Bsi88st/tkt02054oN3HEN52pnLu5hMA4=",
      nix_output_basename = "pijul-1.0.0-beta.18",
      change_hash_format = "blake3-base32-nopad-uppercase",
      change_hash_chars = 53,
      state_id_format = "merkle-blake3-base32-nopad-uppercase",
      state_id_chars = 53,
      log_output_format = "pijul-log-hash-json-v1-beta18",
      diff_output_format = "pijul-diff-json-v1-beta18",
      channel_output_format = "pijul-channel-lines-v1-beta18",
      license = "GPL-2.0-or-later",
      cohort_identity = "ed1fcd07852103dd8ea0b2e824aac6cad51406c987e54240077888ce796aee2b",
    },
    operations = {
      query_allowlist = ['log, 'diff, 'channel_list],
      protected_mutations = ['record, 'unrecord, 'apply, 'fork, 'reset, 'channel_mutation],
      unsupported = ['change],
      confirmation_required = true,
    },
    bounds = {
      max_log_entries = configured_max_log_entries,
      max_diff_records = configured_max_diff_records,
      max_record_records = configured_max_record_records,
      max_path_bytes = configured_max_path_bytes,
      max_channel_bytes = configured_max_channel_bytes,
      max_diagnostic_bytes = configured_max_diagnostic_bytes,
      max_stdout_bytes = configured_max_stdout_bytes,
      max_stderr_bytes = configured_max_stderr_bytes,
      max_stdin_bytes = configured_max_stdin_bytes,
      max_memory_bytes = configured_max_memory_bytes,
      max_cpu_milliseconds = configured_max_cpu_milliseconds,
      deadline_milliseconds = configured_deadline_milliseconds,
      minimum_hash_prefix_chars = configured_minimum_hash_prefix_chars,
    },
    channels = {
      allowlist = ["main"],
      allow_unlisted = false,
    },
    identity = {
      load_local_identity_files = false,
      report_author_class = false,
      report_descriptive_timestamp = false,
      signature_statuses = ["not-reported", "present-unverified", "absent", "unsupported"],
      signature_authenticity_claim = false,
    },
    projections = ['checkpoint, 'snapshot_ledger, 'provenance, 'workspace],
    receipt = {
      repository_class = "explicit-local",
      status = "validated",
      redaction_class = "metadata-only",
      omission_class = "content-omitted",
      truncation_class = "not-truncated",
      projection_targets = ['checkpoint, 'snapshot_ledger, 'provenance, 'workspace],
      non_claims = [
        "not-repository-integrity",
        "not-change-semantic-correctness",
        "not-content-correctness",
        "not-merge-safety",
        "not-authorship-authenticity",
        "not-signature-verification",
        "not-key-custody",
        "not-network-or-remote-correctness",
        "not-conflict-resolution-quality",
        "not-persistence-or-durability",
        "not-production-readiness",
        "not-release-eligibility",
      ],
    },
    evidence = {
      metadata_only = true,
      raw_diffs_allowed = false,
      file_bodies_allowed = false,
      change_descriptions_allowed = false,
      author_details_allowed = false,
      key_material_allowed = false,
      private_paths_allowed = false,
      credentials_allowed = false,
      unrestricted_diagnostics_allowed = false,
      non_claims = [
        "not-repository-integrity",
        "not-change-semantic-correctness",
        "not-content-correctness",
        "not-merge-safety",
        "not-authorship-authenticity",
        "not-signature-verification",
        "not-key-custody",
        "not-network-or-remote-correctness",
        "not-conflict-resolution-quality",
        "not-persistence-or-durability",
        "not-production-readiness",
        "not-release-eligibility",
      ],
    },
  },
}
