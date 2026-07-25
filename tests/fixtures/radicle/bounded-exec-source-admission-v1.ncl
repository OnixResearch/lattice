# Typed producer-evidence admission for the Lattice Bounded Exec source cutover.
let SourceAdmission = {
  schema | String,
  status | String,
  observed_date | String,
  repository | { rid | String, reviewed_commit | String, source_archive_blake3 | String },
  publication | { receipt_type | String, receipt_blake3 | String, source_revision | String },
  onixos | { catalog_blake3 | String, publication_acceptance_revision | String, acceptance_evidence | String },
  sources | { cargo_git | String, nix_git | String, crane_source | String },
  availability | { seed_count | Number, primary_only | String, replica_only | String, github_fallback_configured | Bool },
  consumer | { repository | String, phase | String },
  non_claims | Array String,
}
in
let admitted_seed_count = 2 in
let admission | SourceAdmission = {
  schema = "lattice.radicle-source-admission.v1",
  status = "accepted",
  observed_date = "2026-07-25",
  repository = {
    rid = "rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo",
    reviewed_commit = "29dac88ecded94457572db3fdfaaaab95fa91525",
    source_archive_blake3 = "4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927",
  },
  publication = {
    receipt_type = "bounded-exec.radicle-publication.v1",
    receipt_blake3 = "6eab0250b4bfefb6f6e91ec75d9786a76ded20bc052e2e624b44c6bc5b1cc9f7",
    source_revision = "7f7bd84",
  },
  onixos = {
    catalog_blake3 = "f132c01a9d0edb4141201146c1dd5dc1eacfe77a2b080e6f99b2c4181d856655",
    publication_acceptance_revision = "b5ba554",
    acceptance_evidence = "onixos:cairn/changes/add-radicle-forge-pilot/evidence/bounded-exec-publication-acceptance-2026-07-25.md",
  },
  sources = {
    cargo_git = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git",
    nix_git = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525",
    crane_source = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525",
  },
  availability = {
    seed_count = admitted_seed_count,
    primary_only = "verified",
    replica_only = "verified",
    github_fallback_configured = false,
  },
  consumer = {
    repository = "lattice",
    phase = "pre-cutover-admission",
  },
  non_claims = [
    "lattice-cutover-complete",
    "lattice-behavior-correctness",
    "lattice-release-readiness",
    "bounded-exec-source-correctness",
    "ci-correctness",
    "whole-stack-github-independence",
    "third-party-source-github-independence",
  ],
}
in
admission
