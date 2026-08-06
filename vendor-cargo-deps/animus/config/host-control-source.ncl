# Immutable standalone host-control source and ownership profile.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let ExactRevision = std.contract.from_predicate (fun value => std.is_string value && std.string.length value == 40) in
let Profile = {
  schema | NonEmpty,
  repository | NonEmpty,
  revision | ExactRevision,
  package | NonEmpty,
  semantic_owner | NonEmpty,
  animus_surface | NonEmpty,
  tile_adoption | NonEmpty,
  forbidden_sources | Array NonEmpty,
  retained_authority | Array NonEmpty,
  non_claims | Array NonEmpty,
} in
{
  host_control_source | Profile = {
    schema = "animus.host-control-source/v1",
    repository = "ssh://git@github.com/OnixResearch/host-control.git",
    revision = "e359ccb80a52efb860307278510fa5297ac970ed",
    package = "host-control-core",
    semantic_owner = "OnixResearch/host-control",
    animus_surface = "compatibility-reexport-only",
    tile_adoption = "independent-exact-revision",
    forbidden_sources = ["workspace-relative", "release-local", "branch-only", "tag-only", "vendored-copy", "direct-transitive-fork"],
    retained_authority = ["consumer-policy", "transport", "daemon", "pty", "process", "filesystem", "clock", "ui", "persistence"],
    non_claims = ["not-daemon-correctness", "not-transport-security", "not-pty-preservation", "not-policy-authorization", "not-whole-product-readiness"],
  },
}
