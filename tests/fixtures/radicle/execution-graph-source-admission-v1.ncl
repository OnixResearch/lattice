# Typed producer-evidence admission for Lattice's execution-graph source cutover.
let SourceAdmission = {
  schema_version | Number,
  receipt_type | String,
  status | String,
  observed_date | String,
  repository | { rid | String, reviewed_commit | String, source_archive_blake3 | String, license | String },
  publication | { receipt_type | String, receipt_blake3 | String, publication_revision | String, identity_revision | String, delegate_threshold | Number, signed_refs_feature | String },
  onix | { receipt_type | String, receipt_blake3 | String, acceptance_revision | String },
  onixos | { acceptance_revision | String, acceptance_evidence | String },
  sources | { cargo_git | String, cargo_locked_source | String, nix_git | String, nix_nar_hash | String },
  availability | { seed_count | Number, aspen_only | String, desktop_only | String, https | String, receive_pack_status | Number, unknown_rid_status | Number, github_fallback_configured | Bool },
  compatibility | { consumer_baseline | String, vendor_lib_blake3 | String, reviewed_lib_blake3 | String, source_diff_lines | Number, runner_tests | Number },
  consumer | { repository | String, phase | String },
  non_claims | Array String,
}
in
let admitted_seed_count = 2 in
let required_delegate_threshold = 2 in
let not_found_status = 404 in
let source_diff_line_count = 0 in
let runner_test_count = 21 in
let admission | SourceAdmission = {
  schema_version = 1,
  receipt_type = "lattice.radicle-execution-graph-source-admission.v1",
  status = "accepted",
  observed_date = "2026-07-26",
  repository = {
    rid = "rad:z2oYsb9jGTyp68BKYhzpivY1eK58a",
    reviewed_commit = "03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    source_archive_blake3 = "4b5aa3756369236fc82fbbf501d35993cfa208f142694cdd30ca370d6241192c",
    license = "AGPL-3.0-or-later",
  },
  publication = {
    receipt_type = "execution-graph.radicle-publication.v1",
    receipt_blake3 = "5573a000f71f1992ed3aa6ddb1197aca29b2b551ef58643b5bd930bef78270cf",
    publication_revision = "fffc6968f6e17c235bcfa708fb00749a379c15ff",
    identity_revision = "d407356b5fd5465c62aca4d7d6c61156947cbac1",
    delegate_threshold = required_delegate_threshold,
    signed_refs_feature = "parent",
  },
  onix = {
    receipt_type = "onix.radicle-execution-graph-source-admission.v1",
    receipt_blake3 = "f69725f837567bcb2a86fa4b0b6f9a5bfa413b34c78cb3b8bd35f07707e9ed61",
    acceptance_revision = "a41a4fb6d69b6f2e4028d33e54c5650ab11c906d",
  },
  onixos = {
    acceptance_revision = "04d20e0cd880030ae1745c12d2b7f0c3988fe58c",
    acceptance_evidence = "onixos:.cairn/archive/2026-07-25-admit-execution-graph-radicle-source/evidence/source-acceptance-2026-07-26.md",
  },
  sources = {
    cargo_git = "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git",
    cargo_locked_source = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b#03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    nix_git = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    nix_nar_hash = "sha256-SzdJGsS631Hk3CbPLThz+2YPgvdoB1cXb3BPusJTX8I=",
  },
  availability = {
    seed_count = admitted_seed_count,
    aspen_only = "verified",
    desktop_only = "verified",
    https = "verified",
    receive_pack_status = not_found_status,
    unknown_rid_status = not_found_status,
    github_fallback_configured = false,
  },
  compatibility = {
    consumer_baseline = "c8d56ad83a57287064edb85f58808f53e4bf4536",
    vendor_lib_blake3 = "91450b94f6256c71057507b6ae97f63e505dc57377198be38087530bedefccb4",
    reviewed_lib_blake3 = "91450b94f6256c71057507b6ae97f63e505dc57377198be38087530bedefccb4",
    source_diff_lines = source_diff_line_count,
    runner_tests = runner_test_count,
  },
  consumer = { repository = "lattice", phase = "pre-cutover-admission" },
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
}
in
admission
