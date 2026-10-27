// r[impl agent_harness_runtime.authoritative_completion.execution_receipts]
// r[impl agent_harness_runtime.authoritative_completion.pass_only]
// r[impl agent_harness_runtime.authoritative_completion.receipt_gate]
// r[impl agent_harness_runtime.authoritative_completion.feedback]
// Typed diagnostics replace panics for untrusted inputs; the compatibility API keeps reviewed public names and bounded mutable sinks.
#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::borrowed_argument_types,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::path_segment_repetition,
        tigerstyle::unbounded_collection_growth
    )
)]
use std::collections::{BTreeMap, BTreeSet};

use super::selection::cairn_command_hash;
use super::validation::{
    DiagnosticContext, diagnostic, hash_parts, is_hash, is_safe_argv, is_safe_identifier,
    is_safe_path,
};
use super::{
    AgentCheckDiagnostic, AgentCheckDiagnosticCode, AgentCheckExecutionDraft, AgentCheckSelection,
    AuthoritativeCompletionCandidate, AuthoritativeCompletionCandidateDraft,
    AuthoritativeCompletionOutcome, AuthoritativeCompletionReceipt, CAIRN_COMPLETION_SCHEMA,
    CairnCompletionSummaryDraft, CheckStatus, CompletionCandidateSource, ExecutionEvidenceReport,
};

const TYPED_COMPLETION_REQUEST: &str = "completion-request";
const STEEL_COMPATIBILITY_REQUEST: &str = "COMPLETE";
const AUTHORITATIVE_CLASSIFICATION: &str = "authoritative";
const PASS_VERDICT: &str = "PASS";
const COMPLETION_RECEIPT_SCHEMA: &str = "animus.authoritative-completion-receipt.v1";
const MAX_OUTPUT_BYTES: u64 = 1_048_576;
const MAX_DIAGNOSTIC_CHARS: usize = 512;
const ZERO_EXIT_CODE: i32 = 0;
const ONE_BASED_MINIMUM: u32 = 1;
const EMPTY_ARGV: &[&str] = &[];
const UNSAFE_METADATA_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
    "credential",
    "secret",
    "environment value",
    "raw pty",
    "unbounded log",
    "/home/",
    "/tmp/",
];
const COMPLETION_NON_CLAIMS: &[&str] = &[
    "completion proves exact selected-check coverage for the supplied identities only",
    "completion does not prove check-oracle soundness",
    "completion does not prove honest command execution beyond supplied host evidence",
    "completion does not prove model correctness",
    "completion does not prove whole-product correctness",
    "completion does not prove production readiness or arbitrary-host support",
    "completion does not grant release, push, or pull-request authority",
];

const _: () = assert!(MAX_OUTPUT_BYTES > 0);
const _: () = assert!(MAX_DIAGNOSTIC_CHARS > 0);
const _: () = assert!(ONE_BASED_MINIMUM > 0);

/// Validates and binds a typed or exact Steel completion request.
///
/// # Errors
///
/// Returns a bounded diagnostic when request spelling, work identity, or final
/// subject identity is malformed. A valid candidate still carries no passing
/// completion authority.
pub fn admit_completion_candidate(
    draft: &AuthoritativeCompletionCandidateDraft<'_>,
) -> Result<AuthoritativeCompletionCandidate, Box<AgentCheckDiagnostic>> {
    let expected = match draft.source {
        CompletionCandidateSource::Typed => TYPED_COMPLETION_REQUEST,
        CompletionCandidateSource::SteelCompatibility => STEEL_COMPATIBILITY_REQUEST,
    };
    if draft.request != expected
        || !is_safe_identifier(draft.work_id)
        || !is_hash(draft.subject_hash)
    {
        return Err(Box::new(diagnostic(
            AgentCheckDiagnosticCode::CandidateInvalid,
            DiagnosticContext::NONE,
            "completion candidate spelling or identity is invalid".to_string(),
            EMPTY_ARGV,
        )));
    }
    let source = match draft.source {
        CompletionCandidateSource::Typed => "typed",
        CompletionCandidateSource::SteelCompatibility => "steel-compatibility",
    };
    let work_id_hash = blake3::hash(draft.work_id.as_bytes()).to_hex().to_string();
    let candidate_hash = hash_parts(
        "animus.authoritative-completion.candidate.v1",
        [
            source,
            draft.request,
            work_id_hash.as_str(),
            draft.subject_hash,
        ],
    );
    Ok(AuthoritativeCompletionCandidate {
        source: draft.source,
        request: draft.request.to_string(),
        work_id_hash,
        subject_hash: draft.subject_hash.to_string(),
        candidate_hash,
    })
}

