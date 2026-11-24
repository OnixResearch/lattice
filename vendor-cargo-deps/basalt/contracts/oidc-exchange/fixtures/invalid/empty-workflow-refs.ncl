let H = import "../helpers.ncl" in
H.with_claims {
  repository_owner = H.default_claims.repository_owner,
  repository_owner_id = H.default_claims.repository_owner_id,
  repository = H.default_claims.repository,
  repository_id = H.default_claims.repository_id,
  job_workflow_refs = [],
  allowed_refs = H.default_claims.allowed_refs,
  environment = H.default_claims.environment,
}
