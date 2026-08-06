let Source = {
  repository | String,
  revision | String,
  host_control_revision | String,
  crates | Array String,
  normalized_workspace_manifest | Bool,
  normalized_crate_manifests | Bool,
  host_control_path_dependency | Bool,
  includes_compile_time_assets | Bool,
} in
{
  repository = "ssh://git@github.com/OnixResearch/animus.git",
  revision = "f1662a8ed290b170c3986d2ddf2909b36d7e5fdc",
  host_control_revision = "7735faf3d8895b0bfa89f563184c4321b3b144a1",
  crates = [
    "animus-adapter-observability",
    "animus-adapter-rlm-kernel",
    "animus-adapter-safe",
    "animus-adapters",
    "animus-core",
    "animus-router",
    "animus-runtime",
    "animus-steel",
  ],
  normalized_workspace_manifest = true,
  normalized_crate_manifests = true,
  host_control_path_dependency = true,
  includes_compile_time_assets = true,
} | Source
