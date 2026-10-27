let Contracts = import "../source-contracts.ncl" in
{
  source = fun rid url revision version upstream checksum backend non_claims =>
    {
      schema = "basalt.oidc-exchange.sources.v1",
      bounded_http = { rid = rid, url = url, revision = revision },
      jsonwebtoken = {
        version = version,
        upstream_revision = upstream,
        crate_checksum = checksum,
        backend = backend,
      },
      non_claims = non_claims,
    } | Contracts.Source,
  identity = Contracts.identity,
  non_claims = Contracts.non_claims,
}
