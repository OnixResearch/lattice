// r[impl agent_harness_runtime.authoritative_completion.selection]
// Selection rejects untrusted facts through diagnostics; public compatibility names and bounded
// mutable sinks remain stable for embedders.
#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::borrowed_argument_types,
        tigerstyle::non_trait_imports,
        tigerstyle::path_segment_repetition
    )
)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::AgentCheckDiagnostic;
use super::AgentCheckDiagnosticCode;
use super::AgentCheckRegistry;
use super::AgentCheckSelection;
use super::CairnRequiredCheck;
use super::ChangedArtifactDraft;
use super::CheckCapability;
use super::CheckTier;
use super::validation::DiagnosticContext;
use super::validation::diagnostic;
use super::validation::hash_parts;
use super::validation::is_hash;
use super::validation::is_safe_identifier;
use super::validation::is_safe_path;

const CHANGED_ARTIFACT_HASH_DOMAIN: &str = "animus.authoritative-completion.changed-artifacts.v1";
const EMPTY_ARGV: &[&str] = &[];
const JSON_QUOTE_CAPACITY: usize = 2;

/// Borrowed inputs for pure repository-check selection.
#[derive(Clone, Copy)]
pub struct AgentCheckSelectionRequest<'a> {
    /// Validated repository-owned registry.
    pub registry: &'a AgentCheckRegistry,
    /// Host-supplied changed-artifact facts.
    pub changed_artifacts: &'a [ChangedArtifactDraft<'a>],
    /// Host-observed capabilities available to selected checks.
    pub available_capabilities: &'a [CheckCapability],
    /// Maximum admitted check tier.
    pub maximum_tier: CheckTier,
    /// BLAKE3 identity of the final subject under validation.
    pub final_subject_hash: &'a str,
}

#[derive(Clone, Copy)]
struct PolicyBlockerRequest<'a> {
    registry: &'a AgentCheckRegistry,
    selected_ids: &'a BTreeSet<String>,
    capabilities: &'a [CheckCapability],
    maximum_tier: CheckTier,
}

/// Selects repository-required checks from already-loaded facts without effects.
///
/// Unknown safe paths use the registry's conservative fallback set. Capability
/// and tier restrictions remain blockers rather than silently removing required
/// checks.
#[must_use]
pub fn select_required_agent_checks(request: AgentCheckSelectionRequest<'_>) -> AgentCheckSelection {
    let AgentCheckSelectionRequest {
        registry,
        changed_artifacts,
        available_capabilities,
        maximum_tier,
        final_subject_hash,
    } = request;
    let mut blockers = Vec::new();
    validate_selection_input(changed_artifacts, final_subject_hash, &mut blockers);
    let changed_artifact_hash = changed_artifact_hash(changed_artifacts);
    let selected_ids = selected_check_ids(registry, changed_artifacts);
    let selected_ids = prerequisite_closure(registry, &selected_ids);
    let required_checks = required_checks(registry, &selected_ids);
    append_policy_blockers(
        PolicyBlockerRequest {
            registry,
            selected_ids: &selected_ids,
            capabilities: available_capabilities,
            maximum_tier,
        },
        &mut blockers,
    );
    if required_checks.is_empty() {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::ChangedArtifactInvalid,
            DiagnosticContext::NONE,
            "required-check selection is empty".to_string(),
            EMPTY_ARGV,
        ));
    }
    let all_ids = registry.checks.iter().map(|row| row.check_id.clone()).collect::<BTreeSet<_>>();
    let not_selected = all_ids.difference(&selected_ids).cloned().collect::<Vec<_>>();
    let selection_hash = cairn_selection_hash(required_checks.as_slice());
    AgentCheckSelection {
        repository_id: registry.repository_id.clone(),
        registry_id: registry.registry_id.clone(),
        registry_hash: registry.registry_hash.clone(),
        selection_hash,
        policy_hash: registry.cairn_policy_hash.clone(),
        changed_artifact_hash,
        final_subject_hash: final_subject_hash.to_string(),
        required_checks,
        not_selected,
        blockers,
    }
}

