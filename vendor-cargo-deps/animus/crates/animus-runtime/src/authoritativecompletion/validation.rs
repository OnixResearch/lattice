// Registry validation reports every bounded input fault instead of asserting on untrusted data; reviewed DTO names and diagnostic sinks preserve compatibility.
#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::borrowed_argument_types,
        tigerstyle::compound_condition,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units
    )
)]
use std::collections::{BTreeMap, BTreeSet};

use super::{
    AGENT_CHECK_REGISTRY_SCHEMA, AgentCheckDiagnostic, AgentCheckDiagnosticCode,
    AgentCheckRegistry, AgentCheckRegistryDraft, AgentCheckRow, AgentCheckRowDraft,
    CAIRN_COMPLETION_SCHEMA, CheckStatus, PINNED_CAIRN_POLICY_HASH,
};

const HASH_HEX_CHARS: usize = 64;
const MAX_CHECKS: usize = 64;
const MAX_ARGS: usize = 32;
const MAX_REFERENCES: usize = 32;
const MAX_TEXT_CHARS: usize = 512;
const SHELL_ESCAPE_ARG_COUNT: usize = 2;
const WORKING_ROOT: &str = "<repository-root>";
const HASH_DOMAIN: &str = "animus.authoritative-completion.v1";
const SAFE_RISK_LABELS: &[&str] = &["low", "medium", "high"];
const SAFE_TIMEOUT_LABELS: &[&str] = &["short", "medium", "long"];
const SAFE_OUTPUT_LABELS: &[&str] = &["small", "medium", "large"];
const DIAGNOSTIC_NON_CLAIMS: &[&str] = &[
    "diagnostic does not prove check-oracle soundness",
    "diagnostic does not authorize retry or mutation",
    "diagnostic omits raw prompts, tool payloads, credentials, and unbounded logs",
];

const _: () = assert!(HASH_HEX_CHARS > 0);
const _: () = assert!(MAX_CHECKS > 0);
const _: () = assert!(MAX_ARGS > 0);
const _: () = assert!(MAX_REFERENCES > 0);
const _: () = assert!(MAX_TEXT_CHARS > 0);

#[derive(Clone, Copy)]
enum ReferenceKind {
    Identifier,
    Path,
}

#[derive(Clone, Copy)]
struct ReferenceSet<'a> {
    values: &'a [&'a str],
    label: &'static str,
    kind: ReferenceKind,
    allow_empty: bool,
}

#[derive(Clone, Copy)]
struct RegistryHashInput<'a> {
    repository_id: &'a str,
    registry_id: &'a str,
    policy_hash: &'a str,
    fallback_ids: &'a [String],
    rows: &'a [AgentCheckRow],
}

/// Validates an already-loaded repository-owned check registry.
///
/// # Errors
///
/// Returns bounded diagnostics for malformed identities, unsafe argv or paths,
/// unknown prerequisites, cycles, duplicate rows, or stale Cairn bindings.
pub fn validate_agent_check_registry(
    draft: &AgentCheckRegistryDraft<'_>,
) -> Result<AgentCheckRegistry, Vec<AgentCheckDiagnostic>> {
    let mut issues = Vec::new();
    validate_registry_header(draft, &mut issues);
    validate_registry_bounds(draft, &mut issues);
    let mut rows = Vec::with_capacity(draft.checks.len().min(MAX_CHECKS));
    for row in draft.checks.iter().take(MAX_CHECKS) {
        if let Some(validated) = validate_row(row, &mut issues) {
            rows.push(validated);
        }
    }
    validate_registry_links(draft, &rows, &mut issues);
    if !issues.is_empty() {
        return Err(issues);
    }
    rows.sort_by(|left, right| left.check_id.cmp(&right.check_id));
    let fallback_check_ids = sorted_unique_strings(draft.fallback_check_ids);
    let registry_hash = registry_hash(RegistryHashInput {
        repository_id: draft.repository_id,
        registry_id: draft.registry_id,
        policy_hash: draft.cairn_policy_hash,
        fallback_ids: fallback_check_ids.as_slice(),
        rows: rows.as_slice(),
    });
    Ok(AgentCheckRegistry {
        repository_id: draft.repository_id.to_string(),
        registry_id: draft.registry_id.to_string(),
        cairn_policy_hash: draft.cairn_policy_hash.to_string(),
        fallback_check_ids,
        checks: rows,
        registry_hash,
    })
}

