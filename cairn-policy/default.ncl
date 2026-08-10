let Upstream = import "upstream-default.ncl" in
let LatticeTraceabilityProfile = {
  id = "cairn-default",
  requirement_sources = [
    { root = ".cairn/specs", source_kind = "accepted_specs", extensions = ["md"] },
  ],
  evidence_sources = [
    { root = "src", source_kind = "rust_source", extensions = ["rs"] },
    { root = "tests", source_kind = "rust_source", extensions = ["rs"] },
    { root = "examples", source_kind = "rust_source", extensions = ["rs"] },
    { root = "examples", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "scripts", source_kind = "tool_source", extensions = ["rs"] },
    { root = "cairn-policy", source_kind = "nickel_source", extensions = ["ncl"] },
  ],
  marker_verbs = ["impl", "verify", "depends", "related"],
} in
let TrellisTraceabilityProfile = {
  id = "trellis-rust",
  requirement_sources = [
    { root = "docs/requirements.md", source_kind = "requirements_doc", extensions = ["md"] },
  ],
  evidence_sources = [
    { root = "src", source_kind = "rust_source", extensions = ["rs"] },
    { root = "tests", source_kind = "rust_source", extensions = ["rs"] },
  ],
  marker_verbs = ["impl", "verify", "depends", "related"],
} in
Upstream & {
  name | force = "lattice-default",
  policy_schema_compatibility | force = Upstream.policy_schema_compatibility & {
    refresh_command | force = "nix run path:$CAIRN_SOURCE#cairn -- policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json",
  },
  traceability_policy | force = {
    default_profile = "cairn-default",
    profiles = [LatticeTraceabilityProfile, TrellisTraceabilityProfile],
  },
}
