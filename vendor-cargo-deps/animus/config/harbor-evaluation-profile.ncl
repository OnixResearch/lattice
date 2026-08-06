# Typed local-only Harbor interoperability profile.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Profile = {
  schema | NonEmpty, profile_id | NonEmpty, enabled | Bool,
  harbor | { release | NonEmpty, revision | NonEmpty, task_schema | NonEmpty, atif_schema | NonEmpty, license | NonEmpty },
  task | { identity | NonEmpty, dataset_identity | NonEmpty, network | [| 'none |], remote_inputs | Bool },
  agent | { identity | NonEmpty, model_route_identity | NonEmpty },
  environment | { identity | NonEmpty, backend | [| 'fixture_local |], network | [| 'none |], cpus | Positive, memory_mb | Positive, storage_mb | Positive },
  verifier | { identity | NonEmpty, rubric_identity | NonEmpty, mode | [| 'separate |], network | [| 'none |] },
  budgets | { trials | Positive, concurrency | Positive, atif_steps | Positive, artifacts | Positive },
  secrets | { values_allowed | Bool, declarations | Array NonEmpty },
  rewards | { missing | [| 'error |], invalid | [| 'error |], non_finite | [| 'error |], judge | [| 'advisory |] },
  redaction | [| 'metadata_only |], non_claims | Array NonEmpty,
} in
{
  harbor_evaluation_profile | Profile = {
    schema = "animus.harbor-evaluation-profile.v1", profile_id = "harbor-v0.18-local-atif-v1.7",
    enabled = true,
    harbor = { release = "v0.18.0", revision = "527d50deb63a5d279e8c20593c18a2cbc7f61f9e", task_schema = "1.3", atif_schema = "ATIF-v1.7", license = "Apache-2.0" },
    task = { identity = "task:fixture-local", dataset_identity = "dataset:fixture-local", network = 'none, remote_inputs = false },
    agent = { identity = "agent:fixture", model_route_identity = "model:fixture" },
    environment = { identity = "environment:fixture-local", backend = 'fixture_local, network = 'none, cpus = 1, memory_mb = 512, storage_mb = 1024 },
    verifier = { identity = "verifier:fixture-separate", rubric_identity = "rubric:fixture-v1", mode = 'separate, network = 'none },
    budgets = { trials = 256, concurrency = 16, atif_steps = 4096, artifacts = 128 },
    secrets = { values_allowed = false, declarations = [] },
    rewards = { missing = 'error, invalid = 'error, non_finite = 'error, judge = 'advisory },
    redaction = 'metadata_only,
    non_claims = ["not-verifier-soundness", "not-benchmark-superiority", "not-automatic-completion", "not-harbor-cloud-or-hub-parity", "not-production-or-release-readiness"],
  },
}