fn validate_registry_header(
    draft: &AgentCheckRegistryDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    require_exact(
        draft.schema,
        AGENT_CHECK_REGISTRY_SCHEMA,
        "registry schema is unsupported",
        issues,
    );
    require_exact(
        draft.supported_cairn_schema,
        CAIRN_COMPLETION_SCHEMA,
        "registry does not pin the supported Cairn completion schema",
        issues,
    );
    require_exact(
        draft.cairn_policy_hash,
        PINNED_CAIRN_POLICY_HASH,
        "registry Cairn policy identity is stale",
        issues,
    );
    for (label, value) in [
        ("repository identity", draft.repository_id),
        ("registry identity", draft.registry_id),
    ] {
        if !is_safe_identifier(value) {
            issues.push(diagnostic(
                AgentCheckDiagnosticCode::RegistryInvalid,
                DiagnosticContext::NONE,
                format!("{label} is unsafe"),
                &[],
            ));
        }
    }
}

fn validate_registry_bounds(
    draft: &AgentCheckRegistryDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    if draft.checks.is_empty() || draft.checks.len() > MAX_CHECKS {
        issues.push(diagnostic(
            AgentCheckDiagnosticCode::RegistryInvalid,
            DiagnosticContext::NONE,
            "registry check count is outside the finite bound".to_string(),
            &[],
        ));
    }
    if draft.fallback_check_ids.is_empty()
        || draft.fallback_check_ids.len() > MAX_REFERENCES
        || !all_unique(draft.fallback_check_ids)
    {
        issues.push(diagnostic(
            AgentCheckDiagnosticCode::RegistryInvalid,
            DiagnosticContext::NONE,
            "conservative fallback set is empty, duplicate, or oversized".to_string(),
            &[],
        ));
    }
}

fn validate_row(
    draft: &AgentCheckRowDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) -> Option<AgentCheckRow> {
    let issue_count = issues.len();
    require_safe_identifier(draft.check_id, "check ID", draft, issues);
    require_safe_identifier(draft.profile, "profile", draft, issues);
    require_safe_argv(draft.argv, "exact argv", draft, issues);
    if draft.argv != draft.reproduction_argv {
        issues.push(row_diagnostic(
            draft,
            "reproduction argv differs from exact execution argv",
        ));
    }
    require_safe_argv(draft.reproduction_argv, "reproduction argv", draft, issues);
    validate_row_references(draft, issues);
    validate_row_labels(draft, issues);
    if issues.len() != issue_count {
        return None;
    }
    Some(AgentCheckRow {
        check_id: draft.check_id.to_string(),
        profile: draft.profile.to_string(),
        argv: strings(draft.argv),
        affected_paths: sorted_unique_strings(draft.affected_paths),
        affected_crates: sorted_unique_strings(draft.affected_crates),
        affected_surfaces: sorted_unique_strings(draft.affected_surfaces),
        prerequisites: sorted_unique_strings(draft.prerequisites),
        capability: draft.capability,
        tier: draft.tier,
        risk: draft.risk.to_string(),
        timeout_class: draft.timeout_class.to_string(),
        output_budget_class: draft.output_budget_class.to_string(),
        expected_artifacts: sorted_unique_strings(draft.expected_artifacts),
        documentation_refs: sorted_unique_strings(draft.documentation_refs),
        positive_contract: draft.positive_contract.to_string(),
        negative_contract: draft.negative_contract.to_string(),
    })
}