/// Validates exact result coverage and pass-only execution evidence.
#[must_use]
pub fn admit_execution_results(
    selection: &AgentCheckSelection,
    results: &[AgentCheckExecutionDraft<'_>],
) -> ExecutionEvidenceReport {
    let mut diagnostics = selection.blockers.clone();
    let required = selection
        .required_checks
        .iter()
        .map(|check| (check.check_id.as_str(), check))
        .collect::<BTreeMap<_, _>>();
    let by_id = index_execution_results(results, &required, &mut diagnostics);
    let mut base_passed = BTreeSet::new();
    let mut statuses = BTreeMap::new();
    for check in &selection.required_checks {
        let rows = by_id
            .get(check.check_id.as_str())
            .cloned()
            .unwrap_or_default();
        if rows.is_empty() {
            statuses.insert(check.check_id.clone(), CheckStatus::Unknown);
            diagnostics.push(diagnostic(
                AgentCheckDiagnosticCode::MissingResult,
                DiagnosticContext::for_check(check.check_id.as_str(), Some(CheckStatus::Unknown)),
                "required check has no execution result".to_string(),
                command_refs(check.command.as_slice()).as_slice(),
            ));
            continue;
        }
        if rows.len() != 1 {
            statuses.insert(check.check_id.clone(), CheckStatus::Unknown);
            diagnostics.push(result_diagnostic(
                AgentCheckDiagnosticCode::DuplicateResult,
                rows[0],
                Some(check),
                "required check has duplicate execution results",
            ));
            continue;
        }
        let result = rows[0];
        statuses.insert(check.check_id.clone(), result.status);
        let before = diagnostics.len();
        validate_result(selection, check, result, &mut diagnostics);
        if !result.status.is_passing() {
            diagnostics.push(result_diagnostic(
                AgentCheckDiagnosticCode::NonPassingResult,
                result,
                Some(check),
                "required check status is not completion eligible",
            ));
        }
        if diagnostics.len() == before && result.status.is_passing() {
            base_passed.insert(check.check_id.clone());
        }
    }
    let mut passed_checks = Vec::new();
    for check in &selection.required_checks {
        if !base_passed.contains(&check.check_id) {
            continue;
        }
        let prerequisites_pass = check
            .prerequisites
            .iter()
            .all(|prerequisite| base_passed.contains(prerequisite));
        if prerequisites_pass {
            passed_checks.push(check.check_id.clone());
        } else {
            statuses.insert(check.check_id.clone(), CheckStatus::Blocked);
            diagnostics.push(diagnostic(
                AgentCheckDiagnosticCode::PrerequisiteNotPassing,
                DiagnosticContext::for_check(check.check_id.as_str(), Some(CheckStatus::Blocked)),
                "required prerequisite did not pass".to_string(),
                command_refs(check.command.as_slice()).as_slice(),
            ));
        }
    }
    passed_checks.sort();
    let mut non_passing_checks = statuses
        .into_iter()
        .filter(|(check_id, _)| !passed_checks.contains(check_id))
        .collect::<Vec<_>>();
    non_passing_checks.sort_by(|left, right| left.0.cmp(&right.0));
    diagnostics.sort_by(|left, right| {
        (left.code(), left.check_id(), left.diagnostic_id()).cmp(&(
            right.code(),
            right.check_id(),
            right.diagnostic_id(),
        ))
    });
    let evidence_hash = execution_evidence_hash(results);
    ExecutionEvidenceReport {
        passed_checks,
        non_passing_checks,
        diagnostics,
        evidence_hash,
    }
}

fn index_execution_results<'slice, 'draft>(
    results: &'slice [AgentCheckExecutionDraft<'draft>],
    required: &BTreeMap<&str, &super::CairnRequiredCheck>,
    diagnostics: &mut Vec<AgentCheckDiagnostic>,
) -> BTreeMap<String, Vec<&'slice AgentCheckExecutionDraft<'draft>>> {
    let mut by_id = BTreeMap::new();
    for result in results {
        by_id
            .entry(result.check_id.to_string())
            .or_insert_with(Vec::new)
            .push(result);
        if !required.contains_key(result.check_id) {
            diagnostics.push(result_diagnostic(
                AgentCheckDiagnosticCode::UnknownResult,
                result,
                None,
                "execution result names an unselected check",
            ));
        }
    }
    by_id
}

