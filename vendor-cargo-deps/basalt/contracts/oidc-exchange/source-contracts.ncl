let Identity = {
  bounded_http_rid = "rad:z4ARxEt93T6gJTrTZpUyKHMAemPiv",
  bounded_http_url = "https://seed.radicle.garden/z4ARxEt93T6gJTrTZpUyKHMAemPiv.git",
  bounded_http_revision = "5abc2b93e0f288aba251a0750ce2d716eb9757fc",
  jsonwebtoken_version = "11.0.0",
  jsonwebtoken_revision = "900010fb1be550ec2cfa8c32038eb154f9ef56d7",
  jsonwebtoken_checksum = "881733cbc631fc9e472e24447ce32a64bedf2da498d6d8570b08edc87de71f65",
} in
let NonClaims = [
  "source identity does not prove cryptographic correctness",
  "source identity does not prove remote availability",
  "dependency review does not transfer issuer or authorization authority",
] in
let exact_fields = fun allowed value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length allowed
  && std.array.all (fun field => std.array.elem field allowed) (std.record.fields value)
in
let valid = fun value =>
  exact_fields ["schema", "bounded_http", "jsonwebtoken", "non_claims"] value
  && value.schema == "basalt.oidc-exchange.sources.v1"
  && exact_fields ["rid", "url", "revision"] value.bounded_http
  && value.bounded_http.rid == Identity.bounded_http_rid
  && value.bounded_http.url == Identity.bounded_http_url
  && value.bounded_http.revision == Identity.bounded_http_revision
  && exact_fields ["version", "upstream_revision", "crate_checksum", "backend"] value.jsonwebtoken
  && value.jsonwebtoken.version == Identity.jsonwebtoken_version
  && value.jsonwebtoken.upstream_revision == Identity.jsonwebtoken_revision
  && value.jsonwebtoken.crate_checksum == Identity.jsonwebtoken_checksum
  && value.jsonwebtoken.backend == "rust_crypto"
  && value.non_claims == NonClaims
in
{
  Source = std.contract.from_predicate valid,
  identity = Identity,
  non_claims = NonClaims,
}
