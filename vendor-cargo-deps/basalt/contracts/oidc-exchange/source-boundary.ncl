let Contracts = import "source-contracts.ncl" in
({
  schema = "basalt.oidc-exchange.sources.v1",
  bounded_http = {
    rid = Contracts.identity.bounded_http_rid,
    url = Contracts.identity.bounded_http_url,
    revision = Contracts.identity.bounded_http_revision,
  },
  jsonwebtoken = {
    version = Contracts.identity.jsonwebtoken_version,
    upstream_revision = Contracts.identity.jsonwebtoken_revision,
    crate_checksum = Contracts.identity.jsonwebtoken_checksum,
    backend = "rust_crypto",
  },
  non_claims = Contracts.non_claims,
}) | Contracts.Source
