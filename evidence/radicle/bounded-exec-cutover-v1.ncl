# Canonical typed receipt for Lattice's exact-revision Bounded Exec source cutover.
let CutoverReceipt = {
  schema_version | Number,
  receipt_type | String,
  status | String,
  observed_date | String,
  producer | { publication_receipt_blake3 | String, source_admission_blake3 | String, onixos_acceptance_revision | String, onixos_catalog_blake3 | String },
  repository | { rid | String, reviewed_commit | String, source_archive_blake3 | String },
  consumer | { repository | String, source_revision | String },
  cargo | { manifest_url | String, locked_source | String, packages | Array String, github_fallback_configured | Bool },
  nix | { flake_url | String, locked_url | String, locked_revision | String, nar_hash | String, revision_count | Number },
  crane | { cargo_source | String, substituted_input | String },
  behavior | { repository_validation_tests | Number, bounded_process_tests | Number, completion | String, failure | String, deadline | String, truncation | String, cancellation | String, process_group_teardown | String },
  availability | { primary_only | String, replica_only | String, unavailable_paths_diagnostic | String, github_fallback_configured | Bool },
  validation | { bounded_source_check | String, cairn_tasks_gate | String, scripts_check | String, octet | String, nix_flake_check | String },
  non_claims | Array String,
  evidence | Array String,
}
in
let repository_validation_test_count = 6 in
let bounded_process_test_count = 21 in
let nix_revision_count = 3 in
let receipt | CutoverReceipt = {
  schema_version = 1,
  receipt_type = "lattice.radicle-bounded-exec-cutover.v1",
  status = "accepted",
  observed_date = "2026-07-25",
  producer = {
    publication_receipt_blake3 = "6eab0250b4bfefb6f6e91ec75d9786a76ded20bc052e2e624b44c6bc5b1cc9f7",
    source_admission_blake3 = "c7707d1fb3367e752ddde42ac1f89df1808551b9b7560b29564a959d911aabe3",
    onixos_acceptance_revision = "b5ba554",
    onixos_catalog_blake3 = "f132c01a9d0edb4141201146c1dd5dc1eacfe77a2b080e6f99b2c4181d856655",
  },
  repository = {
    rid = "rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo",
    reviewed_commit = "29dac88ecded94457572db3fdfaaaab95fa91525",
    source_archive_blake3 = "4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927",
  },
  consumer = {
    repository = "lattice",
    source_revision = "d69ec49",
  },
  cargo = {
    manifest_url = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git",
    locked_source = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525",
    packages = ["bounded-exec", "bounded-exec-core"],
    github_fallback_configured = false,
  },
  nix = {
    flake_url = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525",
    locked_url = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git",
    locked_revision = "29dac88ecded94457572db3fdfaaaab95fa91525",
    nar_hash = "sha256-BVmqyUYyoNpY6LfOABxwPO3DY88ZtXeKNg3TPoGCcL0=",
    revision_count = nix_revision_count,
  },
  crane = {
    cargo_source = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525",
    substituted_input = "bounded-exec",
  },
  behavior = {
    repository_validation_tests = repository_validation_test_count,
    bounded_process_tests = bounded_process_test_count,
    completion = "passed",
    failure = "passed",
    deadline = "passed",
    truncation = "passed",
    cancellation = "passed",
    process_group_teardown = "passed",
  },
  availability = {
    primary_only = "verified",
    replica_only = "verified",
    unavailable_paths_diagnostic = "source.bounded_exec_identity",
    github_fallback_configured = false,
  },
  validation = {
    bounded_source_check = "passed",
    cairn_tasks_gate = "passed-before-closeout",
    scripts_check = "blocked-pre-existing-traceability-fixture",
    octet = "blocked-pre-existing-892-finding-baseline",
    nix_flake_check = "blocked-pre-existing-octet-baseline",
  },
  non_claims = [
    "radicle-protocol-correctness",
    "bounded-exec-correctness",
    "host-sandboxing",
    "consumer-release-readiness",
    "octet-baseline-cleanliness",
    "ci-correctness",
    "third-party-source-github-independence",
    "whole-stack-github-independence",
  ],
  evidence = [
    "cairn/changes/consume-bounded-exec-from-radicle/evidence/source-cutover-2026-07-25.md",
    "tests/fixtures/radicle/bounded-exec-source-admission-v1.json",
  ],
}
in
receipt
