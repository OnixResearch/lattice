let Upstream = import "upstream-default.ncl" in
let LocalContracts = import "traceability-contracts.ncl" in
# r[impl lattice.repository_validation.traceability_sources]
let LatticeTraceabilityProfile = {
  id = "cairn-default",
  requirement_sources = [
    { root = ".cairn/specs", source_kind = "accepted_specs", extensions = ["md"] },
  ],
  evidence_sources = [
    { root = "src", source_kind = "rust_source", extensions = ["rs"] },
    { root = "crates", source_kind = "rust_source", extensions = ["rs"] },
    { root = "tests", source_kind = "rust_source", extensions = ["rs"] },
    { root = "examples", source_kind = "rust_source", extensions = ["rs"] },
    { root = "examples", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "scripts", source_kind = "tool_source", extensions = ["rs"] },
    { root = "cairn-policy/fixtures", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "cairn-policy/default.ncl", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "cairn-policy/traceability-contracts.ncl", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "Cargo.toml", source_kind = "tool_source", extensions = ["toml"] },
    { root = "flake.nix", source_kind = "tool_source", extensions = ["nix"] },
    { root = "contracts/role-protocol-runtime-profile.ncl", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "profiles/role-protocol-runtime-v1.ncl", source_kind = "nickel_source", extensions = ["ncl"] },
    { root = "docs/choregraph-effect-authority.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/content-identity-source.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/dag-node-memoization.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/expression-dependencies.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/nickel-eval-source.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/role-protocol-execution.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/workflow-domain-boundaries.md", source_kind = "tool_source", extensions = ["md"] },
  ],
  marker_verbs = ["impl", "verify", "depends", "related"],
  assurance_level = "declared",
  anchor_kind = "exact_marker_bytes",
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
  assurance_level = "declared",
  anchor_kind = "exact_marker_bytes",
} in
Upstream & {
  name | force = "lattice-default",
  policy_schema_compatibility | force = Upstream.policy_schema_compatibility & {
    refresh_command | force = "nix run path:$CAIRN_SOURCE#cairn -- policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json",
  },
  # r[impl lattice.repository_validation.traceability_sources.scenario.unsafe-root]
  traceability_policy | LocalContracts.Policy | force = {
    default_profile = "cairn-default",
    profiles = [LatticeTraceabilityProfile, TrellisTraceabilityProfile],
  },
}