/// Reduces candidate, selection, host evidence, and Cairn authority once.
#[must_use]
pub fn evaluate_authoritative_completion(
    candidate: &AuthoritativeCompletionCandidate,
    selection: &AgentCheckSelection,
    evidence: &ExecutionEvidenceReport,
    cairn: Option<&CairnCompletionSummaryDraft<'_>>,
) -> AuthoritativeCompletionOutcome {
    let mut blockers = evidence.diagnostics.clone();
    if candidate.subject_hash != selection.final_subject_hash {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::CandidateInvalid,
            DiagnosticContext::NONE,
            "completion candidate is stale for the final subject".to_string(),
            EMPTY_ARGV,
        ));
    }
    if !evidence.non_passing_checks.is_empty() {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::NonPassingResult,
            DiagnosticContext::NONE,
            "required check set is not exact pass-only coverage".to_string(),
            EMPTY_ARGV,
        ));
    }
    match cairn {
        Some(summary) => validate_cairn_summary(selection, evidence, summary, &mut blockers),
        None => blockers.push(diagnostic(
            AgentCheckDiagnosticCode::CairnSummaryInvalid,
            DiagnosticContext::NONE,
            "current Cairn completion-validation summary is missing".to_string(),
            EMPTY_ARGV,
        )),
    }
    blockers.sort_by(|left, right| {
        (left.code(), left.check_id(), left.diagnostic_id()).cmp(&(
            right.code(),
            right.check_id(),
            right.diagnostic_id(),
        ))
    });
    blockers.dedup_by(|left, right| left.diagnostic_id == right.diagnostic_id);
    if !blockers.is_empty() {
        return AuthoritativeCompletionOutcome::Blocked(blockers);
    }
    match cairn {
        Some(summary) => AuthoritativeCompletionOutcome::Verified(Box::new(completion_receipt(
            candidate, selection, evidence, summary,
        ))),
        None => AuthoritativeCompletionOutcome::Blocked(vec![diagnostic(
            AgentCheckDiagnosticCode::CairnSummaryInvalid,
            DiagnosticContext::NONE,
            "current Cairn completion-validation summary is missing".to_string(),
            EMPTY_ARGV,
        )]),
    }
}

fn validate_result(
    selection: &AgentCheckSelection,
    check: &super::CairnRequiredCheck,
    result: &AgentCheckExecutionDraft<'_>,
    diagnostics: &mut Vec<AgentCheckDiagnostic>,
) {
    let command = result
        .command
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let command_matches = command == check.command
        && result.command_hash == check.command_hash
        && result.command_hash == cairn_command_hash(command.as_slice());
    let identities_match = result.subject_hash == selection.final_subject_hash
        && result.registry_hash == selection.registry_hash
        && result.selection_hash == selection.selection_hash
        && result.policy_hash == selection.policy_hash;
    let hashes_valid = [
        result.command_hash,
        result.subject_hash,
        result.registry_hash,
        result.selection_hash,
        result.policy_hash,
        result.evidence_hash,
        result.execution_receipt_hash,
        result.stdout_hash,
        result.stderr_hash,
    ]
    .iter()
    .all(|value| is_hash(value));
    let output_bounded =
        result.stdout_bytes <= MAX_OUTPUT_BYTES && result.stderr_bytes <= MAX_OUTPUT_BYTES;
    let artifacts_safe = result
        .artifacts
        .iter()
        .all(|artifact| is_safe_path(artifact.path) && is_hash(artifact.hash));
    let diagnostic_safe = result
        .diagnostic
        .as_ref()
        .is_none_or(|value| producer_diagnostic_is_safe(value, result.command));
    let passing_facts = !result.status.is_passing()
        || (result.exit_code == Some(ZERO_EXIT_CODE)
            && result.signal.is_none()
            && !result.timed_out
            && result.cleanup_complete);
    for (matches, summary) in [
        (
            command_matches,
            "execution command identity differs from selection",
        ),
        (
            identities_match,
            "execution result identity is stale or mismatched",
        ),
        (
            hashes_valid,
            "execution result contains a malformed BLAKE3 identity",
        ),
        (
            output_bounded,
            "execution output counts exceed the finite bound",
        ),
        (
            artifacts_safe,
            "execution result contains an unsafe artifact reference",
        ),
        (
            diagnostic_safe,
            "execution result contains unsafe diagnostic metadata",
        ),
        (
            passing_facts,
            "passed status contradicts process or cleanup facts",
        ),
    ] {
        if !matches {
            diagnostics.push(result_diagnostic(
                AgentCheckDiagnosticCode::ResultMismatch,
                result,
                Some(check),
                summary,
            ));
        }
    }
}

