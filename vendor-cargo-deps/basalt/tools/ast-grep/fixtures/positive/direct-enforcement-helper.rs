// Positive fixture: enforcement helper use (accepted shaped usage).
use basalt::enforce;
use basalt::EnforcementRequest;
use basalt::parse_policy_json;

fn check(policy_json: &str, request: EnforcementRequest) -> bool {
    let policy = parse_policy_json(policy_json).ok()?;
    enforce(&policy, &request).map_or(false, |r| r.is_allowed())
}
