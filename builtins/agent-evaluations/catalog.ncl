# Authoritative built-in Lattice agent-evaluation catalog.
let Identity = String in
let Subject = {
  repository_identity | Identity,
  workload_revision | Identity,
  fixture_identity | Identity,
  scaffold_identity | Identity,
  initial_state_identity | Identity,
  hidden_material | String,
  mutation_scope | Array String,
} in
let Environment = {
  identity | Identity,
  network | String,
  isolated | Bool,
  target_class | String,
} in
let Budget = {
  trials | Number,
  concurrency | Number,
  max_tokens | Number,
  max_duration_ms | Number,
  max_deployments | Number,
} in
let Gate = {
  id | String,
  implementation_identity | Identity,
  input_identity | Identity,
  evidence_schema | String,
  objective | Bool,
} in
let Rubric = {
  identity | Identity,
  judge_profile_identity | Identity,
  advisory | Bool,
} in
let Reference = {
  identity | Identity,
  source_identity | Identity,
  capture_class | String,
  freshness_required | Bool,
  fresh | Bool,
  agent_controlled | Bool,
} in
let ArtifactPolicy = {
  identity | Identity,
  allowed_roles | Array String,
  allowed_media | Array String,
  max_count | Number,
  max_bytes | Number,
  retention | String,
  private_destination | Bool,
  redaction_required | Bool,
} in
let LivePosture = {
  enabled | Bool,
  operator_actor | optional | String,
  target_namespace | optional | String,
  credential_handles | Array String,
  retention | String,
  cleanup_required | Bool,
  unknown_outcome | String,
} in
let Case = {
  id | String,
  class | String,
  asset_path | String,
  asset_identity | Identity,
  subject | Subject,
  route | String,
  agent_profile_identity | Identity,
  environment | Environment,
  policy_identity | Identity,
  budget | Budget,
  gates | Array Gate,
  rubric | optional | Rubric,
  references | Array Reference,
  artifacts | ArtifactPolicy,
  live | LivePosture,
  non_claims | Array String,
} in
let Catalog = { schema | String, cases | Array Case } in
let repository = "b3:55c41d7bc3f305066b505528dfc22f99fce59b52a00a1b547a2277688bb84d80" in
let local_agent = "b3:68524b63f5eadaf4752fa4b94a4fb4283eddeb876849fd7b40ef1ffafd893177" in
let hard_agent = "b3:7623e052a4c3bd3c2d2bd2c0898c427bfada0c7425e8cf27394e06726362cb40" in
let live_agent = "b3:c69c43a67ae8ce85d65ec16bf340b104668348136bf18e02f3d2e1831291e938" in
let gate_format = "b3:97946d09c05790ad7311092e08c272d477fdb7628ea278c9ca396c84111aef53" in
let gate_test = "b3:2e9605c235ece2248a0d386abfdaf5897665eeebaee688eaf394fa727923dee9" in
let gate_policy = "b3:3515412c39bfb511b58b294eaaa4cddf9aebc64e4535e7ac8b94d5e328812a6f" in
let gate_deploy = "b3:f1ebc4ed6d93e3ac299345694e9ca14fe4f197cd2ad2dec1a1b00ba3b4ecfdcc" in
let shared_non_claims = [
  "agent_correctness_not_proven",
  "verifier_soundness_not_proven",
  "judge_correctness_not_proven",
  "benchmark_superiority_not_proven",
  "deployment_correctness_not_proven",
  "release_eligibility_not_proven",
] in
let no_live = {
  enabled = false,
  credential_handles = [],
  retention = "ephemeral",
  cleanup_required = false,
  unknown_outcome = "retain_unknown",
} in
let local_artifacts = {
  identity = "b3:d4e46365ccf62c0f233c9bbb0c1f491d664a1446c13f6e3a08ec0e09807236f2",
  allowed_roles = ["result", "bounded_log"],
  allowed_media = ["text", "json"],
  max_count = 8,
  max_bytes = 1048576,
  retention = "reviewed_private",
  private_destination = true,
  redaction_required = true,
} in
({
  schema = "lattice.agent-evaluation-catalog.v1",
  cases = [
    {
      id = "hard-local",
      class = "hard",
      asset_path = "cases/hard-local.txt",
      asset_identity = "b3:39aa3898f42f2990ef72ab095bcd6f844a83a8040d9c810480f78194ab9261f6",
      subject = {
        repository_identity = repository,
        workload_revision = "b3:faff2a743ea8c580c3aad50a428ead7cde2c20d66d934bd64d44eda69e7babcb",
        fixture_identity = "b3:4b6d2a703e5ef68859a58eed444979026402132e996cc59f423f276402a10bdc",
        scaffold_identity = "b3:8d2e8eef2936f17ac98cc60f9359842a1fcc025fa28af1366beb1ebebaae14a9",
        initial_state_identity = "b3:29f03a2854c093e745f8b5626aa364d08bc3598d4394c6fcf4d05d114283098e",
        hidden_material = "frozen",
        mutation_scope = ["src", "tests"],
      },
      route = "agent",
      agent_profile_identity = hard_agent,
      environment = {
        identity = "b3:ee0f839929fd0a74c4b91d7e6d39c8e42072c67c4154c4f5813044a03d069bf7",
        network = "isolated",
        isolated = true,
        target_class = "local-isolated",
      },
      policy_identity = "b3:fb5fcb40399346c32fb4d2d75cae7e549fff6c9801ad3aea85ff1746e539e67d",
      budget = { trials = 2, concurrency = 1, max_tokens = 16384, max_duration_ms = 900000, max_deployments = 0 },
      gates = [
        { id = "format", implementation_identity = gate_format, input_identity = "b3:71268773882779615a8ecb762c4dd941d39b31170d9ccb5aafd74906d602311a", evidence_schema = "lattice.eval-gate.v1", objective = true },
        { id = "policy", implementation_identity = gate_policy, input_identity = "b3:71268773882779615a8ecb762c4dd941d39b31170d9ccb5aafd74906d602311a", evidence_schema = "lattice.eval-gate.v1", objective = true },
      ],
      references = [{ identity = "b3:707b691f0a9aec2525032396f6f148726c01ea3f6e1a75cdd8b8e76d35b46b92", source_identity = "b3:0986534ce3d502e1e9b840e824521437f8e710e8926e4cbb21c3c4062930a2d2", capture_class = "frozen-hidden", freshness_required = false, fresh = true, agent_controlled = false }],
      artifacts = {
        identity = "b3:e6c9b8f5e700004021586b5e2e0b5d602f83e8ecc98c14caa2e87198c6cb8f96",
        allowed_roles = ["result", "bounded_log"],
        allowed_media = ["text", "json"],
        max_count = 8,
        max_bytes = 1048576,
        retention = "reviewed_private",
        private_destination = true,
        redaction_required = true,
      },
      live = no_live,
      non_claims = shared_non_claims,
    },
    {
      id = "live-sandbox",
      class = "live",
      asset_path = "cases/live-sandbox.txt",
      asset_identity = "b3:96d8be4cee3d1e55094f4f5cca67b148a77a48c60f121fbc5020f727bd800f1a",
      subject = {
        repository_identity = repository,
        workload_revision = "b3:abce570e76d3c7f2b701678cd79a5195848bbc2ea4ba64c7775f108802b61ebc",
        fixture_identity = "b3:a11957982ad1930351a4d4c8b7938d4f1e0f77af0b9d9ba12127fdac09ef2b08",
        scaffold_identity = "b3:55c8c883c65b9075d58bae3d4532a543b15bd4105001829f213064b3105c1bfb",
        initial_state_identity = "b3:c944fabfd37bf03be603f4395930bf903db144ccd1525d55f2293a7453f72d57",
        hidden_material = "frozen",
        mutation_scope = ["src", "tests"],
      },
      route = "workflow",
      agent_profile_identity = live_agent,
      environment = {
        identity = "b3:2e7524b6916c19af6734c52daa8b8c34bda125598c23850e989701eab0ddce97",
        network = "live",
        isolated = true,
        target_class = "sandbox-live",
      },
      policy_identity = "b3:fb5fcb40399346c32fb4d2d75cae7e549fff6c9801ad3aea85ff1746e539e67d",
      budget = { trials = 1, concurrency = 1, max_tokens = 32768, max_duration_ms = 1200000, max_deployments = 1 },
      gates = [
        { id = "deploy", implementation_identity = gate_deploy, input_identity = "b3:72c394d779924ab95ef73dcba05f3ddd19bafad1211ff858b4eec492dee2cf64", evidence_schema = "lattice.eval-gate.v1", objective = true },
        { id = "policy", implementation_identity = gate_policy, input_identity = "b3:72c394d779924ab95ef73dcba05f3ddd19bafad1211ff858b4eec492dee2cf64", evidence_schema = "lattice.eval-gate.v1", objective = true },
      ],
      references = [{ identity = "b3:391f85bfda94f97220b18ea6ed3ff283a8e035edb7f71cce9350b26c1db1560a", source_identity = "b3:0ffacdbf58c587947b13b1826861443ae18d90a6cff7354f39ff4394a76f0c24", capture_class = "current-sandbox", freshness_required = true, fresh = true, agent_controlled = false }],
      artifacts = {
        identity = "b3:485cea1c347b8f022cfdb4a48d6064620430bd7f0e10460aea5a61972c8806b7",
        allowed_roles = ["result", "bounded_log", "deployment_observation"],
        allowed_media = ["text", "json"],
        max_count = 8,
        max_bytes = 1048576,
        retention = "reviewed_private",
        private_destination = true,
        redaction_required = true,
      },
      live = {
        enabled = true,
        operator_actor = "operator-reviewed",
        target_namespace = "eval-sandbox",
        credential_handles = ["secret-ref-eval-sandbox"],
        retention = "reviewed_private",
        cleanup_required = true,
        unknown_outcome = "cleanup_required",
      },
      non_claims = shared_non_claims,
    },
    {
      id = "skill-local",
      class = "skill",
      asset_path = "cases/skill-local.txt",
      asset_identity = "b3:36ccc4c9457ce94c42eab205c582ca586f84e88ede86d8610c912a0751cbd659",
      subject = {
        repository_identity = repository,
        workload_revision = "b3:9ed67806d62e63eabca28d695c4ff1d7cb06b8c2473f598d1292202f2fac45e5",
        fixture_identity = "b3:9f8f4bd0704d2c2c4a141a59f1964a0260af0c61228652dd17143ca702d7cf30",
        scaffold_identity = "b3:15705f6b1138730bf9c73e0a13884097c305c139d0d7f8f2f7a84d933aa06244",
        initial_state_identity = "b3:8bb3ed198434cb0537769e3d18eea4bb1a47d3a8b807c36df0a7bd65448cb53b",
        hidden_material = "none",
        mutation_scope = ["src", "tests"],
      },
      route = "workflow",
      agent_profile_identity = local_agent,
      environment = { identity = "b3:f805bb4874f37bb6bb53a96f79e90b38ac2035659451b86b645af3c57d9a227e", network = "none", isolated = true, target_class = "local-hermetic" },
      policy_identity = "b3:fb5fcb40399346c32fb4d2d75cae7e549fff6c9801ad3aea85ff1746e539e67d",
      budget = { trials = 2, concurrency = 2, max_tokens = 8192, max_duration_ms = 300000, max_deployments = 0 },
      gates = [{ id = "test", implementation_identity = gate_test, input_identity = "b3:04a5e58c49167dc7a5dea5e5a946bae2a06558420604dc45810ecacb01811a65", evidence_schema = "lattice.eval-gate.v1", objective = true }],
      rubric = { identity = "b3:1b8484165b304e3a51ff4bcbf17f6ee772860b335bb0e699b1505543b7b2f46e", judge_profile_identity = "b3:ac1de2d9a648ce00d43d978c1fbb3149f50799d6ebeca9cbdbfd6b6f796babaa", advisory = true },
      references = [{ identity = "b3:473ec524cf6a6179ac85a2e178ce25b76db8b87c550f723e4f6bad3d8957db0b", source_identity = "b3:a6e18ff334525f4d1cdc153360c8ce83a4b8f8a9efc7f8abcbbaf4ec7b4ae156", capture_class = "frozen-fixture", freshness_required = false, fresh = true, agent_controlled = false }],
      artifacts = local_artifacts,
      live = no_live,
      non_claims = shared_non_claims,
    },
    {
      id = "unit-local",
      class = "unit",
      asset_path = "cases/unit-local.txt",
      asset_identity = "b3:86c7ec6c5930bbf28dba51a098e9313b3ae95ef8920d8a14656b5fe6e77f65a8",
      subject = {
        repository_identity = repository,
        workload_revision = "b3:d56aaa25e6a4deb441c85f6997cadf34d33696ebd69ac65647d2c3c6371ea264",
        fixture_identity = "b3:1ea544872765f48eac961edbc13612df6e91305096468f868d09b4e1dbc5a4c6",
        scaffold_identity = "b3:70eac1709f559f0a9dd0f92bcc9bec62aeaa0ac73ba1b5ffea5a9545870c3e6b",
        initial_state_identity = "b3:6caa9540e84f4281119201d45d7fe02cf7de19894d4deba59cf51a699563ff4b",
        hidden_material = "none",
        mutation_scope = ["src", "tests"],
      },
      route = "agent",
      agent_profile_identity = local_agent,
      environment = { identity = "b3:f805bb4874f37bb6bb53a96f79e90b38ac2035659451b86b645af3c57d9a227e", network = "none", isolated = true, target_class = "local-hermetic" },
      policy_identity = "b3:fb5fcb40399346c32fb4d2d75cae7e549fff6c9801ad3aea85ff1746e539e67d",
      budget = { trials = 2, concurrency = 2, max_tokens = 4096, max_duration_ms = 60000, max_deployments = 0 },
      gates = [
        { id = "format", implementation_identity = gate_format, input_identity = "b3:ebb18cb4b1408826cbac2150021aac0f4ab0296af0079e05023cf54138aa5a93", evidence_schema = "lattice.eval-gate.v1", objective = true },
        { id = "test", implementation_identity = gate_test, input_identity = "b3:ebb18cb4b1408826cbac2150021aac0f4ab0296af0079e05023cf54138aa5a93", evidence_schema = "lattice.eval-gate.v1", objective = true },
      ],
      references = [{ identity = "b3:6455bce5be6abded792f2cda3d01242b51c88b1abcc412d4c5ec2d6c1d62c4bd", source_identity = "b3:02404d63cf8d82fdb75360867299b1e36c64c917a0c948d2538b416f06181da5", capture_class = "frozen-fixture", freshness_required = false, fresh = true, agent_controlled = false }],
      artifacts = local_artifacts,
      live = no_live,
      non_claims = shared_non_claims,
    },
  ],
}) | Catalog
