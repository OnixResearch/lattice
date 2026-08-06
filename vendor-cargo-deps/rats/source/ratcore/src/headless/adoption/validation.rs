const MAX_INVENTORY_ROWS: usize = 64;
const MAX_CAPABILITIES_PER_ROW: usize = 16;
const MAX_INVENTORY_TEXT_BYTES: usize = 160;
const MAX_ISSUES_PER_ROW: usize = 70;

/// Validates bounded component-family ownership and capability evidence.
#[must_use]
pub fn validate_headless_inventory(
    rows: &[super::model::HeadlessComponentInventoryRow],
) -> Vec<super::model::HeadlessInventoryIssue> {
    if rows.len() > MAX_INVENTORY_ROWS {
        return vec![issue(
            super::model::HeadlessInventoryIssueCode::InventoryTooLarge,
            "inventory",
        )];
    }
    let issue_slots = rows.len().saturating_mul(MAX_ISSUES_PER_ROW);
    let mut issues = Vec::with_capacity(issue_slots);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        issues.extend(validate_row(row));
        let key = (row.component_id.as_str(), row.owner.as_str());
        if !seen.insert(key) {
            issues.push(issue(
                super::model::HeadlessInventoryIssueCode::DuplicateRow,
                &format!("{}:{}", row.component_id, row.owner),
            ));
        }
    }
    issues.sort();
    issues.dedup();
    debug_assert!(issues.len() <= issue_slots);
    debug_assert!(issues.iter().all(|entry| entry.subject.chars().count() <= MAX_INVENTORY_TEXT_BYTES));
    issues
}

fn validate_row(row: &super::model::HeadlessComponentInventoryRow) -> Vec<super::model::HeadlessInventoryIssue> {
    let mut issues = Vec::with_capacity(MAX_ISSUES_PER_ROW);
    if !valid_text(&row.component_id) || !valid_text(&row.owner) {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::InvalidIdentity, &row.component_id));
    }
    let Some(classification) = super::model::HeadlessComponentClassification::parse(&row.classification) else {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::UnknownClassification, &row.component_id));
        return issues;
    };
    if matches!(
        classification,
        super::model::HeadlessComponentClassification::RendererOwned
            | super::model::HeadlessComponentClassification::IntentionallyDivergent
    ) && blank(row.reason.as_ref())
    {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::OwnershipReasonMissing, &row.component_id));
    }
    if matches!(
        classification,
        super::model::HeadlessComponentClassification::Candidate
            | super::model::HeadlessComponentClassification::Blocked
    ) && blank(row.next_action.as_ref())
    {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::NextActionMissing, &row.component_id));
    }
    issues.extend(validate_capabilities(row, classification));
    debug_assert!(issues.len() <= MAX_ISSUES_PER_ROW);
    debug_assert!(issues.iter().all(|entry| entry.subject.chars().count() <= MAX_INVENTORY_TEXT_BYTES));
    issues
}

fn validate_capabilities(
    row: &super::model::HeadlessComponentInventoryRow,
    classification: super::model::HeadlessComponentClassification,
) -> Vec<super::model::HeadlessInventoryIssue> {
    let mut issues = Vec::with_capacity(MAX_ISSUES_PER_ROW);
    if row.capabilities.len() > MAX_CAPABILITIES_PER_ROW {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::InventoryTooLarge, &row.component_id));
    }
    let mut seen = std::collections::BTreeSet::new();
    for entry in row.capabilities.iter().take(MAX_CAPABILITIES_PER_ROW) {
        if !seen.insert(entry.capability) {
            issues.push(issue(
                super::model::HeadlessInventoryIssueCode::DuplicateCapability,
                &format!("{}:{:?}", row.component_id, entry.capability),
            ));
        }
        let has_evidence = !blank(entry.evidence.as_ref());
        if classification == super::model::HeadlessComponentClassification::Conforming
            && (entry.support != super::model::HeadlessCapabilitySupport::Direct || !has_evidence)
        {
            issues.push(issue(
                super::model::HeadlessInventoryIssueCode::ConformanceEvidenceMissing,
                &format!("{}:{:?}", row.component_id, entry.capability),
            ));
        }
        if entry.support != super::model::HeadlessCapabilitySupport::Direct && blank(entry.reason.as_ref()) {
            issues.push(issue(
                super::model::HeadlessInventoryIssueCode::CapabilityReasonMissing,
                &format!("{}:{:?}", row.component_id, entry.capability),
            ));
        }
        if entry.support != super::model::HeadlessCapabilitySupport::Direct && blank(entry.next_action.as_ref()) {
            issues.push(issue(
                super::model::HeadlessInventoryIssueCode::NextActionMissing,
                &format!("{}:{:?}", row.component_id, entry.capability),
            ));
        }
    }
    if classification == super::model::HeadlessComponentClassification::Conforming
        && seen.len() != super::model::HeadlessComponentCapability::all().len()
    {
        issues.push(issue(super::model::HeadlessInventoryIssueCode::ConformanceEvidenceMissing, &row.component_id));
    }
    debug_assert!(issues.len() <= MAX_ISSUES_PER_ROW);
    debug_assert!(seen.len() <= MAX_CAPABILITIES_PER_ROW);
    issues
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_INVENTORY_TEXT_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
}

fn blank(value: Option<&String>) -> bool {
    value.is_none_or(|text| text.trim().is_empty())
}

fn issue(code: super::model::HeadlessInventoryIssueCode, subject: &str) -> super::model::HeadlessInventoryIssue {
    super::model::HeadlessInventoryIssue {
        code,
        subject: subject.chars().take(MAX_INVENTORY_TEXT_BYTES).collect(),
    }
}
