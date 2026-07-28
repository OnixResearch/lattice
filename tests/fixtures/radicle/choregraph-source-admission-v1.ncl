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
    revision = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c",
    source_url = "https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git",
    source_archive_blake3 = "dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5",
    release_bundle_blake3 = "a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9",
    candidate_blake3 = "e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a",
    license = "AGPL-3.0-or-later",
    cargo_revision = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c",
    nix_revision = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c",
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
