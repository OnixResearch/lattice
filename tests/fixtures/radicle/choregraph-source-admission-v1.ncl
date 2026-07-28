let Hex40 =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
  )
in
let Hex64 =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
  )
in
let ExactUrl =
  std.contract.from_predicate (fun value =>
    value == "https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git"
  )
in
let Admission = {
  schema | String,
  status | [| 'accepted |],
  rid | String,
  revision | Hex40,
  source_url | ExactUrl,
  source_archive_blake3 | Hex64,
  release_bundle_blake3 | Hex64,
  candidate_blake3 | Hex64,
  license | String,
  cargo_revision | Hex40,
  nix_revision | Hex40,
  prerequisites = {
    execution_graph_baseline | Bool,
    nickel_evaluator_baseline | Bool,
  },
  evidence = {
    single_step_proof | Bool,
    trace_replay_proof | Bool,
    trusted_surface | Bool,
    kamacite_artifacts | Bool,
    nickel_boundary | Bool,
    lattice_conformance | Bool,
    assumptions | Bool,
    non_claims | Bool,
  },
  forbidden_fallbacks | Array String,
  non_claims | Array String,
}
in
{
  admission | Admission = {
    schema = "lattice.choregraph-source-admission.v1",
    status = 'accepted,
    rid = "rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF",
    revision = "81b260da2256147ec82cb5dd44c0a0c1dcb7500d",
    source_url = "https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git",
    source_archive_blake3 = "41adfbdae4bc5b688ef92ab11cd39ed9c8f572bf6214f522d9b4a1290e5f84d5",
    release_bundle_blake3 = "6912c485313908411040fa533bd75a1cfd2611d7589605268ae111b3ba1472bc",
    candidate_blake3 = "c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65",
    license = "AGPL-3.0-or-later",
    cargo_revision = "81b260da2256147ec82cb5dd44c0a0c1dcb7500d",
    nix_revision = "81b260da2256147ec82cb5dd44c0a0c1dcb7500d",
    prerequisites = {
      execution_graph_baseline = true,
      nickel_evaluator_baseline = true,
    },
    evidence = {
      single_step_proof = true,
      trace_replay_proof = true,
      trusted_surface = true,
      kamacite_artifacts = true,
      nickel_boundary = true,
      lattice_conformance = true,
      assumptions = true,
      non_claims = true,
    },
    forbidden_fallbacks = ["floating", "github", "sibling", "vendor", "runtime"],
    non_claims = [
      "verifier-soundness",
      "whole-lattice-correctness",
      "runtime-effect-safety",
      "persistence-correctness",
      "release-eligibility",
    ],
  },
}
