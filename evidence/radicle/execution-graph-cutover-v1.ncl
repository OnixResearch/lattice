let CutoverReceipt = {
  schema_version | Number,
  receipt_type | String,
  status | String,
  observed_date | String,
  producer | { publication_receipt_blake3 | String, onix_admission_blake3 | String, onixos_acceptance_revision | String, source_admission_blake3 | String },
  repository | { rid | String, reviewed_commit | String, source_archive_blake3 | String },
  consumer | { repository | String, baseline_revision | String, cutover_revision | String },
  cargo | { dependency_key | String, package | String, git | String, locked_source | String, vendor_present | Bool, fallback_configured | Bool },
  nix | { input | String, locked_revision | String, nar_hash | String, crane_source | String },
  compatibility | { source_diff_lines | Number, source_admission_tests | Number, direct_graph_tests | Number, runner_tests_before | Number, runner_tests_after | Number },
  validation_baseline | { records | Number, unique_ids | Number, lint_location_inventory | String, previous_hash | String, accepted_hash | String, new_ids | Number, stale_ids | Number },
  validation | { scripts_check | String, octet | String, cairn | String, nix_flake_check | String, commit_hook | String },
  non_claims | Array String,
  evidence | Array String,
}
in
let source_diff_line_count = 0 in
let source_admission_test_count = 3 in
let direct_graph_test_count = 2 in
let runner_test_count = 21 in
let baseline_record_count = 1141 in
let baseline_unique_id_count = 892 in
let no_drift_count = 0 in
let receipt | CutoverReceipt = {
  schema_version = 1,
  receipt_type = "lattice.radicle-execution-graph-cutover.v1",
  status = "accepted",
  observed_date = "2026-07-26",
  producer = {
    publication_receipt_blake3 = "5573a000f71f1992ed3aa6ddb1197aca29b2b551ef58643b5bd930bef78270cf",
    onix_admission_blake3 = "f69725f837567bcb2a86fa4b0b6f9a5bfa413b34c78cb3b8bd35f07707e9ed61",
    onixos_acceptance_revision = "04d20e0cd880030ae1745c12d2b7f0c3988fe58c",
    source_admission_blake3 = "06e3d56202412f4ed7c59152b85de53ace97d958be8383353974a5348972e6f8",
  },
  repository = {
    rid = "rad:z2oYsb9jGTyp68BKYhzpivY1eK58a",
    reviewed_commit = "03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    source_archive_blake3 = "4b5aa3756369236fc82fbbf501d35993cfa208f142694cdd30ca370d6241192c",
  },
  consumer = {
    repository = "lattice",
    baseline_revision = "c8d56ad83a57287064edb85f58808f53e4bf4536",
    cutover_revision = "a85da0afc2f658ca29fcb8ebe6927896ca85e085",
  },
  cargo = {
    dependency_key = "seaglass-execution-graph",
    package = "execution-graph-core",
    git = "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git",
    locked_source = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b#03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    vendor_present = false,
    fallback_configured = false,
  },
  nix = {
    input = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    locked_revision = "03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    nar_hash = "sha256-SzdJGsS631Hk3CbPLThz+2YPgvdoB1cXb3BPusJTX8I=",
    crane_source = "execution-graph",
  },
  compatibility = {
    source_diff_lines = source_diff_line_count,
    source_admission_tests = source_admission_test_count,
    direct_graph_tests = direct_graph_test_count,
    runner_tests_before = runner_test_count,
    runner_tests_after = runner_test_count,
  },
  validation_baseline = {
    records = baseline_record_count,
    unique_ids = baseline_unique_id_count,
    lint_location_inventory = "equal",
    previous_hash = "b3:ff87ad266a61563c76bc4f5835896a080c6d3580a1eb514c1cdf49bda63394be",
    accepted_hash = "b3:0056baa6ca2ace924ae9d98c8a1e6f028fbcdc3af99a838af140b5eecfd337b2",
    new_ids = no_drift_count,
    stale_ids = no_drift_count,
  },
  validation = {
    scripts_check = "passed",
    octet = "passed",
    cairn = "passed",
    nix_flake_check = "passed-x86_64-linux",
    commit_hook = "passed",
  },
  non_claims = [
    "source-correctness",
    "workflow-correctness",
    "scheduler-fairness",
    "effect-safety",
    "persistence-correctness",
    "radicle-availability",
    "release-readiness",
    "whole-stack-github-independence",
  ],
  evidence = [
    "cairn/archive/2026-07-26-adopt-radicle-execution-graph/evidence/source-cutover-2026-07-26.md",
    "tests/fixtures/radicle/execution-graph-source-admission-v1.json",
  ],
}
in
receipt
