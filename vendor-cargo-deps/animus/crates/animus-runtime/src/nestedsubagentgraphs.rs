//! Nested subagent delegation graph contracts.
//!
//! This module validates explicit graph DTOs and computes safe rollups. It does
//! not schedule children, call providers, inject services, poll queues, render UI,
//! mutate workspaces, or persist lifecycle state.

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

use std::collections::{BTreeMap, BTreeSet};

const HASH_DOMAIN: &str = "animus.nested-subagent-graphs.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_SCHEMA: &str = "animus.nested-subagent-graph.receipt.v1";
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const EMPTY_COUNT: usize = 0;
const EXPECTED_ROOT_COUNT: usize = 1;
const MIN_DEPTH: u16 = 1;
const ROOT_DEPTH: u16 = 0;
const DEFAULT_MAX_DEPTH: u16 = 4;
const MIN_BUDGET: u32 = 1;
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_TRANSCRIPT:",
    "RAW_CHILD_LOG:",
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_OUTPUT:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkspaceAuthority {
    ReadOnly,
    WriteIntent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBudget {
    pub context_tokens: u32,
    pub turns: u32,
    pub tool_calls: u32,
    pub time_units: u32,
    pub output_detail_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedSubagentNode {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub depth: u16,
    pub capabilities: BTreeSet<String>,
    pub workspace_scope: String,
    pub workspace_authority: WorkspaceAuthority,
    pub mutation_authority: bool,
    pub model_ceiling: String,
    pub budget: NestedBudget,
    pub terminal_status: NestedTerminalStatus,
    pub safe_output_hash: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedSubagentEdge {
    pub parent_id: String,
    pub child_id: String,
    pub ancestry_proof_hash: String,
    pub host_admission_receipt_hash: Option<String>,
    pub edge_kind: NestedEdgeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedEdgeKind {
    SpawnRequest,
    ReviewDelegation,
    ReadOnlyInvestigation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedTerminalStatus {
    Pending,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedGraphPolicy {
    pub max_depth: u16,
    pub require_host_admission: bool,
    pub host_service_available: bool,
}

impl Default for NestedGraphPolicy {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            require_host_admission: true,
            host_service_available: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedSubagentGraph {
    pub graph_label: String,
    pub nodes: Vec<NestedSubagentNode>,
    pub edges: Vec<NestedSubagentEdge>,
    pub policy: NestedGraphPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NestedGraphIssueKind {
    EmptyField,
    UnsafeText,
    InvalidHash,
    DuplicateNode,
    UnknownNode,
    Cycle,
    StaleLineage,
    ParentChildMismatch,
    OverDepth,
    AmbiguousRoot,
    MissingHostAdmission,
    AuthorityWidening,
    BudgetInflation,
    MissingHostService,
    ReceiptLeak,
}

impl NestedGraphIssueKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::UnsafeText => "unsafe-text",
            Self::InvalidHash => "invalid-hash",
            Self::DuplicateNode => "duplicate-node",
            Self::UnknownNode => "unknown-node",
            Self::Cycle => "cycle",
            Self::StaleLineage => "stale-lineage",
            Self::ParentChildMismatch => "parent-child-mismatch",
            Self::OverDepth => "over-depth",
            Self::AmbiguousRoot => "ambiguous-root",
            Self::MissingHostAdmission => "missing-host-admission",
            Self::AuthorityWidening => "authority-widening",
            Self::BudgetInflation => "budget-inflation",
            Self::MissingHostService => "missing-host-service",
            Self::ReceiptLeak => "receipt-leak",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedGraphIssue {
    pub kind: NestedGraphIssueKind,
    pub subject: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedGraphValidationReport {
    pub valid: bool,
    pub graph_hash: String,
    pub issues: Vec<NestedGraphIssue>,
    pub issue_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedGraphRollup {
    pub graph_hash: String,
    pub terminal_status: NestedTerminalStatus,
    pub failed_paths: Vec<String>,
    pub cancelled_nodes: Vec<String>,
    pub evidence_hashes: Vec<String>,
    pub mutation_intent_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedGraphReceipt {
    pub schema: String,
    pub graph_label: String,
    pub graph_hash: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub issue_codes: Vec<String>,
    pub redacted: bool,
    pub receipt_hash: String,
    pub leak_marker: Option<String>,
}

pub type NestedGraphResult<T> = Result<T, Vec<NestedGraphIssue>>;

pub fn validate_nested_subagent_graph(graph: &NestedSubagentGraph) -> NestedGraphValidationReport {
    let mut issues = Vec::new();
    validate_label(graph.graph_label.as_str(), &mut issues);
    if graph.policy.max_depth < MIN_DEPTH {
        issues.push(issue(
            NestedGraphIssueKind::OverDepth,
            graph.graph_label.as_str(),
        ));
    }
    let node_ids = collect_node_ids(graph, &mut issues);
    validate_nodes(graph, &mut issues);
    validate_edges(graph, &node_ids, &mut issues);
    validate_roots(graph, &mut issues);
    collect_cycles(graph, &mut issues);
    validate_monotonicity(graph, &mut issues);
    let graph_hash = nested_graph_hash("graph", canonical_graph(graph).as_bytes());
    let issue_codes = issue_codes(&issues);
    NestedGraphValidationReport {
        valid: issues.is_empty(),
        graph_hash,
        issues,
        issue_codes,
    }
}

pub fn rollup_nested_subagent_graph(
    graph: &NestedSubagentGraph,
) -> NestedGraphResult<NestedGraphRollup> {
    let report = validate_nested_subagent_graph(graph);
    if !report.valid {
        return Err(report.issues);
    }
    let failed_paths = graph
        .nodes
        .iter()
        .filter(|node| node.terminal_status == NestedTerminalStatus::Failed)
        .map(|node| node_path(graph, node.node_id.as_str()))
        .collect::<Vec<_>>();
    let cancelled_nodes = graph
        .nodes
        .iter()
        .filter(|node| node.terminal_status == NestedTerminalStatus::Cancelled)
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let terminal_status = if !failed_paths.is_empty() {
        NestedTerminalStatus::Failed
    } else if !cancelled_nodes.is_empty() {
        NestedTerminalStatus::Cancelled
    } else if graph
        .nodes
        .iter()
        .all(|node| node.terminal_status == NestedTerminalStatus::Succeeded)
    {
        NestedTerminalStatus::Succeeded
    } else {
        NestedTerminalStatus::Pending
    };
    Ok(NestedGraphRollup {
        graph_hash: report.graph_hash,
        terminal_status,
        failed_paths,
        cancelled_nodes,
        evidence_hashes: graph
            .nodes
            .iter()
            .map(|node| node.safe_output_hash.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        mutation_intent_count: graph
            .nodes
            .iter()
            .filter(|node| node.mutation_authority)
            .count(),
    })
}

pub fn require_nested_graph_host_service(graph: &NestedSubagentGraph) -> NestedGraphResult<()> {
    if graph.policy.host_service_available {
        Ok(())
    } else {
        Err(vec![issue(
            NestedGraphIssueKind::MissingHostService,
            graph.graph_label.as_str(),
        )])
    }
}

pub fn project_nested_graph_receipt(
    graph: &NestedSubagentGraph,
    report: &NestedGraphValidationReport,
) -> NestedGraphReceipt {
    let mut receipt = NestedGraphReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        graph_label: graph.graph_label.clone(),
        graph_hash: report.graph_hash.clone(),
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        issue_codes: report.issue_codes.clone(),
        redacted: true,
        receipt_hash: String::new(),
        leak_marker: None,
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    receipt
}

pub fn validate_nested_graph_receipt(receipt: &NestedGraphReceipt) -> NestedGraphResult<()> {
    let mut issues = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        issues.push(issue(
            NestedGraphIssueKind::EmptyField,
            receipt.schema.as_str(),
        ));
    }
    validate_label(receipt.graph_label.as_str(), &mut issues);
    validate_hash(receipt.graph_hash.as_str(), &mut issues);
    validate_hash(receipt.receipt_hash.as_str(), &mut issues);
    if !receipt.redacted {
        issues.push(issue(
            NestedGraphIssueKind::ReceiptLeak,
            receipt.graph_label.as_str(),
        ));
    }
    if let Some(marker) = &receipt.leak_marker {
        if UNSAFE_MARKERS
            .iter()
            .any(|unsafe_marker| marker.contains(unsafe_marker))
        {
            issues.push(issue(
                NestedGraphIssueKind::ReceiptLeak,
                receipt.graph_label.as_str(),
            ));
        }
    }
    result_from_issues(issues, ())
}

pub fn nested_graph_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn collect_node_ids(
    graph: &NestedSubagentGraph,
    issues: &mut Vec<NestedGraphIssue>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for node in &graph.nodes {
        if !ids.insert(node.node_id.clone()) {
            issues.push(issue(
                NestedGraphIssueKind::DuplicateNode,
                node.node_id.as_str(),
            ));
        }
    }
    ids
}

fn validate_nodes(graph: &NestedSubagentGraph, issues: &mut Vec<NestedGraphIssue>) {
    for node in &graph.nodes {
        validate_label(node.node_id.as_str(), issues);
        if let Some(parent_id) = &node.parent_id {
            validate_label(parent_id.as_str(), issues);
        }
        validate_label(node.workspace_scope.as_str(), issues);
        validate_label(node.model_ceiling.as_str(), issues);
        validate_hash(node.safe_output_hash.as_str(), issues);
        for evidence in &node.evidence_refs {
            validate_label(evidence.as_str(), issues);
        }
        for capability in &node.capabilities {
            validate_label(capability.as_str(), issues);
        }
        if node.depth > graph.policy.max_depth {
            issues.push(issue(
                NestedGraphIssueKind::OverDepth,
                node.node_id.as_str(),
            ));
        }
        validate_budget(&node.budget, node.node_id.as_str(), issues);
    }
}

fn validate_edges(
    graph: &NestedSubagentGraph,
    node_ids: &BTreeSet<String>,
    issues: &mut Vec<NestedGraphIssue>,
) {
    for edge in &graph.edges {
        validate_label(edge.parent_id.as_str(), issues);
        validate_label(edge.child_id.as_str(), issues);
        validate_hash(edge.ancestry_proof_hash.as_str(), issues);
        if let Some(receipt_hash) = &edge.host_admission_receipt_hash {
            validate_hash(receipt_hash.as_str(), issues);
        } else if graph.policy.require_host_admission {
            issues.push(issue(
                NestedGraphIssueKind::MissingHostAdmission,
                edge.child_id.as_str(),
            ));
        }
        if edge.parent_id == edge.child_id {
            issues.push(issue(NestedGraphIssueKind::Cycle, edge.child_id.as_str()));
        }
        if !node_ids.contains(edge.parent_id.as_str()) || !node_ids.contains(edge.child_id.as_str())
        {
            issues.push(issue(
                NestedGraphIssueKind::UnknownNode,
                edge.child_id.as_str(),
            ));
        }
        if let Some(child) = graph
            .nodes
            .iter()
            .find(|node| node.node_id == edge.child_id)
        {
            if child.parent_id.as_deref() != Some(edge.parent_id.as_str()) {
                issues.push(issue(
                    NestedGraphIssueKind::ParentChildMismatch,
                    edge.child_id.as_str(),
                ));
            }
            let expected_depth = graph
                .nodes
                .iter()
                .find(|node| node.node_id == edge.parent_id)
                .map_or(ROOT_DEPTH, |parent| parent.depth.saturating_add(MIN_DEPTH));
            if child.depth != expected_depth {
                issues.push(issue(
                    NestedGraphIssueKind::StaleLineage,
                    edge.child_id.as_str(),
                ));
            }
        }
    }
}

fn validate_roots(graph: &NestedSubagentGraph, issues: &mut Vec<NestedGraphIssue>) {
    let roots = graph
        .nodes
        .iter()
        .filter(|node| node.parent_id.is_none() && node.depth == ROOT_DEPTH)
        .count();
    if roots != EXPECTED_ROOT_COUNT {
        issues.push(issue(
            NestedGraphIssueKind::AmbiguousRoot,
            graph.graph_label.as_str(),
        ));
    }
}

fn collect_cycles(graph: &NestedSubagentGraph, issues: &mut Vec<NestedGraphIssue>) {
    let incoming = incoming_parents(graph);
    for node in sorted_node_ids(graph) {
        let mut visiting = BTreeSet::new();
        if has_cycle(node.as_str(), &incoming, &mut visiting) {
            issues.push(issue(NestedGraphIssueKind::Cycle, node.as_str()));
        }
    }
}

fn validate_monotonicity(graph: &NestedSubagentGraph, issues: &mut Vec<NestedGraphIssue>) {
    let by_id = graph
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        let (Some(parent), Some(child)) = (
            by_id.get(edge.parent_id.as_str()),
            by_id.get(edge.child_id.as_str()),
        ) else {
            continue;
        };
        if !child.capabilities.is_subset(&parent.capabilities) {
            issues.push(issue(
                NestedGraphIssueKind::AuthorityWidening,
                child.node_id.as_str(),
            ));
        }
        if child.workspace_scope != parent.workspace_scope {
            issues.push(issue(
                NestedGraphIssueKind::AuthorityWidening,
                child.node_id.as_str(),
            ));
        }
        if child.workspace_authority > parent.workspace_authority {
            issues.push(issue(
                NestedGraphIssueKind::AuthorityWidening,
                child.node_id.as_str(),
            ));
        }
        if child.mutation_authority && !parent.mutation_authority {
            issues.push(issue(
                NestedGraphIssueKind::AuthorityWidening,
                child.node_id.as_str(),
            ));
        }
        if child.model_ceiling != parent.model_ceiling {
            issues.push(issue(
                NestedGraphIssueKind::AuthorityWidening,
                child.node_id.as_str(),
            ));
        }
        if budget_widens(&parent.budget, &child.budget) {
            issues.push(issue(
                NestedGraphIssueKind::BudgetInflation,
                child.node_id.as_str(),
            ));
        }
    }
}

fn validate_budget(budget: &NestedBudget, subject: &str, issues: &mut Vec<NestedGraphIssue>) {
    if budget.context_tokens < MIN_BUDGET
        || budget.turns < MIN_BUDGET
        || budget.tool_calls < MIN_BUDGET
        || budget.time_units < MIN_BUDGET
        || budget.output_detail_bytes < MIN_BUDGET
    {
        issues.push(issue(NestedGraphIssueKind::BudgetInflation, subject));
    }
}

fn budget_widens(parent: &NestedBudget, child: &NestedBudget) -> bool {
    child.context_tokens > parent.context_tokens
        || child.turns > parent.turns
        || child.tool_calls > parent.tool_calls
        || child.time_units > parent.time_units
        || child.output_detail_bytes > parent.output_detail_bytes
}

fn incoming_parents(graph: &NestedSubagentGraph) -> BTreeMap<String, BTreeSet<String>> {
    let mut incoming = BTreeMap::new();
    for edge in &graph.edges {
        incoming
            .entry(edge.child_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(edge.parent_id.clone());
    }
    incoming
}

fn has_cycle(
    node: &str,
    incoming: &BTreeMap<String, BTreeSet<String>>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if !visiting.insert(node.to_owned()) {
        return true;
    }
    let found = incoming.get(node).is_some_and(|parents| {
        parents
            .iter()
            .any(|parent| has_cycle(parent.as_str(), incoming, visiting))
    });
    visiting.remove(node);
    found
}

fn node_path(graph: &NestedSubagentGraph, node_id: &str) -> String {
    let by_id = graph
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut path = Vec::new();
    let mut cursor = Some(node_id);
    while let Some(current) = cursor {
        path.push(current.to_owned());
        cursor = by_id
            .get(current)
            .and_then(|node| node.parent_id.as_deref());
    }
    path.reverse();
    path.join("/")
}

fn sorted_node_ids(graph: &NestedSubagentGraph) -> Vec<String> {
    graph
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn canonical_graph(graph: &NestedSubagentGraph) -> String {
    let edges = graph
        .edges
        .iter()
        .map(|edge| format!("{}->{}", edge.parent_id, edge.child_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(HASH_SEPARATOR);
    format!(
        "{}{}{}{}{}",
        graph.graph_label,
        HASH_SEPARATOR,
        sorted_node_ids(graph).join(HASH_SEPARATOR),
        HASH_SEPARATOR,
        edges
    )
}

fn receipt_hash(receipt: &NestedGraphReceipt) -> String {
    let canonical = format!(
        "{}{}{}{}{}{}{}{}{}",
        receipt.schema,
        HASH_SEPARATOR,
        receipt.graph_label,
        HASH_SEPARATOR,
        receipt.graph_hash,
        HASH_SEPARATOR,
        receipt.node_count,
        HASH_SEPARATOR,
        receipt.edge_count
    );
    nested_graph_hash("receipt", canonical.as_bytes())
}

fn validate_label(value: &str, issues: &mut Vec<NestedGraphIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(issue(NestedGraphIssueKind::EmptyField, value));
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(issue(NestedGraphIssueKind::UnsafeText, value));
    }
}

fn validate_hash(value: &str, issues: &mut Vec<NestedGraphIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(issue(NestedGraphIssueKind::InvalidHash, value));
    }
}

fn issue(kind: NestedGraphIssueKind, subject: &str) -> NestedGraphIssue {
    NestedGraphIssue {
        kind,
        subject: subject.to_owned(),
    }
}

fn issue_codes(issues: &[NestedGraphIssue]) -> Vec<String> {
    issues
        .iter()
        .map(|issue| issue.kind.code().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn result_from_issues<T>(issues: Vec<NestedGraphIssue>, value: T) -> NestedGraphResult<T> {
    if issues.is_empty() {
        Ok(value)
    } else {
        Err(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAPH_LABEL: &str = "nested-subagents";
    const ROOT: &str = "parent";
    const CHILD: &str = "child";
    const GRANDCHILD: &str = "grandchild";
    const SIBLING: &str = "sibling";
    const WORKSPACE: &str = "workspace-a";
    const MODEL: &str = "model-small";
    const CAP_READ: &str = "context.read";
    const CAP_SEARCH: &str = "search.safe";
    const CAP_MUTATE: &str = "mutation.intent";
    const EVIDENCE: &str = "evidence-ref";
    const OUTPUT_BYTES: &[u8] = b"safe output";
    const RECEIPT_BYTES: &[u8] = b"host admission";
    const CONTEXT_TOKENS: u32 = 100;
    const CHILD_CONTEXT_TOKENS: u32 = 50;
    const TURNS: u32 = 4;
    const CHILD_TURNS: u32 = 2;
    const TOOL_CALLS: u32 = 3;
    const CHILD_TOOL_CALLS: u32 = 1;
    const TIME_UNITS: u32 = 10;
    const CHILD_TIME_UNITS: u32 = 5;
    const OUTPUT_DETAIL: u32 = 400;
    const CHILD_OUTPUT_DETAIL: u32 = 200;

    #[test]
    fn two_level_and_sibling_graphs_validate_and_roll_up_safely() {
        // r[verify agent_harness_runtime.subagent_nested_graphs.graph]
        // r[verify agent_harness_runtime.subagent_nested_graphs.admission]
        // r[verify agent_harness_runtime.subagent_nested_graphs.rollup]
        let graph = graph_with_nodes(vec![
            node(NodeFixture {
                node_id: ROOT,
                parent_id: None,
                depth: ROOT_DEPTH,
                capabilities: caps(&[CAP_READ, CAP_SEARCH]),
                terminal_status: NestedTerminalStatus::Succeeded,
            }),
            node(NodeFixture {
                node_id: CHILD,
                parent_id: Some(ROOT),
                depth: MIN_DEPTH,
                capabilities: caps(&[CAP_READ]),
                terminal_status: NestedTerminalStatus::Succeeded,
            }),
            node(NodeFixture {
                node_id: SIBLING,
                parent_id: Some(ROOT),
                depth: MIN_DEPTH,
                capabilities: caps(&[CAP_SEARCH]),
                terminal_status: NestedTerminalStatus::Failed,
            }),
        ]);
        let report = validate_nested_subagent_graph(&graph);
        let rollup = rollup_nested_subagent_graph(&graph).expect("rollup");
        let receipt = project_nested_graph_receipt(&graph, &report);

        assert!(report.valid);
        assert_eq!(rollup.terminal_status, NestedTerminalStatus::Failed);
        assert_eq!(rollup.failed_paths, vec![format!("{ROOT}/{SIBLING}")]);
        assert!(validate_nested_graph_receipt(&receipt).is_ok());
    }

    #[test]
    fn nested_graph_rejects_cycles_over_depth_authority_widening_and_missing_admission() {
        // r[verify agent_harness_runtime.subagent_nested_graphs.monotonicity]
        // r[verify agent_harness_runtime.subagent_nested_graphs.verification]
        let mut graph = graph_with_nodes(vec![
            node(NodeFixture {
                node_id: ROOT,
                parent_id: Some(GRANDCHILD),
                depth: ROOT_DEPTH,
                capabilities: caps(&[CAP_READ]),
                terminal_status: NestedTerminalStatus::Pending,
            }),
            node(NodeFixture {
                node_id: CHILD,
                parent_id: Some(ROOT),
                depth: MIN_DEPTH,
                capabilities: caps(&[CAP_READ, CAP_MUTATE]),
                terminal_status: NestedTerminalStatus::Pending,
            }),
            node(NodeFixture {
                node_id: GRANDCHILD,
                parent_id: Some(CHILD),
                depth: DEFAULT_MAX_DEPTH + MIN_DEPTH,
                capabilities: caps(&[CAP_READ]),
                terminal_status: NestedTerminalStatus::Pending,
            }),
        ]);
        graph.edges = vec![
            edge(ROOT, CHILD, None),
            edge(
                CHILD,
                GRANDCHILD,
                Some(nested_graph_hash("receipt", RECEIPT_BYTES)),
            ),
            edge(
                GRANDCHILD,
                ROOT,
                Some(nested_graph_hash("receipt", RECEIPT_BYTES)),
            ),
        ];
        graph.nodes[1].mutation_authority = true;
        graph.nodes[1].workspace_authority = WorkspaceAuthority::WriteIntent;
        graph.nodes[1].budget.context_tokens = CONTEXT_TOKENS + CHILD_CONTEXT_TOKENS;

        let report = validate_nested_subagent_graph(&graph);

        assert!(!report.valid);
        assert!(
            report
                .issue_codes
                .contains(&NestedGraphIssueKind::Cycle.code().to_owned())
        );
        assert!(
            report
                .issue_codes
                .contains(&NestedGraphIssueKind::OverDepth.code().to_owned())
        );
        assert!(
            report
                .issue_codes
                .contains(&NestedGraphIssueKind::MissingHostAdmission.code().to_owned())
        );
        assert!(
            report
                .issue_codes
                .contains(&NestedGraphIssueKind::AuthorityWidening.code().to_owned())
        );
        assert!(
            report
                .issue_codes
                .contains(&NestedGraphIssueKind::BudgetInflation.code().to_owned())
        );
    }

    #[test]
    fn host_execution_boundary_and_receipt_leaks_fail_closed() {
        // r[verify agent_harness_runtime.subagent_nested_graphs.host_owned]
        // r[verify agent_harness_runtime.subagent_nested_graphs.verification.closeout]
        let graph = graph_with_nodes(vec![
            node(NodeFixture {
                node_id: ROOT,
                parent_id: None,
                depth: ROOT_DEPTH,
                capabilities: caps(&[CAP_READ]),
                terminal_status: NestedTerminalStatus::Succeeded,
            }),
            node(NodeFixture {
                node_id: CHILD,
                parent_id: Some(ROOT),
                depth: MIN_DEPTH,
                capabilities: caps(&[CAP_READ]),
                terminal_status: NestedTerminalStatus::Cancelled,
            }),
        ]);
        let service = require_nested_graph_host_service(&graph).expect_err("host required");
        assert!(
            service
                .iter()
                .any(|issue| issue.kind == NestedGraphIssueKind::MissingHostService)
        );

        let report = validate_nested_subagent_graph(&graph);
        let mut receipt = project_nested_graph_receipt(&graph, &report);
        receipt.redacted = false;
        receipt.leak_marker = Some("RAW_TRANSCRIPT:secret".to_owned());
        let issues = validate_nested_graph_receipt(&receipt).expect_err("receipt leak");
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == NestedGraphIssueKind::ReceiptLeak)
        );
    }

    fn graph_with_nodes(nodes: Vec<NestedSubagentNode>) -> NestedSubagentGraph {
        let edges = nodes
            .iter()
            .filter_map(|node| {
                node.parent_id.as_ref().map(|parent| {
                    edge(
                        parent.as_str(),
                        node.node_id.as_str(),
                        Some(nested_graph_hash("receipt", RECEIPT_BYTES)),
                    )
                })
            })
            .collect();
        NestedSubagentGraph {
            graph_label: GRAPH_LABEL.to_owned(),
            nodes,
            edges,
            policy: NestedGraphPolicy::default(),
        }
    }

    struct NodeFixture {
        node_id: &'static str,
        parent_id: Option<&'static str>,
        depth: u16,
        capabilities: BTreeSet<String>,
        terminal_status: NestedTerminalStatus,
    }

    fn node(fixture: NodeFixture) -> NestedSubagentNode {
        NestedSubagentNode {
            node_id: fixture.node_id.to_owned(),
            parent_id: fixture.parent_id.map(str::to_owned),
            depth: fixture.depth,
            capabilities: fixture.capabilities,
            workspace_scope: WORKSPACE.to_owned(),
            workspace_authority: WorkspaceAuthority::ReadOnly,
            mutation_authority: false,
            model_ceiling: MODEL.to_owned(),
            budget: budget(fixture.depth),
            terminal_status: fixture.terminal_status,
            safe_output_hash: nested_graph_hash(fixture.node_id, OUTPUT_BYTES),
            evidence_refs: vec![EVIDENCE.to_owned()],
        }
    }

    fn edge(parent: &str, child: &str, receipt: Option<String>) -> NestedSubagentEdge {
        NestedSubagentEdge {
            parent_id: parent.to_owned(),
            child_id: child.to_owned(),
            ancestry_proof_hash: nested_graph_hash(
                "ancestry",
                format!("{parent}->{child}").as_bytes(),
            ),
            host_admission_receipt_hash: receipt,
            edge_kind: NestedEdgeKind::ReviewDelegation,
        }
    }

    fn caps(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    const fn budget(depth: u16) -> NestedBudget {
        if depth == ROOT_DEPTH {
            NestedBudget {
                context_tokens: CONTEXT_TOKENS,
                turns: TURNS,
                tool_calls: TOOL_CALLS,
                time_units: TIME_UNITS,
                output_detail_bytes: OUTPUT_DETAIL,
            }
        } else {
            NestedBudget {
                context_tokens: CHILD_CONTEXT_TOKENS,
                turns: CHILD_TURNS,
                tool_calls: CHILD_TOOL_CALLS,
                time_units: CHILD_TIME_UNITS,
                output_detail_bytes: CHILD_OUTPUT_DETAIL,
            }
        }
    }
}
