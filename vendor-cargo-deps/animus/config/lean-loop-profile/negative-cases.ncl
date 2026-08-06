# Invalid Lean-loop profile cases exercised by the focused Rust validator.
{
  cases = [
    { id = "unknown-protocol", field = "protocol.identity", value = "unknown" },
    { id = "unknown-toolchain", field = "toolchain.cohort", value = "lean4-unreviewed" },
    { id = "non-positive-bound", field = "bounds.frame_bytes", value = "0" },
    { id = "inconsistent-bounds", field = "bounds.text_bytes", value = "greater-than-frame" },
    { id = "unknown-decision", field = "allowed_decisions", value = "shell" },
    { id = "ambient-executable-discovery", field = "launch.path_discovery", value = "true" },
    { id = "unrestricted-environment", field = "launch.inherited_environment", value = "true" },
    { id = "implicit-fallback", field = "launch.fallback_policy", value = "steel" },
    { id = "missing-sandbox-posture", field = "launch.sandbox_posture", value = "missing" },
    { id = "stale-source-hash", field = "identity.proof_source_identity", value = "stale" },
    { id = "unsafe-label", field = "toolchain.package", value = "/private/path" },
    { id = "weakened-redaction", field = "redaction.raw_payloads", value = "true" },
    { id = "weakened-non-claims", field = "non_claims", value = "empty" },
  ],
}
