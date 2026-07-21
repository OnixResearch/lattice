let Source = {
  repository | String,
  revision | String,
  crate_path | String,
  normalized_manifest | Bool,
} in
{
  repository = "ssh://git@github.com/OnixResearch/host-control.git",
  revision = "e359ccb80a52efb860307278510fa5297ac970ed",
  crate_path = "crates/host-control-core",
  normalized_manifest = true,
} | Source
