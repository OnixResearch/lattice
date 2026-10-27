use alloc::string::ToString;

pub(super) fn validate(
    draft: &super::PolicyDraft<'_>,
) -> Result<super::Policy, super::super::Error> {
    let fields = Fields::from_draft(draft)?;
    if fields.trust_status == super::TrustStatus::Untrusted {
        return Err(super::super::Error::UntrustedDescriptor {
            trust_scope: fields.trust_scope,
        });
    }
    validate_precedence(draft.precedence)?;
    Ok(fields.into_policy(draft.precedence))
}

struct Fields {
    action: super::super::super::GuardedAction,
    resource_label: super::super::Text,
    outcome: super::Outcome,
    source_identity: super::super::Text,
    trust_scope: super::TrustScope,
    trust_status: super::TrustStatus,
    prompt_summary_policy: super::PromptSummaryPolicy,
}

impl Fields {
    fn from_draft(draft: &super::PolicyDraft<'_>) -> Result<Self, super::super::Error> {
        let action_text = super::super::required_text(super::super::ACTION_FIELD, draft.action)?;
        let outcome_text = super::super::required_text(super::super::OUTCOME_FIELD, draft.outcome)?;
        let scope_text =
            super::super::required_text(super::super::TRUST_SCOPE_FIELD, draft.trust_scope)?;
        let trust_text =
            super::super::required_text(super::super::TRUST_STATUS_FIELD, draft.trust_status)?;
        let prompt_text = super::super::required_text(
            super::super::PROMPT_POLICY_FIELD,
            draft.prompt_summary_policy,
        )?;
        Ok(Self {
            action: action_from_label(action_text.as_str())
                .ok_or_else(|| unsupported(super::super::ACTION_FIELD, action_text))?,
            resource_label: super::super::required_text(
                super::super::RESOURCE_FIELD,
                draft.resource_label,
            )?,
            outcome: outcome_from_label(outcome_text.as_str())
                .ok_or_else(|| unsupported(super::super::OUTCOME_FIELD, outcome_text))?,
            source_identity: super::super::required_text(
                super::super::SOURCE_FIELD,
                draft.source_identity,
            )?,
            trust_scope: trust_scope_from_label(scope_text.as_str())
                .ok_or_else(|| unsupported(super::super::TRUST_SCOPE_FIELD, scope_text))?,
            trust_status: trust_status_from_label(trust_text.as_str())
                .ok_or_else(|| unsupported(super::super::TRUST_STATUS_FIELD, trust_text))?,
            prompt_summary_policy: summary_policy_from_label(prompt_text.as_str())
                .ok_or_else(|| unsupported(super::super::PROMPT_POLICY_FIELD, prompt_text))?,
        })
    }

    fn into_policy(self, precedence: u16) -> super::Policy {
        let precedence_text = precedence.to_string();
        let policy_hash = super::super::hash_parts(&[
            super::super::HASH_SCHEMA,
            self.action.as_str(),
            self.resource_label.as_str(),
            self.outcome.as_str(),
            self.source_identity.as_str(),
            self.trust_scope.as_str(),
            precedence_text.as_str(),
            self.prompt_summary_policy.as_str(),
        ]);
        super::Policy {
            action: self.action,
            resource_label: self.resource_label,
            outcome: self.outcome,
            source_identity: self.source_identity,
            trust_scope: self.trust_scope,
            trust_status: self.trust_status,
            precedence,
            prompt_summary: self.prompt_summary_policy,
            hash: policy_hash,
        }
    }
}

fn unsupported(
    field: crate::validation::FieldName,
    value: super::super::Text,
) -> super::super::Error {
    super::super::Error::UnsupportedValue {
        field: field.as_str(),
        value,
    }
}

fn validate_precedence(precedence: u16) -> Result<(), super::super::Error> {
    if !(super::super::MIN_PRECEDENCE..=super::super::MAX_PRECEDENCE).contains(&precedence) {
        return Err(super::super::Error::PrecedenceOutOfRange {
            min: super::super::MIN_PRECEDENCE,
            max: super::super::MAX_PRECEDENCE,
            actual: precedence,
        });
    }
    Ok(())
}

fn action_from_label(value: &str) -> Option<super::super::super::GuardedAction> {
    match value {
        "model" => Some(super::super::super::GuardedAction::Model),
        "tool" => Some(super::super::super::GuardedAction::Tool),
        "filesystem" => Some(super::super::super::GuardedAction::Filesystem),
        "exec" => Some(super::super::super::GuardedAction::Exec),
        "network" => Some(super::super::super::GuardedAction::Network),
        "steel" => Some(super::super::super::GuardedAction::Steel),
        "agent_role" => Some(super::super::super::GuardedAction::AgentRole),
        _ => None,
    }
}

fn outcome_from_label(value: &str) -> Option<super::Outcome> {
    match value {
        super::super::ALLOW_OUTCOME_LABEL => Some(super::Outcome::Allow),
        super::super::ASK_OUTCOME_LABEL => Some(super::Outcome::Ask),
        super::super::DENY_OUTCOME_LABEL => Some(super::Outcome::Deny),
        _ => None,
    }
}

fn trust_scope_from_label(value: &str) -> Option<super::TrustScope> {
    match value {
        super::super::USER_SCOPE_LABEL => Some(super::TrustScope::User),
        super::super::PROJECT_SCOPE_LABEL => Some(super::TrustScope::Project),
        super::super::SESSION_SCOPE_LABEL => Some(super::TrustScope::Session),
        super::super::HARNESS_SCOPE_LABEL => Some(super::TrustScope::Harness),
        _ => None,
    }
}

fn trust_status_from_label(value: &str) -> Option<super::TrustStatus> {
    match value {
        super::super::TRUSTED_LABEL => Some(super::TrustStatus::Trusted),
        super::super::UNTRUSTED_LABEL => Some(super::TrustStatus::Untrusted),
        _ => None,
    }
}

fn summary_policy_from_label(value: &str) -> Option<super::PromptSummaryPolicy> {
    match value {
        super::super::REDACTED_SUMMARY_POLICY_LABEL => Some(super::PromptSummaryPolicy::Redacted),
        _ => None,
    }
}
