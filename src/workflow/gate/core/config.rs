pub fn validate_node_config(config: &crate::workflow::gate::model::ApprovalNodeConfig) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    {
        let mut sink = IssueSink { values: &mut issues };
        if config.safe_summary.trim().is_empty() || config.safe_summary.len() > MAX_SAFE_SUMMARY_BYTES {
            sink.push(issue(ValidationCode::UnsafeSummary, "approval summary must be non-empty and bounded"));
        }
        validate_count_and_unique(
            CollectionValidation {
                values: &config.selected_input_ports,
                count: MAX_SELECTED_INPUTS,
                label: "selected input ports",
            },
            &mut sink,
        );
        validate_count_and_unique(
            CollectionValidation {
                values: &config.required_artifacts,
                count: MAX_REQUIRED_ARTIFACTS,
                label: "required artifacts",
            },
            &mut sink,
        );
        validate_authority(config, &mut sink);
        validate_expiry(config, &mut sink);
        if config.reason.max_bytes == 0 || config.reason.max_bytes > MAX_REASON_POLICY_BYTES {
            sink.push(issue(
                ValidationCode::InvalidReasonPolicy,
                "approval reason policy must use a non-zero bounded byte limit",
            ));
        }
        validate_resume_schema(config, &mut sink);
        validate_non_claims(config, &mut sink);
    }
    debug_assert!(issues.len() <= MAX_NON_CLAIMS);
    debug_assert!(issues.iter().all(|issue| !issue.message.is_empty()));
    issues
}

fn validate_authority(config: &crate::workflow::gate::model::ApprovalNodeConfig, issues: &mut IssueSink<'_>) {
    let initial_issue_count = issues.values.len();
    let is_ability_policy_invalid = [
        config.authority.accepted_abilities.is_empty(),
        config.authority.accepted_abilities.len() > crate::workflow::gate::model::DecisionAbility::VARIANT_COUNT,
        has_duplicates(&config.authority.accepted_abilities),
    ]
    .into_iter()
    .any(std::convert::identity);
    if is_ability_policy_invalid {
        issues.push(issue(
            ValidationCode::MissingAuthority,
            "approval authority must declare unique accepted decision abilities",
        ));
    }
    match config.authority.profile {
        crate::workflow::gate::model::AuthorityProfile::LocalOperator => {
            let is_local_policy_invalid = [
                config.authority.local_actors.is_empty(),
                config.authority.local_actors.len() > MAX_POLICY_ACTORS,
                has_duplicates(&config.authority.local_actors),
                config.authority.delegated_policy_reference.is_some(),
            ]
            .into_iter()
            .any(std::convert::identity);
            if is_local_policy_invalid {
                issues.push(issue(
                    ValidationCode::MissingAuthority,
                    "local approval authority requires unique actors and no delegated policy reference",
                ));
            }
        }
        crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified => {
            let has_valid_reference =
                config.authority.delegated_policy_reference.as_deref().is_some_and(is_valid_reference);
            if !config.authority.local_actors.is_empty() || !has_valid_reference {
                issues.push(issue(
                    ValidationCode::MissingAuthority,
                    "delegated approval authority requires one bounded policy reference and no local actor fallback",
                ));
            }
        }
    }
    debug_assert!(issues.values.len() >= initial_issue_count);
    debug_assert!(issues.values.iter().all(|issue| !issue.message.is_empty()));
}

fn validate_expiry(config: &crate::workflow::gate::model::ApprovalNodeConfig, issues: &mut IssueSink<'_>) {
    let is_valid_ttl = |ttl_millis: u64| (MIN_TTL_MILLIS..=MAX_TTL_MILLIS).contains(&ttl_millis);
    if !is_valid_ttl(config.expiry.decision_ttl_millis) || !is_valid_ttl(config.expiry.resume_ttl_millis) {
        issues.push(issue(
            ValidationCode::InvalidExpiry,
            "approval expiry windows must be non-zero and within the reviewed maximum",
        ));
    }
}

fn validate_non_claims(config: &crate::workflow::gate::model::ApprovalNodeConfig, issues: &mut IssueSink<'_>) {
    let observed = config.required_non_claims.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    let required = crate::workflow::gate::model::REQUIRED_APPROVAL_NON_CLAIMS
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    if config.required_non_claims.len() > MAX_NON_CLAIMS || observed != required {
        issues.push(issue(
            ValidationCode::MissingNonClaim,
            "approval node must declare the exact reviewed non-claim set",
        ));
    }
}

fn validate_resume_schema(config: &crate::workflow::gate::model::ApprovalNodeConfig, issues: &mut IssueSink<'_>) {
    let is_schema_invalid = [
        config.resume.values.len() > MAX_RESUME_FIELDS,
        config.resume.credential_bindings.len() > MAX_RESUME_FIELDS,
        has_duplicates_by(config.resume.values.iter().map(|field| field.name.as_str())),
        has_duplicates(&config.resume.credential_bindings),
        config.resume.values.iter().any(|field| !is_valid_type_tag(&field.type_tag)),
    ]
    .into_iter()
    .any(std::convert::identity);
    if is_schema_invalid {
        issues.push(issue(
            ValidationCode::UnsafeResumeSchema,
            "approval resume schema must be bounded, unique, and use supported non-secret value types",
        ));
    }
}
