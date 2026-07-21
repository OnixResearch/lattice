use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

const MISSING_SET_DOMAIN: &str = "lattice.traceability-missing-set.v1";
const REVIEW_POLICY_DOMAIN: &str = "lattice.traceability-inventory-policy.v1";
const INVENTORY_DOMAIN: &str = "lattice.traceability-debt-inventory.v1";
const EXPECTED_SCHEMA_VERSION: &str = "lattice.traceability-debt-inventory.v1";
const EXPECTED_HASH_ALGORITHM: &str = "BLAKE3";
const HEX_HASH_LENGTH: usize = 64;
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CoverageReceipt {
    pub profile_id: String,
    pub requirements: usize,
    pub referenced: usize,
    pub missing: Vec<String>,
    pub dangling: Vec<String>,
    pub policy_hash: String,
    pub input_hash: String,
    pub receipt_hash: String,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewPolicy {
    pub schema_version: String,
    pub hash_algorithm: String,
    pub expected_missing_count: usize,
    pub expected_missing_hash: String,
    pub reviewed: Vec<ReviewDecision>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub requirement_id: String,
    pub classification: Classification,
    pub evidence: Vec<EvidenceReference>,
    pub rationale_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    ExactMaintainedEvidence,
    PartialEvidenceOnly,
    NoExactEvidenceIdentified,
    Unassessed,
}

impl Classification {
    fn as_str(self) -> &'static str {
        match self {
            Self::ExactMaintainedEvidence => "exact_maintained_evidence",
            Self::PartialEvidenceOnly => "partial_evidence_only",
            Self::NoExactEvidenceIdentified => "no_exact_evidence_identified",
            Self::Unassessed => "unassessed",
        }
    }

    fn requires_evidence(self) -> bool {
        matches!(self, Self::ExactMaintainedEvidence | Self::PartialEvidenceOnly)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InventoryIssue {
    pub code: String,
    pub subject: String,
}

impl InventoryIssue {
    fn new(code: &str, subject: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            subject: subject.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub profile_id: String,
    pub requirements: usize,
    pub referenced: usize,
    pub missing: usize,
    pub dangling: usize,
    pub valid: bool,
    pub policy_hash: String,
    pub input_hash: String,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewPolicySummary {
    pub expected_missing_count: usize,
    pub expected_missing_hash: String,
    pub reviewed_decisions: usize,
    pub policy_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryRow {
    pub requirement_id: String,
    pub subsystem: String,
    pub classification: Classification,
    pub evidence: Vec<EvidenceReference>,
    pub rationale_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySummary {
    pub total: usize,
    pub reviewed: usize,
    pub unassessed: usize,
    pub by_classification: BTreeMap<String, usize>,
    pub by_subsystem: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryReport {
    pub schema_version: String,
    pub hash_algorithm: String,
    pub coverage: CoverageSummary,
    pub review_policy: ReviewPolicySummary,
    pub summary: InventorySummary,
    pub rows: Vec<InventoryRow>,
    pub missing_set_hash: String,
    pub non_claims: Vec<String>,
    pub issues: Vec<InventoryIssue>,
    pub valid: bool,
    pub inventory_hash: String,
}

#[derive(Serialize)]
struct InventoryHashPayload<'a> {
    schema_version: &'a str,
    hash_algorithm: &'a str,
    coverage: &'a CoverageSummary,
    review_policy: &'a ReviewPolicySummary,
    summary: &'a InventorySummary,
    rows: &'a [InventoryRow],
    missing_set_hash: &'a str,
    non_claims: &'a [String],
    issues: &'a [InventoryIssue],
    valid: bool,
}

// r[impl lattice.repository_validation.traceability_debt_inventory]
pub fn build_inventory(
    coverage: &CoverageReceipt,
    policy: &ReviewPolicy,
    observed_evidence: &BTreeMap<String, String>,
) -> InventoryReport {
    let mut issues = validate_inputs(coverage, policy);
    let normalized_policy = normalized_policy(policy);
    let policy_hash = review_policy_hash(&normalized_policy);
    let mut missing = coverage.missing.clone();
    missing.sort();
    missing.dedup();
    let decisions = reviewed_decisions(&normalized_policy, &missing, observed_evidence, &mut issues);
    let rows = missing
        .iter()
        .map(|requirement_id| inventory_row(requirement_id, decisions.get(requirement_id)))
        .collect::<Vec<_>>();
    let summary = summarize_rows(&rows);
    sort_issues(&mut issues);
    let missing_hash = missing_set_hash(&missing);
    let valid = issues.is_empty();
    let coverage_summary = CoverageSummary {
        profile_id: coverage.profile_id.clone(),
        requirements: coverage.requirements,
        referenced: coverage.referenced,
        missing: coverage.missing.len(),
        dangling: coverage.dangling.len(),
        valid: coverage.valid,
        policy_hash: coverage.policy_hash.clone(),
        input_hash: coverage.input_hash.clone(),
        receipt_hash: coverage.receipt_hash.clone(),
    };
    let policy_summary = ReviewPolicySummary {
        expected_missing_count: policy.expected_missing_count,
        expected_missing_hash: policy.expected_missing_hash.clone(),
        reviewed_decisions: policy.reviewed.len(),
        policy_hash,
    };
    let mut report = InventoryReport {
        schema_version: EXPECTED_SCHEMA_VERSION.to_string(),
        hash_algorithm: EXPECTED_HASH_ALGORITHM.to_string(),
        coverage: coverage_summary,
        review_policy: policy_summary,
        summary,
        rows,
        missing_set_hash: missing_hash,
        non_claims: normalized_policy.non_claims.clone(),
        issues,
        valid,
        inventory_hash: String::new(),
    };
    report.inventory_hash = inventory_hash(&report);
    report
}

pub fn missing_set_hash(ids: &[String]) -> String {
    let mut sorted = ids.to_vec();
    sorted.sort();
    let mut hasher = blake3::Hasher::new();
    update_field(&mut hasher, MISSING_SET_DOMAIN.as_bytes());
    for id in sorted {
        update_field(&mut hasher, id.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub fn expected_evidence(policy: &ReviewPolicy) -> Result<Vec<EvidenceReference>, Vec<InventoryIssue>> {
    let mut issues = Vec::new();
    let mut references = BTreeMap::new();
    for decision in &policy.reviewed {
        for evidence in &decision.evidence {
            validate_evidence_reference(evidence, "review", &mut issues);
            if let Some(previous_hash) = references.insert(evidence.path.clone(), evidence.blake3.clone())
                && previous_hash != evidence.blake3
            {
                issues.push(InventoryIssue::new("review.evidence_conflict", &evidence.path));
            }
        }
    }
    sort_issues(&mut issues);
    if !issues.is_empty() {
        return Err(issues);
    }
    Ok(references.into_iter().map(|(path, blake3)| EvidenceReference { path, blake3 }).collect())
}

pub fn verify_report(report: &InventoryReport) -> Vec<InventoryIssue> {
    let mut issues = Vec::new();
    if report.schema_version != EXPECTED_SCHEMA_VERSION {
        issues.push(InventoryIssue::new("report.schema_version", &report.schema_version));
    }
    if report.hash_algorithm != EXPECTED_HASH_ALGORITHM {
        issues.push(InventoryIssue::new("report.hash_algorithm", &report.hash_algorithm));
    }
    if !report.issues.is_empty() || !report.valid {
        issues.push(InventoryIssue::new("report.embedded_invalid", report.issues.len().to_string()));
    }
    let row_ids = report.rows.iter().map(|row| row.requirement_id.clone()).collect::<Vec<_>>();
    let mut sorted_ids = row_ids.clone();
    sorted_ids.sort();
    let unique_ids = sorted_ids.iter().collect::<BTreeSet<_>>();
    if row_ids != sorted_ids {
        issues.push(InventoryIssue::new("report.rows_unsorted", "rows"));
    }
    if unique_ids.len() != row_ids.len() {
        issues.push(InventoryIssue::new("report.rows_duplicate", "rows"));
    }
    if report.coverage.missing != row_ids.len() {
        issues.push(InventoryIssue::new("report.missing_count", report.coverage.missing.to_string()));
    }
    let computed_missing_hash = missing_set_hash(&row_ids);
    if report.missing_set_hash != computed_missing_hash {
        issues.push(InventoryIssue::new("report.missing_hash", &report.missing_set_hash));
    }
    if report.review_policy.expected_missing_hash != report.missing_set_hash {
        issues.push(InventoryIssue::new("report.policy_missing_hash", &report.review_policy.expected_missing_hash));
    }
    if report.review_policy.expected_missing_count != row_ids.len() {
        issues.push(InventoryIssue::new(
            "report.policy_missing_count",
            report.review_policy.expected_missing_count.to_string(),
        ));
    }
    let expected_summary = summarize_rows(&report.rows);
    if report.summary != expected_summary {
        issues.push(InventoryIssue::new("report.summary", "summary"));
    }
    for row in &report.rows {
        validate_row(row, &mut issues);
    }
    validate_non_claims(&report.non_claims, &mut issues);
    let expected_inventory_hash = inventory_hash(report);
    if report.inventory_hash != expected_inventory_hash {
        issues.push(InventoryIssue::new("report.inventory_hash", &report.inventory_hash));
    }
    sort_issues(&mut issues);
    issues
}

fn validate_inputs(coverage: &CoverageReceipt, policy: &ReviewPolicy) -> Vec<InventoryIssue> {
    let mut issues = Vec::new();
    if policy.schema_version != EXPECTED_SCHEMA_VERSION {
        issues.push(InventoryIssue::new("policy.schema_version", &policy.schema_version));
    }
    if policy.hash_algorithm != EXPECTED_HASH_ALGORITHM {
        issues.push(InventoryIssue::new("policy.hash_algorithm", &policy.hash_algorithm));
    }
    if coverage.valid || coverage.missing.is_empty() {
        issues.push(InventoryIssue::new("coverage.expected_missing", coverage.valid.to_string()));
    }
    if !coverage.dangling.is_empty() {
        issues.push(InventoryIssue::new("coverage.dangling", coverage.dangling.len().to_string()));
    }
    for hash in [&coverage.policy_hash, &coverage.input_hash, &coverage.receipt_hash] {
        if !valid_blake3_hex(hash) {
            issues.push(InventoryIssue::new("coverage.hash", hash));
        }
    }
    let mut seen_missing = BTreeSet::new();
    for id in &coverage.missing {
        if !valid_requirement_id(id) {
            issues.push(InventoryIssue::new("coverage.requirement_id", id));
        }
        if !seen_missing.insert(id) {
            issues.push(InventoryIssue::new("coverage.duplicate_missing", id));
        }
    }
    if policy.expected_missing_count != coverage.missing.len() {
        issues.push(InventoryIssue::new("policy.expected_missing_count", policy.expected_missing_count.to_string()));
    }
    let actual_missing_hash = missing_set_hash(&coverage.missing);
    if policy.expected_missing_hash != actual_missing_hash {
        issues.push(InventoryIssue::new("policy.expected_missing_hash", &policy.expected_missing_hash));
    }
    if !valid_blake3_hex(&policy.expected_missing_hash) {
        issues.push(InventoryIssue::new("policy.missing_hash_format", &policy.expected_missing_hash));
    }
    validate_non_claims(&policy.non_claims, &mut issues);
    issues
}

fn reviewed_decisions<'a>(
    policy: &'a ReviewPolicy,
    missing: &[String],
    observed_evidence: &BTreeMap<String, String>,
    issues: &mut Vec<InventoryIssue>,
) -> BTreeMap<String, &'a ReviewDecision> {
    let mut decisions = BTreeMap::new();
    for decision in &policy.reviewed {
        if !valid_requirement_id(&decision.requirement_id) {
            issues.push(InventoryIssue::new("review.requirement_id", &decision.requirement_id));
        }
        if missing.binary_search(&decision.requirement_id).is_err() {
            issues.push(InventoryIssue::new("review.id_not_missing", &decision.requirement_id));
        }
        if decision.classification == Classification::Unassessed {
            issues.push(InventoryIssue::new("review.explicit_unassessed", &decision.requirement_id));
        }
        if decision.rationale_code.is_empty() || !valid_code(&decision.rationale_code) {
            issues.push(InventoryIssue::new("review.rationale_code", &decision.requirement_id));
        }
        if decision.classification.requires_evidence() && decision.evidence.is_empty() {
            issues.push(InventoryIssue::new("review.evidence_missing", &decision.requirement_id));
        }
        if !decision.classification.requires_evidence() && !decision.evidence.is_empty() {
            issues.push(InventoryIssue::new("review.evidence_unexpected", &decision.requirement_id));
        }
        for evidence in &decision.evidence {
            validate_evidence_reference(evidence, "review", issues);
            match observed_evidence.get(&evidence.path) {
                Some(observed_hash) if observed_hash == &evidence.blake3 => {}
                Some(_) => issues.push(InventoryIssue::new("review.evidence_hash", &evidence.path)),
                None => issues.push(InventoryIssue::new("review.evidence_unavailable", &evidence.path)),
            }
        }
        if decisions.insert(decision.requirement_id.clone(), decision).is_some() {
            issues.push(InventoryIssue::new("review.duplicate_id", &decision.requirement_id));
        }
    }
    decisions
}

fn inventory_row(requirement_id: &str, decision: Option<&&ReviewDecision>) -> InventoryRow {
    match decision {
        Some(decision) => InventoryRow {
            requirement_id: requirement_id.to_string(),
            subsystem: subsystem(requirement_id),
            classification: decision.classification,
            evidence: decision.evidence.clone(),
            rationale_code: decision.rationale_code.clone(),
        },
        None => InventoryRow {
            requirement_id: requirement_id.to_string(),
            subsystem: subsystem(requirement_id),
            classification: Classification::Unassessed,
            evidence: Vec::new(),
            rationale_code: "not-reviewed".to_string(),
        },
    }
}

fn summarize_rows(rows: &[InventoryRow]) -> InventorySummary {
    let mut by_classification = BTreeMap::new();
    let mut by_subsystem = BTreeMap::new();
    let mut reviewed = 0_usize;
    for row in rows {
        *by_classification.entry(row.classification.as_str().to_string()).or_insert(0) += 1;
        *by_subsystem.entry(row.subsystem.clone()).or_insert(0) += 1;
        if row.classification != Classification::Unassessed {
            reviewed = reviewed.saturating_add(1);
        }
    }
    InventorySummary {
        total: rows.len(),
        reviewed,
        unassessed: rows.len().saturating_sub(reviewed),
        by_classification,
        by_subsystem,
    }
}

fn validate_row(row: &InventoryRow, issues: &mut Vec<InventoryIssue>) {
    if !valid_requirement_id(&row.requirement_id) {
        issues.push(InventoryIssue::new("report.row_id", &row.requirement_id));
    }
    if row.subsystem != subsystem(&row.requirement_id) {
        issues.push(InventoryIssue::new("report.row_subsystem", &row.requirement_id));
    }
    if row.classification.requires_evidence() && row.evidence.is_empty() {
        issues.push(InventoryIssue::new("report.row_evidence_missing", &row.requirement_id));
    }
    if !row.classification.requires_evidence() && !row.evidence.is_empty() {
        issues.push(InventoryIssue::new("report.row_evidence_unexpected", &row.requirement_id));
    }
    for evidence in &row.evidence {
        validate_evidence_reference(evidence, "report.row", issues);
    }
}

fn validate_evidence_reference(evidence: &EvidenceReference, prefix: &str, issues: &mut Vec<InventoryIssue>) {
    if !safe_relative_path(&evidence.path) {
        issues.push(InventoryIssue::new(&format!("{prefix}.evidence_path"), &evidence.path));
    }
    if !valid_blake3_hex(&evidence.blake3) {
        issues.push(InventoryIssue::new(&format!("{prefix}.evidence_hash_format"), &evidence.path));
    }
}

fn normalized_policy(policy: &ReviewPolicy) -> ReviewPolicy {
    let mut normalized = policy.clone();
    for decision in &mut normalized.reviewed {
        decision.evidence.sort();
    }
    normalized.reviewed.sort_by(|left, right| left.requirement_id.cmp(&right.requirement_id));
    normalized.non_claims.sort();
    normalized
}

fn review_policy_hash(policy: &ReviewPolicy) -> String {
    let bytes = serde_json::to_vec(policy).expect("serializing typed review policy cannot fail");
    domain_hash(REVIEW_POLICY_DOMAIN, &bytes)
}

fn inventory_hash(report: &InventoryReport) -> String {
    let payload = InventoryHashPayload {
        schema_version: &report.schema_version,
        hash_algorithm: &report.hash_algorithm,
        coverage: &report.coverage,
        review_policy: &report.review_policy,
        summary: &report.summary,
        rows: &report.rows,
        missing_set_hash: &report.missing_set_hash,
        non_claims: &report.non_claims,
        issues: &report.issues,
        valid: report.valid,
    };
    let bytes = serde_json::to_vec(&payload).expect("serializing typed inventory cannot fail");
    domain_hash(INVENTORY_DOMAIN, &bytes)
}

fn domain_hash(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_field(&mut hasher, domain.as_bytes());
    update_field(&mut hasher, bytes);
    hasher.finalize().to_hex().to_string()
}

fn update_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).expect("inventory field length must fit in u64");
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

fn subsystem(requirement_id: &str) -> String {
    requirement_id.split('.').nth(1).unwrap_or("invalid").replace('_', "-")
}

fn valid_requirement_id(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
}

fn valid_code(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '.' | '-')
        })
}

fn safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') {
        return false;
    }
    let path = Path::new(value);
    !path.is_absolute()
        && !value.contains("//")
        && !value.starts_with("./")
        && !value.ends_with('/')
        && path.components().all(|component| matches!(component, Component::Normal(_)))
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '/' | '-'))
}

fn valid_blake3_hex(value: &str) -> bool {
    value.len() == HEX_HASH_LENGTH
        && value.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}

fn validate_non_claims(non_claims: &[String], issues: &mut Vec<InventoryIssue>) {
    for required in REQUIRED_NON_CLAIMS {
        if !non_claims.iter().any(|value| value == required) {
            issues.push(InventoryIssue::new("policy.non_claim", *required));
        }
    }
}

fn sort_issues(issues: &mut Vec<InventoryIssue>) {
    issues.sort();
    issues.dedup();
}