fn producer_diagnostic_is_safe(
    value: &super::AgentCheckDiagnosticDraft<'_>,
    command: &[&str],
) -> bool {
    let summary_lower = value.summary.to_ascii_lowercase();
    let source_safe = value.source.is_none_or(|source| {
        is_safe_path(source.path)
            && source.line >= ONE_BASED_MINIMUM
            && source.column >= ONE_BASED_MINIMUM
    });
    is_safe_identifier(value.remediation_label)
        && !value.summary.is_empty()
        && value.summary.chars().count() <= MAX_DIAGNOSTIC_CHARS
        && !UNSAFE_METADATA_MARKERS
            .iter()
            .any(|marker| summary_lower.contains(marker))
        && value.reproduction_argv == command
        && is_safe_argv(value.reproduction_argv)
        && value.working_root == "<repository-root>"
        && source_safe
}

fn result_diagnostic(
    code: AgentCheckDiagnosticCode,
    result: &AgentCheckExecutionDraft<'_>,
    required: Option<&super::CairnRequiredCheck>,
    summary: &str,
) -> AgentCheckDiagnostic {
    let argv = required.map_or_else(
        || result.command.to_vec(),
        |check| command_refs(check.command.as_slice()),
    );
    let mut value = diagnostic(
        code,
        DiagnosticContext::for_check(result.check_id, Some(result.status)),
        summary.to_string(),
        argv.as_slice(),
    );
    value.subject_hash = Some(result.subject_hash.to_string());
    value.registry_hash = Some(result.registry_hash.to_string());
    value.selection_hash = Some(result.selection_hash.to_string());
    value.evidence_hash = Some(result.evidence_hash.to_string());
    value.receipt_hash = Some(result.execution_receipt_hash.to_string());
    value.artifact_refs = result
        .artifacts
        .iter()
        .filter(|artifact| is_safe_path(artifact.path) && is_hash(artifact.hash))
        .map(|artifact| artifact.path.to_string())
        .collect();
    if let Some(producer) = result
        .diagnostic
        .as_ref()
        .filter(|producer| producer_diagnostic_is_safe(producer, result.command))
    {
        value.remediation_label = producer.remediation_label.to_string();
        value.summary = producer.summary.to_string();
        value.reproduction_argv = producer
            .reproduction_argv
            .iter()
            .map(|argument| (*argument).to_string())
            .collect();
        if let Some(source) = producer.source {
            value.source_path = Some(source.path.to_string());
            value.source_line = Some(source.line);
            value.source_column = Some(source.column);
        }
    }
    value
}