fn validate_row_references(draft: &AgentCheckRowDraft<'_>, issues: &mut Vec<AgentCheckDiagnostic>) {
    for references in [
        ReferenceSet {
            values: draft.affected_paths,
            label: "affected paths",
            kind: ReferenceKind::Path,
            allow_empty: false,
        },
        ReferenceSet {
            values: draft.affected_crates,
            label: "affected crates",
            kind: ReferenceKind::Identifier,
            allow_empty: true,
        },
        ReferenceSet {
            values: draft.affected_surfaces,
            label: "affected surfaces",
            kind: ReferenceKind::Identifier,
            allow_empty: false,
        },
        ReferenceSet {
            values: draft.expected_artifacts,
            label: "expected artifacts",
            kind: ReferenceKind::Path,
            allow_empty: false,
        },
        ReferenceSet {
            values: draft.documentation_refs,
            label: "documentation refs",
            kind: ReferenceKind::Path,
            allow_empty: false,
        },
    ] {
        require_reference_set(references, draft, issues);
    }
    require_prerequisite_set(draft, issues);
}

fn validate_row_labels(draft: &AgentCheckRowDraft<'_>, issues: &mut Vec<AgentCheckDiagnostic>) {
    if draft.working_root != WORKING_ROOT {
        issues.push(row_diagnostic(
            draft,
            "reproduction working root must be the safe repository-root label",
        ));
    }
    for (label, value, allowed) in [
        ("risk", draft.risk, SAFE_RISK_LABELS),
        ("timeout", draft.timeout_class, SAFE_TIMEOUT_LABELS),
        (
            "output budget",
            draft.output_budget_class,
            SAFE_OUTPUT_LABELS,
        ),
    ] {
        if !allowed.contains(&value) {
            issues.push(row_diagnostic(
                draft,
                format!("{label} class is unsupported").as_str(),
            ));
        }
    }
    for (label, value) in [
        ("positive contract", draft.positive_contract),
        ("negative contract", draft.negative_contract),
    ] {
        if !is_safe_path(value) {
            issues.push(row_diagnostic(
                draft,
                format!("{label} path is unsafe").as_str(),
            ));
        }
    }
}

