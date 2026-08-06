type Predicate = fn(&super::Descriptor) -> bool;

struct Rule {
    predicate: Predicate,
    reason: super::RiskReason,
}

const REJECTION_RULES: &[Rule] = &[
    Rule {
        predicate: claims_runtime_execution,
        reason: super::RiskReason::RuntimeExecutionClaim,
    },
    Rule {
        predicate: has_broad_domain,
        reason: super::RiskReason::BroadDomain,
    },
    Rule {
        predicate: has_ambient_host_control,
        reason: super::RiskReason::AmbientHostControl,
    },
    Rule {
        predicate: has_unadmitted_direct_chat,
        reason: super::RiskReason::DirectMessageInjection,
    },
    Rule {
        predicate: has_unadmitted_sensitive_permission,
        reason: super::RiskReason::BroadPermission,
    },
    Rule {
        predicate: has_unbounded_callback,
        reason: super::RiskReason::UnboundedCallback,
    },
];

const POLICY_RULES: &[Rule] = &[
    Rule {
        predicate: needs_remote_domain_policy,
        reason: super::RiskReason::RemoteDomainRequiresAdmission,
    },
    Rule {
        predicate: needs_browser_permission_policy,
        reason: super::RiskReason::BrowserPermissionRequiresAdmission,
    },
    Rule {
        predicate: needs_host_callback_policy,
        reason: super::RiskReason::HostCallbackRequiresAdmission,
    },
];

pub(super) fn descriptor(descriptor: &super::Descriptor) -> super::matrix::Decision {
    assert!(!descriptor.resource_uri.is_empty(), "validated descriptor keeps resource identity");
    assert!(!descriptor.mime_profile.is_empty(), "validated descriptor keeps MIME profile");
    if let Some(risk_reason) = first_reason(descriptor, REJECTION_RULES) {
        return decision(super::AdoptionDecision::RejectDefault, risk_reason);
    }
    if let Some(risk_reason) = first_reason(descriptor, POLICY_RULES) {
        return decision(super::AdoptionDecision::NeedsHostPolicy, risk_reason);
    }
    decision(super::AdoptionDecision::DataOnlyAdoptable, super::RiskReason::SafeMetadata)
}

fn first_reason(descriptor: &super::Descriptor, rules: &[Rule]) -> Option<super::RiskReason> {
    for rule in rules {
        if (rule.predicate)(descriptor) {
            return Some(rule.reason);
        }
    }
    None
}

fn decision(adoption_decision: super::AdoptionDecision, risk_reason: super::RiskReason) -> super::matrix::Decision {
    super::matrix::Decision {
        adoption_decision,
        risk_reason,
    }
}

fn claims_runtime_execution(descriptor: &super::Descriptor) -> bool {
    descriptor.claims_runtime_execution
}

fn has_broad_domain(descriptor: &super::Descriptor) -> bool {
    descriptor.csp_domains.iter().any(|domain| broad_domain(domain))
}

fn has_ambient_host_control(descriptor: &super::Descriptor) -> bool {
    descriptor.host_context_sync.iter().any(|context| ambient_host_control(context))
}

fn has_unadmitted_direct_chat(descriptor: &super::Descriptor) -> bool {
    descriptor.callback_channels.iter().any(|callback| direct_chat_callback(callback))
        && !descriptor.policy_admission.direct_chat_messages.is_admitted()
}

fn has_unadmitted_sensitive_permission(descriptor: &super::Descriptor) -> bool {
    descriptor.permission_requests.iter().any(|permission| sensitive_permission(permission))
        && !descriptor.policy_admission.browser_permissions.is_admitted()
}

fn has_unbounded_callback(descriptor: &super::Descriptor) -> bool {
    !descriptor.callback_channels.is_empty()
        && descriptor.lifecycle_policy == super::LifecyclePolicy::UnboundedCallbacks
}

fn needs_remote_domain_policy(descriptor: &super::Descriptor) -> bool {
    !descriptor.csp_domains.is_empty() && !descriptor.policy_admission.remote_domains.is_admitted()
}

fn needs_browser_permission_policy(descriptor: &super::Descriptor) -> bool {
    !descriptor.permission_requests.is_empty() && !descriptor.policy_admission.browser_permissions.is_admitted()
}

fn needs_host_callback_policy(descriptor: &super::Descriptor) -> bool {
    !descriptor.callback_channels.is_empty() && !descriptor.policy_admission.host_callbacks.is_admitted()
}

fn broad_domain(domain: &str) -> bool {
    let lower = domain.to_ascii_lowercase();
    domain.contains(super::WILDCARD_DOMAIN_MARKER) || lower.starts_with(super::INSECURE_HTTP_PREFIX)
}

fn sensitive_permission(permission: &str) -> bool {
    matches!(
        permission,
        super::CAMERA_PERMISSION
            | super::MICROPHONE_PERMISSION
            | super::GEOLOCATION_PERMISSION
            | super::CLIPBOARD_WRITE_PERMISSION
            | super::SCREEN_CAPTURE_PERMISSION
    )
}

fn direct_chat_callback(callback: &str) -> bool {
    matches!(callback, super::DIRECT_CHAT_CALLBACK | super::APPEND_MODEL_MESSAGE_CALLBACK)
}

fn ambient_host_control(context: &str) -> bool {
    matches!(context, super::AMBIENT_HOST_CONTROL_CONTEXT | super::HOST_COMMAND_CONTEXT | super::SHELL_CONTEXT)
}
