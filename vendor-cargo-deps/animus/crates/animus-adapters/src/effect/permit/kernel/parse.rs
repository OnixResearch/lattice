fn validate_precedence(precedence: u16) -> Result<(), super::Error> {
    if !(super::MIN_PRECEDENCE..=super::MAX_PRECEDENCE).contains(&precedence) {
        return Err(super::Error::PrecedenceOutOfRange {
            min: super::MIN_PRECEDENCE,
            max: super::MAX_PRECEDENCE,
            actual: precedence,
        });
    }
    Ok(())
}

fn unsupported(field: crate::validation::FieldName, value: super::Text) -> super::Error {
    super::Error::UnsupportedValue {
        field: field.as_str(),
        value,
    }
}

fn action_from_label(value: &str) -> Option<super::super::GuardedAction> {
    match value {
        "model" => Some(super::super::GuardedAction::Model),
        "tool" => Some(super::super::GuardedAction::Tool),
        "filesystem" => Some(super::super::GuardedAction::Filesystem),
        "exec" => Some(super::super::GuardedAction::Exec),
        "network" => Some(super::super::GuardedAction::Network),
        "steel" => Some(super::super::GuardedAction::Steel),
        "agent_role" => Some(super::super::GuardedAction::AgentRole),
        _ => None,
    }
}

fn outcome_from_label(value: &str) -> Option<super::Outcome> {
    match value {
        super::ALLOW_OUTCOME_LABEL => Some(super::Outcome::Allow),
        super::ASK_OUTCOME_LABEL => Some(super::Outcome::Ask),
        super::DENY_OUTCOME_LABEL => Some(super::Outcome::Deny),
        _ => None,
    }
}

fn trust_scope_from_label(value: &str) -> Option<super::TrustScope> {
    match value {
        super::USER_SCOPE_LABEL => Some(super::TrustScope::User),
        super::PROJECT_SCOPE_LABEL => Some(super::TrustScope::Project),
        super::SESSION_SCOPE_LABEL => Some(super::TrustScope::Session),
        super::HARNESS_SCOPE_LABEL => Some(super::TrustScope::Harness),
        _ => None,
    }
}

fn trust_status_from_label(value: &str) -> Option<super::TrustStatus> {
    match value {
        super::TRUSTED_LABEL => Some(super::TrustStatus::Trusted),
        super::UNTRUSTED_LABEL => Some(super::TrustStatus::Untrusted),
        _ => None,
    }
}
