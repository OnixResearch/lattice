let ConstraintClass = String in
let SandboxPolicy = {
  schema | String,
  policy_id | String,
  allowed_constraint_classes | Array ConstraintClass,
  denied_constraint_fixtures | Array ConstraintClass,
} in

{
  sandbox_policy | SandboxPolicy = {
    schema = "animus.sandbox-policy.v1",
    policy_id = "policy:sandbox-standard",
    allowed_constraint_classes = [
      "filesystem",
      "workdir_cow",
      "process",
      "environment",
      "network",
      "determinism",
    ],
    denied_constraint_fixtures = [
      "http",
    ],
  },
}
