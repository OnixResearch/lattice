// Negative fixture: candidate direct enforcement-internal call site.
fn unsafe_bypass(policy_json: &str) -> bool {
    let raw = std::fs::read_to_string(policy_json).unwrap_or_default();
    parse_internals(&raw)
}
