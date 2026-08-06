//! `ResearchSpec` promotion DAG contracts.
//!
//! This pure core validates group ordering, blocked explanations, and safe
//! receipts. Hosts own benchmark execution, patch application, VCS operations,
//! conflict resolution, archive mutation, scheduling, and persistence.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

const HASH_DOMAIN: &str = "animus.researchspec-promotion-dag.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_SCHEMA: &str = "animus.researchspec-promotion-dag.receipt.v1";
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const EMPTY_COUNT: usize = 0;
const MIN_COUNT: usize = 1;
const UNMET_DEPENDENCY_CODE: &str = "unmet-dependency";
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_DIFF:",
    "RAW_BENCHMARK_OUTPUT:",
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "RAW_MODEL_OUTPUT:",
    "RAW_TOOL_OUTPUT:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_LOG:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionRunStatus {
    Kept,
    Failed,
    Unkept,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionGuardrailStatus {
    Passed,
    Failed,
    Conflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagGroup {
    pub group_id: String,
    pub kept_run_ids: Vec<String>,
    pub touched_file_summaries: Vec<String>,
    pub metric_evidence_hash: String,
    pub patch_hash: String,
    pub run_status: PromotionRunStatus,
    pub hypothesis_resolved: bool,
    pub guardrail_status: PromotionGuardrailStatus,
    pub allow_file_overlap: bool,
    pub stale_run_evidence: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagEdge {
    pub from_group_id: String,
    pub to_group_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDag {
    pub dag_label: String,
    pub groups: Vec<PromotionDagGroup>,
    pub edges: Vec<PromotionDagEdge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PromotionDagIssueKind {
    EmptyField,
    UnsafeLabel,
    InvalidHash,
    DuplicateGroup,
    DuplicateEdge,
    UnknownDependency,
    SelfDependency,
    Cycle,
    OverlappingFiles,
    StaleRunEvidence,
    FailedOrUnkeptRun,
    UnresolvedHypothesis,
    GuardedFileConflict,
    ReceiptLeak,
    InvalidReceipt,
}

impl PromotionDagIssueKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::UnsafeLabel => "unsafe-label",
            Self::InvalidHash => "invalid-hash",
            Self::DuplicateGroup => "duplicate-group",
            Self::DuplicateEdge => "duplicate-edge",
            Self::UnknownDependency => "unknown-dependency",
            Self::SelfDependency => "self-dependency",
            Self::Cycle => "cycle",
            Self::OverlappingFiles => "overlapping-files",
            Self::StaleRunEvidence => "stale-run-evidence",
            Self::FailedOrUnkeptRun => "failed-or-unkept-run",
            Self::UnresolvedHypothesis => "unresolved-hypothesis",
            Self::GuardedFileConflict => "guarded-file-conflict",
            Self::ReceiptLeak => "receipt-leak",
            Self::InvalidReceipt => "invalid-receipt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagIssue {
    pub kind: PromotionDagIssueKind,
    pub subject: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagValidationReport {
    pub valid: bool,
    pub graph_hash: String,
    pub issues: Vec<PromotionDagIssue>,
    pub issue_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionBatch {
    pub batch_index: usize,
    pub group_ids: Vec<String>,
    pub batch_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockedPromotionGroup {
    pub group_id: String,
    pub unmet_dependencies: Vec<String>,
    pub issue_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagPlan {
    pub batches: Vec<PromotionBatch>,
    pub blocked_groups: Vec<BlockedPromotionGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDagReceipt {
    pub schema: String,
    pub dag_label: String,
    pub graph_hash: String,
    pub group_ids: Vec<String>,
    pub edge_ids: Vec<String>,
    pub issue_codes: Vec<String>,
    pub metric_evidence_hashes: Vec<String>,
    pub touched_file_summary_hashes: Vec<String>,
    pub patch_hashes: Vec<String>,
    pub redacted: bool,
    pub receipt_hash: String,
    pub leak_marker: Option<String>,
}

pub type PromotionDagResult<T> = Result<T, Vec<PromotionDagIssue>>;

pub fn validate_promotion_dag(dag: &PromotionDag) -> PromotionDagValidationReport {
    let mut issues = Vec::new();
    validate_label(dag.dag_label.as_str(), &mut issues);
    let group_ids = collect_group_ids(dag, &mut issues);
    validate_groups(dag, &mut issues);
    validate_edges(dag, &group_ids, &mut issues);
    collect_file_overlaps(dag, &mut issues);
    collect_cycles(dag, &mut issues);
    let graph_hash = promotion_dag_hash("graph", canonical_graph(dag).as_bytes());
    let issue_codes = issue_codes(&issues);
    PromotionDagValidationReport {
        valid: issues.is_empty(),
        graph_hash,
        issues,
        issue_codes,
    }
}

pub fn plan_promotion_dag(dag: &PromotionDag) -> PromotionDagResult<PromotionDagPlan> {
    let report = validate_promotion_dag(dag);
    if !report.valid {
        return Err(report.issues);
    }
    let mut completed = BTreeSet::new();
    let mut remaining = dag.groups.iter().map(|group| group.group_id.clone()).collect::<BTreeSet<_>>();
    let incoming = incoming_dependencies(dag);
    let mut batches = Vec::new();
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .filter(|group_id| {
                incoming
                    .get(group_id.as_str())
                    .is_none_or(|dependencies| dependencies.iter().all(|dep| completed.contains(dep)))
            })
            .cloned()
            .collect::<Vec<_>>();
        if ready.is_empty() {
            break;
        }
        let batch_index = batches.len();
        let batch_hash = promotion_dag_hash("batch", ready.join(HASH_SEPARATOR).as_bytes());
        for group_id in &ready {
            remaining.remove(group_id);
            completed.insert(group_id.clone());
        }
        batches.push(PromotionBatch {
            batch_index,
            group_ids: ready,
            batch_hash,
        });
    }
    let blocked_groups = explain_blocked_groups(dag, &completed);
    Ok(PromotionDagPlan {
        batches,
        blocked_groups,
    })
}

pub fn project_promotion_dag_receipt(dag: &PromotionDag, report: &PromotionDagValidationReport) -> PromotionDagReceipt {
    let mut receipt = PromotionDagReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        dag_label: dag.dag_label.clone(),
        graph_hash: report.graph_hash.clone(),
        group_ids: sorted_group_ids(dag),
        edge_ids: sorted_edge_ids(dag),
        issue_codes: report.issue_codes.clone(),
        metric_evidence_hashes: sorted_metric_hashes(dag),
        touched_file_summary_hashes: sorted_file_hashes(dag),
        patch_hashes: sorted_patch_hashes(dag),
        redacted: true,
        receipt_hash: String::new(),
        leak_marker: None,
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    receipt
}

pub fn validate_promotion_dag_receipt(receipt: &PromotionDagReceipt) -> PromotionDagResult<()> {
    let mut issues = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        issues.push(issue(PromotionDagIssueKind::InvalidReceipt, receipt.schema.as_str()));
    }
    validate_label(receipt.dag_label.as_str(), &mut issues);
    validate_hash(receipt.graph_hash.as_str(), &mut issues);
    validate_hash(receipt.receipt_hash.as_str(), &mut issues);
    validate_hashes(&receipt.metric_evidence_hashes, &mut issues);
    validate_hashes(&receipt.touched_file_summary_hashes, &mut issues);
    validate_hashes(&receipt.patch_hashes, &mut issues);
    if !receipt.redacted {
        issues.push(issue(PromotionDagIssueKind::ReceiptLeak, receipt.dag_label.as_str()));
    }
    if let Some(marker) = &receipt.leak_marker {
        if UNSAFE_MARKERS.iter().any(|unsafe_marker| marker.contains(unsafe_marker)) {
            issues.push(issue(PromotionDagIssueKind::ReceiptLeak, receipt.dag_label.as_str()));
        }
    }
    result_from_issues(issues, ())
}

pub fn promotion_dag_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn collect_group_ids(dag: &PromotionDag, issues: &mut Vec<PromotionDagIssue>) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for group in &dag.groups {
        if !ids.insert(group.group_id.clone()) {
            issues.push(issue(PromotionDagIssueKind::DuplicateGroup, group.group_id.as_str()));
        }
    }
    ids
}

fn validate_groups(dag: &PromotionDag, issues: &mut Vec<PromotionDagIssue>) {
    for group in &dag.groups {
        validate_label(group.group_id.as_str(), issues);
        validate_hash(group.metric_evidence_hash.as_str(), issues);
        validate_hash(group.patch_hash.as_str(), issues);
        if group.kept_run_ids.is_empty() {
            issues.push(issue(PromotionDagIssueKind::EmptyField, group.group_id.as_str()));
        }
        for run_id in &group.kept_run_ids {
            validate_label(run_id.as_str(), issues);
        }
        for file in &group.touched_file_summaries {
            validate_label(file.as_str(), issues);
        }
        if group.stale_run_evidence {
            issues.push(issue(PromotionDagIssueKind::StaleRunEvidence, group.group_id.as_str()));
        }
        if group.run_status != PromotionRunStatus::Kept {
            issues.push(issue(PromotionDagIssueKind::FailedOrUnkeptRun, group.group_id.as_str()));
        }
        if !group.hypothesis_resolved {
            issues.push(issue(PromotionDagIssueKind::UnresolvedHypothesis, group.group_id.as_str()));
        }
        if group.guardrail_status != PromotionGuardrailStatus::Passed {
            issues.push(issue(PromotionDagIssueKind::GuardedFileConflict, group.group_id.as_str()));
        }
    }
}

fn validate_edges(dag: &PromotionDag, group_ids: &BTreeSet<String>, issues: &mut Vec<PromotionDagIssue>) {
    let mut seen = BTreeSet::new();
    for edge in &dag.edges {
        validate_label(edge.from_group_id.as_str(), issues);
        validate_label(edge.to_group_id.as_str(), issues);
        let edge_id = edge_id(edge);
        if !seen.insert(edge_id.clone()) {
            issues.push(issue(PromotionDagIssueKind::DuplicateEdge, edge_id.as_str()));
        }
        if edge.from_group_id == edge.to_group_id {
            issues.push(issue(PromotionDagIssueKind::SelfDependency, edge.from_group_id.as_str()));
        }
        if !group_ids.contains(edge.from_group_id.as_str()) || !group_ids.contains(edge.to_group_id.as_str()) {
            issues.push(issue(PromotionDagIssueKind::UnknownDependency, edge_id.as_str()));
        }
    }
}

fn collect_file_overlaps(dag: &PromotionDag, issues: &mut Vec<PromotionDagIssue>) {
    let mut owner_by_file = BTreeMap::new();
    for group in &dag.groups {
        for file in &group.touched_file_summaries {
            if let Some(owner) = owner_by_file.insert(file.clone(), group.group_id.clone()) {
                if !group.allow_file_overlap {
                    issues.push(issue(
                        PromotionDagIssueKind::OverlappingFiles,
                        format!("{owner}{}{}", HASH_SEPARATOR, group.group_id).as_str(),
                    ));
                }
            }
        }
    }
}

fn collect_cycles(dag: &PromotionDag, issues: &mut Vec<PromotionDagIssue>) {
    let incoming = incoming_dependencies(dag);
    for group in sorted_group_ids(dag) {
        let mut visiting = BTreeSet::new();
        if has_cycle(group.as_str(), &incoming, &mut visiting) {
            issues.push(issue(PromotionDagIssueKind::Cycle, group.as_str()));
        }
    }
}

fn has_cycle(group: &str, incoming: &BTreeMap<String, BTreeSet<String>>, visiting: &mut BTreeSet<String>) -> bool {
    if !visiting.insert(group.to_owned()) {
        return true;
    }
    let found = incoming.get(group).is_some_and(|dependencies| {
        dependencies.iter().any(|dependency| has_cycle(dependency.as_str(), incoming, visiting))
    });
    visiting.remove(group);
    found
}

fn incoming_dependencies(dag: &PromotionDag) -> BTreeMap<String, BTreeSet<String>> {
    let mut incoming = BTreeMap::new();
    for edge in &dag.edges {
        incoming
            .entry(edge.to_group_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(edge.from_group_id.clone());
    }
    incoming
}

fn explain_blocked_groups(dag: &PromotionDag, completed: &BTreeSet<String>) -> Vec<BlockedPromotionGroup> {
    let incoming = incoming_dependencies(dag);
    sorted_group_ids(dag)
        .into_iter()
        .filter_map(|group_id| {
            let unmet = incoming
                .get(group_id.as_str())
                .into_iter()
                .flat_map(|dependencies| dependencies.iter())
                .filter(|dependency| !completed.contains(dependency.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            (!unmet.is_empty()).then_some(BlockedPromotionGroup {
                group_id,
                unmet_dependencies: unmet,
                issue_codes: vec![UNMET_DEPENDENCY_CODE.to_owned()],
            })
        })
        .collect()
}

fn sorted_group_ids(dag: &PromotionDag) -> Vec<String> {
    dag.groups.iter().map(|group| group.group_id.clone()).collect::<BTreeSet<_>>().into_iter().collect()
}

fn sorted_edge_ids(dag: &PromotionDag) -> Vec<String> {
    dag.edges.iter().map(edge_id).collect::<BTreeSet<_>>().into_iter().collect()
}

fn sorted_metric_hashes(dag: &PromotionDag) -> Vec<String> {
    dag.groups
        .iter()
        .map(|group| group.metric_evidence_hash.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_file_hashes(dag: &PromotionDag) -> Vec<String> {
    dag.groups
        .iter()
        .flat_map(|group| group.touched_file_summaries.iter())
        .map(|file| promotion_dag_hash("file", file.as_bytes()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_patch_hashes(dag: &PromotionDag) -> Vec<String> {
    dag.groups
        .iter()
        .map(|group| group.patch_hash.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn edge_id(edge: &PromotionDagEdge) -> String {
    format!("{}->{}", edge.from_group_id, edge.to_group_id)
}

fn canonical_graph(dag: &PromotionDag) -> String {
    format!(
        "{}{}{}{}{}",
        dag.dag_label,
        HASH_SEPARATOR,
        sorted_group_ids(dag).join(HASH_SEPARATOR),
        HASH_SEPARATOR,
        sorted_edge_ids(dag).join(HASH_SEPARATOR)
    )
}

fn receipt_hash(receipt: &PromotionDagReceipt) -> String {
    let canonical = format!(
        "{}{}{}{}{}{}{}",
        receipt.schema,
        HASH_SEPARATOR,
        receipt.dag_label,
        HASH_SEPARATOR,
        receipt.graph_hash,
        HASH_SEPARATOR,
        receipt.group_ids.join(HASH_SEPARATOR)
    );
    promotion_dag_hash("receipt", canonical.as_bytes())
}

fn validate_label(value: &str, issues: &mut Vec<PromotionDagIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(issue(PromotionDagIssueKind::EmptyField, value));
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(issue(PromotionDagIssueKind::UnsafeLabel, value));
    }
}

fn validate_hash(value: &str, issues: &mut Vec<PromotionDagIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(issue(PromotionDagIssueKind::InvalidHash, value));
    }
}

fn validate_hashes(values: &[String], issues: &mut Vec<PromotionDagIssue>) {
    for value in values {
        validate_hash(value.as_str(), issues);
    }
}

fn issue(kind: PromotionDagIssueKind, subject: &str) -> PromotionDagIssue {
    PromotionDagIssue {
        kind,
        subject: subject.to_owned(),
    }
}

fn issue_codes(issues: &[PromotionDagIssue]) -> Vec<String> {
    issues
        .iter()
        .map(|issue| issue.kind.code().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn result_from_issues<T>(issues: Vec<PromotionDagIssue>, value: T) -> PromotionDagResult<T> {
    if issues.is_empty() { Ok(value) } else { Err(issues) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAG_LABEL: &str = "researchspec-promotion";
    const GROUP_A: &str = "group-a";
    const GROUP_B: &str = "group-b";
    const GROUP_C: &str = "group-c";
    const RUN_A: &str = "run-a";
    const RUN_B: &str = "run-b";
    const FILE_A: &str = "src/a.rs";
    const FILE_B: &str = "src/b.rs";
    const METRIC_BYTES: &[u8] = b"metric evidence";
    const PATCH_BYTES: &[u8] = b"patch hash";

    #[test]
    fn dependent_groups_validate_plan_batches_and_project_safe_receipt() {
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag]
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.order]
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.receipts]
        let dag = PromotionDag {
            dag_label: DAG_LABEL.to_owned(),
            groups: vec![group(GROUP_A, RUN_A, FILE_A), group(GROUP_B, RUN_B, FILE_B)],
            edges: vec![PromotionDagEdge {
                from_group_id: GROUP_A.to_owned(),
                to_group_id: GROUP_B.to_owned(),
            }],
        };

        let report = validate_promotion_dag(&dag);
        let plan = plan_promotion_dag(&dag).expect("plan");
        let receipt = project_promotion_dag_receipt(&dag, &report);

        assert!(report.valid);
        assert_eq!(plan.batches[0].group_ids, vec![GROUP_A.to_owned()]);
        assert_eq!(plan.batches[1].group_ids, vec![GROUP_B.to_owned()]);
        assert!(validate_promotion_dag_receipt(&receipt).is_ok());
    }

    #[test]
    fn validation_rejects_cycles_unknowns_overlap_stale_failed_and_unresolved_groups() {
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.validation]
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.verification]
        let mut bad = PromotionDag {
            dag_label: DAG_LABEL.to_owned(),
            groups: vec![group(GROUP_A, RUN_A, FILE_A), PromotionDagGroup {
                touched_file_summaries: vec![FILE_A.to_owned()],
                stale_run_evidence: true,
                run_status: PromotionRunStatus::Failed,
                hypothesis_resolved: false,
                guardrail_status: PromotionGuardrailStatus::Conflict,
                ..group(GROUP_B, RUN_B, FILE_A)
            }],
            edges: vec![
                PromotionDagEdge {
                    from_group_id: GROUP_A.to_owned(),
                    to_group_id: GROUP_B.to_owned(),
                },
                PromotionDagEdge {
                    from_group_id: GROUP_B.to_owned(),
                    to_group_id: GROUP_A.to_owned(),
                },
                PromotionDagEdge {
                    from_group_id: GROUP_C.to_owned(),
                    to_group_id: GROUP_A.to_owned(),
                },
            ],
        };
        bad.groups.push(group(GROUP_A, "run-duplicate", "src/c.rs"));

        let report = validate_promotion_dag(&bad);

        assert!(!report.valid);
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::Cycle.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::DuplicateGroup.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::UnknownDependency.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::OverlappingFiles.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::StaleRunEvidence.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::FailedOrUnkeptRun.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::UnresolvedHypothesis.code().to_owned()));
        assert!(report.issue_codes.contains(&PromotionDagIssueKind::GuardedFileConflict.code().to_owned()));
    }

    #[test]
    fn receipt_rejects_raw_diff_and_unredacted_metadata() {
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.receipts.rejects_raw_diff]
        // r[verify agent_harness_runtime.hermes_researchspec.promotion_dag.docs]
        let dag = PromotionDag {
            dag_label: DAG_LABEL.to_owned(),
            groups: vec![group(GROUP_A, RUN_A, FILE_A)],
            edges: Vec::new(),
        };
        let report = validate_promotion_dag(&dag);
        let mut receipt = project_promotion_dag_receipt(&dag, &report);
        receipt.redacted = false;
        receipt.leak_marker = Some("RAW_DIFF:+secret".to_owned());

        let issues = validate_promotion_dag_receipt(&receipt).expect_err("receipt leak");

        assert!(issues.iter().any(|issue| issue.kind == PromotionDagIssueKind::ReceiptLeak));
    }

    fn group(id: &str, run_id: &str, file: &str) -> PromotionDagGroup {
        PromotionDagGroup {
            group_id: id.to_owned(),
            kept_run_ids: vec![run_id.to_owned()],
            touched_file_summaries: vec![file.to_owned()],
            metric_evidence_hash: promotion_dag_hash(id, METRIC_BYTES),
            patch_hash: promotion_dag_hash(id, PATCH_BYTES),
            run_status: PromotionRunStatus::Kept,
            hypothesis_resolved: true,
            guardrail_status: PromotionGuardrailStatus::Passed,
            allow_file_overlap: false,
            stale_run_evidence: false,
        }
    }
}
