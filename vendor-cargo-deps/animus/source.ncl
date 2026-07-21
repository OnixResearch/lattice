let Source = {
  repository | String,
  revision | String,
  crates | Array String,
  normalized_workspace_manifest | Bool,
  normalized_crate_manifests | Bool,
  includes_compile_time_assets | Bool,
} in
{
  repository = "ssh://git@github.com/OnixResearch/animus.git",
  revision = "37256d392d75fab519dcc18cff8b3abc2d162bd4",
  crates = [
    "animus-adapter-observability",
    "animus-adapter-safe",
    "animus-adapters",
    "animus-core",
    "animus-router",
    "animus-runtime",
    "animus-steel",
  ],
  normalized_workspace_manifest = true,
  normalized_crate_manifests = true,
  includes_compile_time_assets = true,
} | Source
