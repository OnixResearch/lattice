let H = import "../helpers.ncl" in
H.with_claims {
  repository_owner = H.default_claims.repository_owner,
  repository_owner_id = H.default_claims.repository_owner_id,
  repository = H.default_claims.repository,
  repository_id = H.default_claims.repository_id,
  job_workflow_refs = [
    "OnixResearch/mantle/.github/workflows/build.yml@refs/heads/",
  ],
  allowed_refs = ["refs/heads/"],
  environment = H.default_claims.environment,
}