fn validate_cairn_summary(
    selection: &AgentCheckSelection,
    evidence: &ExecutionEvidenceReport,
    summary: &CairnCompletionSummaryDraft<'_>,
    blockers: &mut Vec<AgentCheckDiagnostic>,
) {
    let expected_checks = selection
        .required_checks
        .iter()
        .map(|check| check.check_id.as_str())
        .collect::<BTreeSet<_>>();
    let required_checks = summary
        .required_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let passed_checks = summary
        .passed_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let local_passed = evidence
        .passed_checks
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let identities_match = summary.repository_id == selection.repository_id
        && summary.registry_id == selection.registry_id
        && summary.registry_hash == selection.registry_hash
        && summary.selection_hash == selection.selection_hash
        && summary.policy_hash == selection.policy_hash
        && summary.final_subject_hash == selection.final_subject_hash;
    let shape_matches = summary.schema == CAIRN_COMPLETION_SCHEMA
        && summary.classification == AUTHORITATIVE_CLASSIFICATION
        && summary.valid
        && summary.verdict == PASS_VERDICT
        && summary.non_passing_count == 0
        && summary.diagnostic_count == 0
        && is_hash(summary.input_hash)
        && is_hash(summary.receipt_hash);
    let coverage_matches = required_checks.len() == summary.required_checks.len()
        && passed_checks.len() == summary.passed_checks.len()
        && expected_checks == required_checks
        && expected_checks == passed_checks
        && expected_checks == local_passed;
    if !identities_match || !shape_matches || !coverage_matches {
        blockers.push(diagnostic(
            AgentCheckDiagnosticCode::CairnSummaryInvalid,
            DiagnosticContext::NONE,
            "Cairn completion summary is failing, stale, malformed, or incomplete".to_string(),
            EMPTY_ARGV,
        ));
    }
}

fn completion_receipt(
    candidate: &AuthoritativeCompletionCandidate,
    selection: &AgentCheckSelection,
    evidence: &ExecutionEvidenceReport,
    cairn: &CairnCompletionSummaryDraft<'_>,
) -> AuthoritativeCompletionReceipt {
    let mut required_checks = selection
        .required_checks
        .iter()
        .map(|check| check.check_id.clone())
        .collect::<Vec<_>>();
    required_checks.sort();
    let mut passed_checks = evidence.passed_checks.clone();
    passed_checks.sort();
    let non_claims = COMPLETION_NON_CLAIMS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let required_check_text = required_checks.join(",");
    let passed_check_text = passed_checks.join(",");
    let non_claim_text = COMPLETION_NON_CLAIMS.join("|");
    let receipt_hash = hash_parts(
        COMPLETION_RECEIPT_SCHEMA,
        [
            candidate.candidate_hash.as_str(),
            selection.repository_id.as_str(),
            selection.registry_id.as_str(),
            selection.registry_hash.as_str(),
            selection.selection_hash.as_str(),
            selection.policy_hash.as_str(),
            selection.final_subject_hash.as_str(),
            evidence.evidence_hash.as_str(),
            cairn.receipt_hash,
            required_check_text.as_str(),
            passed_check_text.as_str(),
            non_claim_text.as_str(),
        ],
    );
    AuthoritativeCompletionReceipt {
        schema: COMPLETION_RECEIPT_SCHEMA.to_string(),
        candidate_hash: candidate.candidate_hash.clone(),
        repository_id: selection.repository_id.clone(),
        registry_id: selection.registry_id.clone(),
        registry_hash: selection.registry_hash.clone(),
        selection_hash: selection.selection_hash.clone(),
        policy_hash: selection.policy_hash.clone(),
        final_subject_hash: selection.final_subject_hash.clone(),
        execution_evidence_hash: evidence.evidence_hash.clone(),
        cairn_receipt_hash: cairn.receipt_hash.to_string(),
        required_checks,
        passed_checks,
        non_claims,
        authoritative_completion: true,
        receipt_hash,
    }
}

fn execution_evidence_hash(results: &[AgentCheckExecutionDraft<'_>]) -> String {
    let mut rows = results
        .iter()
        .map(|result| {
            format!(
                "{}\u{1f}{}\u{1f}{}\u{1f}{}",
                result.check_id,
                result.status.as_str(),
                result.evidence_hash,
                result.execution_receipt_hash
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    hash_parts(
        "animus.authoritative-completion.execution-evidence.v1",
        rows.iter().map(String::as_str),
    )
}

fn command_refs(command: &[String]) -> Vec<&str> {
    command.iter().map(String::as_str).collect()
}