/// Computes Cairn's canonical command hash for an exact argv array.
#[must_use]
pub fn cairn_command_hash(command: &[String]) -> String {
    let canonical = json_string_array(command);
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

/// Computes Cairn's canonical selection hash for required check material.
#[must_use]
pub fn cairn_selection_hash(required_checks: &[CairnRequiredCheck]) -> String {
    let mut rows = required_checks
        .iter()
        .map(canonical_selection_row)
        .map(|canonical| {
            let identity = blake3::hash(canonical.as_bytes()).to_hex().to_string();
            (identity, canonical)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let canonical = format!("[{}]", rows.iter().map(|(_, row)| row.as_str()).collect::<Vec<_>>().join(","));
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

fn validate_selection_input(
    artifacts: &[ChangedArtifactDraft<'_>],
    final_subject_hash: &str,
    blockers: &mut Vec<AgentCheckDiagnostic>,
) {
    if artifacts.is_empty() {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::ChangedArtifactInvalid,
            DiagnosticContext::NONE,
            "changed-artifact snapshot is empty".to_string(),
            EMPTY_ARGV,
        ));
    }
    if !is_hash(final_subject_hash) {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::ChangedArtifactInvalid,
            DiagnosticContext::NONE,
            "final subject identity is not BLAKE3 hex".to_string(),
            EMPTY_ARGV,
        ));
    }
    for artifact in artifacts {
        if !is_safe_path(artifact.path) || !is_safe_identifier(artifact.surface) || !is_hash(artifact.content_hash) {
            blockers.push(diagnostic(
                AgentCheckDiagnosticCode::ChangedArtifactInvalid,
                DiagnosticContext::NONE,
                "changed artifact contains an unsafe path, surface, or hash".to_string(),
                EMPTY_ARGV,
            ));
        }
    }
}

fn selected_check_ids(registry: &AgentCheckRegistry, artifacts: &[ChangedArtifactDraft<'_>]) -> BTreeSet<String> {
    let mut selected = BTreeSet::new();
    for artifact in artifacts {
        let direct = registry
            .checks
            .iter()
            .filter(|row| row_matches(row, artifact))
            .map(|row| row.check_id.clone())
            .collect::<Vec<_>>();
        if direct.is_empty() {
            selected.extend(registry.fallback_check_ids.iter().cloned());
        } else {
            selected.extend(direct);
        }
    }
    selected
}

fn row_matches(row: &super::AgentCheckRow, artifact: &ChangedArtifactDraft<'_>) -> bool {
    row.affected_paths.iter().any(|prefix| path_matches(prefix, artifact.path))
        || row.affected_surfaces.iter().any(|surface| surface == artifact.surface)
}

fn path_matches(prefix: &str, path: &str) -> bool {
    path == prefix || path.strip_prefix(prefix).is_some_and(|suffix| suffix.starts_with('/'))
}

fn prerequisite_closure(registry: &AgentCheckRegistry, initial: &BTreeSet<String>) -> BTreeSet<String> {
    let rows = registry.checks.iter().map(|row| (row.check_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let mut selected = initial.clone();
    let mut pending = initial.iter().cloned().collect::<Vec<_>>();
    while let Some(check_id) = pending.pop() {
        if let Some(row) = rows.get(check_id.as_str()) {
            for prerequisite in &row.prerequisites {
                if selected.insert(prerequisite.clone()) {
                    pending.push(prerequisite.clone());
                }
            }
        }
    }
    selected
}

fn required_checks(registry: &AgentCheckRegistry, selected_ids: &BTreeSet<String>) -> Vec<CairnRequiredCheck> {
    let rows = registry
        .checks
        .iter()
        .filter(|row| selected_ids.contains(&row.check_id))
        .map(|row| (row.check_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut indegree = rows.keys().map(|id| (*id, 0_usize)).collect::<BTreeMap<_, _>>();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows.values() {
        for prerequisite in &row.prerequisites {
            if selected_ids.contains(prerequisite) {
                if let Some(value) = indegree.get_mut(row.check_id.as_str()) {
                    *value = value.saturating_add(1);
                }
                dependents.entry(prerequisite.as_str()).or_default().push(row.check_id.as_str());
            }
        }
    }
    let mut ready = indegree.iter().filter(|(_, count)| **count == 0).map(|(id, _)| *id).collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(rows.len());
    while let Some(next) = ready.pop_first() {
        if let Some(row) = rows.get(next) {
            let command = row.argv.clone();
            ordered.push(CairnRequiredCheck {
                check_id: row.check_id.clone(),
                command_hash: cairn_command_hash(command.as_slice()),
                command,
                prerequisites: row.prerequisites.clone(),
            });
        }
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
    ordered
}

fn append_policy_blockers(request: PolicyBlockerRequest<'_>, blockers: &mut Vec<AgentCheckDiagnostic>) {
    for row in &request.registry.checks {
        if !request.selected_ids.contains(&row.check_id) {
            continue;
        }
        let argv = row.argv.iter().map(String::as_str).collect::<Vec<_>>();
        if !request.capabilities.contains(&row.capability) {
            blockers.push(diagnostic(
                AgentCheckDiagnosticCode::CapabilityUnavailable,
                DiagnosticContext::for_check(row.check_id.as_str(), None),
                "required host capability is unavailable".to_string(),
                argv.as_slice(),
            ));
        }
        if row.tier > request.maximum_tier {
            blockers.push(diagnostic(
                AgentCheckDiagnosticCode::TierUnavailable,
                DiagnosticContext::for_check(row.check_id.as_str(), None),
                "required check exceeds the admitted tier".to_string(),
                argv.as_slice(),
            ));
        }
    }
}

fn changed_artifact_hash(artifacts: &[ChangedArtifactDraft<'_>]) -> String {
    let mut rows = artifacts
        .iter()
        .map(|artifact| format!("{}\u{1f}{}\u{1f}{}", artifact.path, artifact.surface, artifact.content_hash))
        .collect::<Vec<_>>();
    rows.sort();
    hash_parts(CHANGED_ARTIFACT_HASH_DOMAIN, rows.iter().map(String::as_str))
}

fn canonical_selection_row(check: &CairnRequiredCheck) -> String {
    let mut prerequisites = check.prerequisites.clone();
    prerequisites.sort();
    prerequisites.dedup();
    format!(
        "{{\"check_id\":{},\"command_hash\":{},\"prerequisites\":{}}}",
        json_string(check.check_id.as_str()),
        json_string(check.command_hash.as_str()),
        json_string_array(prerequisites.as_slice()),
    )
}

fn json_string_array(values: &[String]) -> String {
    format!("[{}]", values.iter().map(|value| json_string(value.as_str())).collect::<Vec<_>>().join(","))
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len().saturating_add(JSON_QUOTE_CAPACITY));
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => {
                output.push_str(format!("\\u{:04x}", u32::from(value)).as_str());
            }
            value => output.push(value),
        }
    }
    output.push('"');
    output
}