fn validate_registry_links(
    draft: &AgentCheckRegistryDraft<'_>,
    rows: &[AgentCheckRow],
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    let ids = rows
        .iter()
        .map(|row| row.check_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != draft.checks.len() {
        issues.push(diagnostic(
            AgentCheckDiagnosticCode::RegistryInvalid,
            DiagnosticContext::NONE,
            "registry check IDs are duplicate or invalid".to_string(),
            &[],
        ));
    }
    for fallback in draft.fallback_check_ids {
        if !ids.contains(fallback) {
            issues.push(diagnostic(
                AgentCheckDiagnosticCode::RegistryInvalid,
                DiagnosticContext::for_check(fallback, None),
                "fallback names an unknown check".to_string(),
                &[],
            ));
        }
    }
    for row in rows {
        for prerequisite in &row.prerequisites {
            if prerequisite == &row.check_id || !ids.contains(prerequisite.as_str()) {
                let argv = row.argv.iter().map(String::as_str).collect::<Vec<_>>();
                issues.push(diagnostic(
                    AgentCheckDiagnosticCode::RegistryInvalid,
                    DiagnosticContext::for_check(row.check_id.as_str(), None),
                    "prerequisite must name another registry check".to_string(),
                    argv.as_slice(),
                ));
            }
        }
    }
    if has_prerequisite_cycle(rows) {
        issues.push(diagnostic(
            AgentCheckDiagnosticCode::RegistryInvalid,
            DiagnosticContext::NONE,
            "registry prerequisite graph contains a cycle".to_string(),
            &[],
        ));
    }
}

fn require_exact(
    actual: &str,
    expected: &str,
    summary: &str,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    if actual != expected {
        issues.push(diagnostic(
            AgentCheckDiagnosticCode::RegistryInvalid,
            DiagnosticContext::NONE,
            summary.to_string(),
            &[],
        ));
    }
}

fn require_safe_identifier(
    value: &str,
    label: &str,
    row: &AgentCheckRowDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    if !is_safe_identifier(value) {
        issues.push(row_diagnostic(row, format!("{label} is unsafe").as_str()));
    }
}

fn require_safe_argv(
    argv: &[&str],
    label: &str,
    row: &AgentCheckRowDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    if !is_safe_argv(argv) {
        issues.push(row_diagnostic(
            row,
            format!("{label} is empty, oversized, duplicated, or unsafe").as_str(),
        ));
    }
}

fn require_reference_set(
    references: ReferenceSet<'_>,
    row: &AgentCheckRowDraft<'_>,
    issues: &mut Vec<AgentCheckDiagnostic>,
) {
    let invalid_item = references.values.iter().any(|value| match references.kind {
        ReferenceKind::Path => !is_safe_path(value),
        ReferenceKind::Identifier => !is_safe_identifier(value),
    });
    if (!references.allow_empty && references.values.is_empty())
        || references.values.len() > MAX_REFERENCES
        || !all_unique(references.values)
        || invalid_item
    {
        issues.push(row_diagnostic(
            row,
            format!(
                "{} are empty, oversized, duplicate, or unsafe",
                references.label
            )
            .as_str(),
        ));
    }
}

fn require_prerequisite_set(row: &AgentCheckRowDraft<'_>, issues: &mut Vec<AgentCheckDiagnostic>) {
    if row.prerequisites.len() > MAX_REFERENCES
        || !all_unique(row.prerequisites)
        || row
            .prerequisites
            .iter()
            .any(|value| !is_safe_identifier(value))
    {
        issues.push(row_diagnostic(
            row,
            "prerequisites are oversized, duplicate, or unsafe",
        ));
    }
}

fn row_diagnostic(row: &AgentCheckRowDraft<'_>, summary: &str) -> AgentCheckDiagnostic {
    let reproduction = if is_safe_argv(row.reproduction_argv) {
        row.reproduction_argv
    } else {
        &[]
    };
    diagnostic(
        AgentCheckDiagnosticCode::RegistryInvalid,
        DiagnosticContext::for_check(row.check_id, None),
        summary.to_string(),
        reproduction,
    )
}

fn has_prerequisite_cycle(rows: &[AgentCheckRow]) -> bool {
    let mut indegree = rows
        .iter()
        .map(|row| (row.check_id.as_str(), 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows {
        for prerequisite in &row.prerequisites {
            if let Some(value) = indegree.get_mut(row.check_id.as_str()) {
                *value = value.saturating_add(1);
            }
            dependents
                .entry(prerequisite.as_str())
                .or_default()
                .push(row.check_id.as_str());
        }
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    let mut visited = 0_usize;
    while let Some(next) = ready.pop_first() {
        visited = visited.saturating_add(1);
        if let Some(children) = dependents.get(next) {
            for child in children {
                if let Some(value) = indegree.get_mut(child) {
                    *value = value.saturating_sub(1);
                    if *value == 0 {
                        ready.insert(child);
                    }
                }
            }
        }
    }
    visited != rows.len()
}

fn registry_hash(input: RegistryHashInput<'_>) -> String {
    let mut parts = vec![
        AGENT_CHECK_REGISTRY_SCHEMA.to_string(),
        input.repository_id.to_string(),
        input.registry_id.to_string(),
        CAIRN_COMPLETION_SCHEMA.to_string(),
        input.policy_hash.to_string(),
        input.fallback_ids.join(","),
    ];
    for row in input.rows {
        parts.extend([
            row.check_id.clone(),
            row.profile.clone(),
            row.argv.join("\u{1f}"),
            row.affected_paths.join(","),
            row.affected_crates.join(","),
            row.affected_surfaces.join(","),
            row.prerequisites.join(","),
            row.capability.as_str().to_string(),
            row.tier.as_str().to_string(),
            row.risk.clone(),
            row.timeout_class.clone(),
            row.output_budget_class.clone(),
            row.expected_artifacts.join(","),
            row.documentation_refs.join(","),
            row.positive_contract.clone(),
            row.negative_contract.clone(),
        ]);
    }
    hash_parts(HASH_DOMAIN, parts.iter().map(String::as_str))
}

#[derive(Clone, Copy)]
pub(super) struct DiagnosticContext<'a> {
    check_id: Option<&'a str>,
    status: Option<CheckStatus>,
}

impl<'a> DiagnosticContext<'a> {
    pub(super) const NONE: Self = Self {
        check_id: None,
        status: None,
    };

    pub(super) const fn for_check(check_id: &'a str, status: Option<CheckStatus>) -> Self {
        Self {
            check_id: Some(check_id),
            status,
        }
    }
}

pub(super) fn diagnostic(
    code: AgentCheckDiagnosticCode,
    context: DiagnosticContext<'_>,
    summary: String,
    argv: &[&str],
) -> AgentCheckDiagnostic {
    let DiagnosticContext { check_id, status } = context;
    let bounded_summary = if summary.chars().count() <= MAX_TEXT_CHARS {
        summary
    } else {
        "diagnostic summary exceeded the safe bound".to_string()
    };
    let argv = argv
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let status_label = status.map_or("", CheckStatus::as_str);
    let id = hash_parts(
        "animus.authoritative-completion.diagnostic.v1",
        [
            code.as_str(),
            check_id.unwrap_or(""),
            status_label,
            bounded_summary.as_str(),
        ],
    );
    AgentCheckDiagnostic {
        diagnostic_id: id,
        code,
        check_id: check_id.map(str::to_string),
        status,
        remediation_label: "refresh-and-rerun-required-check".to_string(),
        summary: bounded_summary,
        subject_hash: None,
        registry_hash: None,
        selection_hash: None,
        evidence_hash: None,
        receipt_hash: None,
        reproduction_argv: argv,
        working_root: WORKING_ROOT.to_string(),
        source_path: None,
        source_line: None,
        source_column: None,
        artifact_refs: Vec::new(),
        non_claims: DIAGNOSTIC_NON_CLAIMS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

pub(super) fn is_hash(value: &str) -> bool {
    value.len() == HASH_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TEXT_CHARS
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b'-' | b'/')
        })
        && !value.contains("..")
        && !value.starts_with('/')
        && !value.ends_with('/')
}

pub(super) fn is_safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TEXT_CHARS
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

pub(super) fn is_safe_argv(values: &[&str]) -> bool {
    if values.is_empty() || values.len() > MAX_ARGS || !all_unique(values) {
        return false;
    }
    let safe = values.iter().all(|value| {
        !value.is_empty()
            && value.len() <= MAX_TEXT_CHARS
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(byte, b'.' | b'_' | b'/' | b'#' | b'=' | b'<' | b'>' | b'-')
            })
    });
    let shell_escape = values
        .windows(SHELL_ESCAPE_ARG_COUNT)
        .any(|pair| matches!(pair, ["bash" | "sh", "-c"]));
    safe && !shell_escape
}

pub(super) fn hash_parts<'a>(domain: &str, parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let length = part.len().to_string();
        hasher.update(length.as_bytes());
        hasher.update(b":");
        hasher.update(part.as_bytes());
        hasher.update(b"|");
    }
    hasher.finalize().to_hex().to_string()
}

fn all_unique(values: &[&str]) -> bool {
    values.iter().copied().collect::<BTreeSet<_>>().len() == values.len()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn sorted_unique_strings(values: &[&str]) -> Vec<String> {
    values
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect()
}
