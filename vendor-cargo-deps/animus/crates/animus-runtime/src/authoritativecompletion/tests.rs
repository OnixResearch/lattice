// Fixture builders return typed values for many tests; assertions remain concentrated in each positive and negative case.
#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]
use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const TEST_COMMAND_HASH: &str = "108c657f5483b5a83731f413b62146dece87980abb850a1019b643bc9b37a55f";
const CLIPPY_COMMAND_HASH: &str =
    "6abfeda462ce5dc98d1aac663c2de398f51bebd75a38bdbb7d01eba912df44e8";
const CAIRN_SELECTION_HASH: &str =
    "b9da53c190687f8ff0bf2a465d6823056328363c7aa4b73f97ee0a10ff158581";
const TEST_COMMAND: &[&str] = &["cargo", "test", "--workspace"];
const CLIPPY_COMMAND: &[&str] = &[
    "cargo",
    "clippy",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
];
const CAIRN_COMMAND: &[&str] = &["nix", "run", ".#cairn", "--", "validate", "--root", "."];
const TEST_PATHS: &[&str] = &["crates"];
const CLIPPY_PATHS: &[&str] = &["crates"];
const EXPECTED_SELECTION_CHECKS: usize = 2;
const CAIRN_PATHS: &[&str] = &["cairn"];
const NO_CRATES: &[&str] = &[];
const RUNTIME_CRATE: &[&str] = &["animus-runtime"];
const TEST_SURFACES: &[&str] = &["rust"];
const CLIPPY_SURFACES: &[&str] = &["lint"];
const CAIRN_SURFACES: &[&str] = &["lifecycle"];
const NO_PREREQUISITES: &[&str] = &[];
const TEST_PREREQUISITE: &[&str] = &["tests"];
const TEST_ARTIFACTS: &[&str] = &["target/tests.json"];
const CLIPPY_ARTIFACTS: &[&str] = &["target/clippy.json"];
const CAIRN_ARTIFACTS: &[&str] = &["target/cairn.json"];
const DOCS: &[&str] = &["README.md"];
const FALLBACKS: &[&str] = &["tests", "cairn"];
const CAPABILITIES: &[CheckCapability] = &[CheckCapability::LocalProcess, CheckCapability::Cairn];
const RESULT_ARTIFACTS: &[AgentCheckArtifactDraft<'_>] = &[AgentCheckArtifactDraft {
    path: "target/check.json",
    hash: HASH_D,
}];
const SAFE_DIAGNOSTIC: AgentCheckDiagnosticDraft<'_> = AgentCheckDiagnosticDraft {
    remediation_label: "rerun-check",
    summary: "rerun the repository-owned required check",
    reproduction_argv: TEST_COMMAND,
    working_root: "<repository-root>",
    source: Some(SourceSpanDraft {
        path: "crates/animus-runtime/src/lib.rs",
        line: 1,
        column: 1,
    }),
};

fn test_row() -> AgentCheckRowDraft<'static> {
    row(
        "tests",
        TEST_COMMAND,
        TEST_PATHS,
        RUNTIME_CRATE,
        TEST_SURFACES,
        NO_PREREQUISITES,
        CheckCapability::LocalProcess,
        CheckTier::Standard,
        TEST_ARTIFACTS,
    )
}

fn clippy_row() -> AgentCheckRowDraft<'static> {
    row(
        "clippy",
        CLIPPY_COMMAND,
        CLIPPY_PATHS,
        RUNTIME_CRATE,
        CLIPPY_SURFACES,
        TEST_PREREQUISITE,
        CheckCapability::LocalProcess,
        CheckTier::Standard,
        CLIPPY_ARTIFACTS,
    )
}

fn cairn_row() -> AgentCheckRowDraft<'static> {
    row(
        "cairn",
        CAIRN_COMMAND,
        CAIRN_PATHS,
        NO_CRATES,
        CAIRN_SURFACES,
        NO_PREREQUISITES,
        CheckCapability::Cairn,
        CheckTier::Quick,
        CAIRN_ARTIFACTS,
    )
}

#[allow(clippy::too_many_arguments)]
fn row(
    check_id: &'static str,
    argv: &'static [&'static str],
    paths: &'static [&'static str],
    crates: &'static [&'static str],
    surfaces: &'static [&'static str],
    prerequisites: &'static [&'static str],
    capability: CheckCapability,
    tier: CheckTier,
    artifacts: &'static [&'static str],
) -> AgentCheckRowDraft<'static> {
    AgentCheckRowDraft {
        check_id,
        profile: check_id,
        argv,
        affected_paths: paths,
        affected_crates: crates,
        affected_surfaces: surfaces,
        prerequisites,
        capability,
        tier,
        risk: "medium",
        timeout_class: "long",
        output_budget_class: "large",
        expected_artifacts: artifacts,
        reproduction_argv: argv,
        working_root: "<repository-root>",
        documentation_refs: DOCS,
        positive_contract: "crates/animus-runtime/src/authoritativecompletion/tests.rs",
        negative_contract: "crates/animus-runtime/src/authoritativecompletion/tests.rs",
    }
}

fn registry() -> AgentCheckRegistry {
    let rows = [test_row(), clippy_row(), cairn_row()];
    validate_agent_check_registry(&registry_draft(rows.as_slice(), FALLBACKS))
        .expect("fixture registry should validate")
}

fn registry_draft<'a>(
    checks: &'a [AgentCheckRowDraft<'a>],
    fallback_check_ids: &'a [&'a str],
) -> AgentCheckRegistryDraft<'a> {
    AgentCheckRegistryDraft {
        schema: AGENT_CHECK_REGISTRY_SCHEMA,
        repository_id: "onixresearch/animus",
        registry_id: "animus-agent-checks",
        supported_cairn_schema: CAIRN_COMPLETION_SCHEMA,
        cairn_policy_hash: PINNED_CAIRN_POLICY_HASH,
        fallback_check_ids,
        checks,
    }
}

fn rust_selection() -> AgentCheckSelection {
    select_required_agent_checks(AgentCheckSelectionRequest {
        registry: &registry(),
        changed_artifacts: &[ChangedArtifactDraft {
            path: "crates/animus-runtime/src/lib.rs",
            surface: "rust",
            content_hash: HASH_A,
        }],
        available_capabilities: CAPABILITIES,
        maximum_tier: CheckTier::Standard,
        final_subject_hash: HASH_C,
    })
}

fn execution<'a>(
    selection: &'a AgentCheckSelection,
    check_id: &'static str,
    status: CheckStatus,
) -> AgentCheckExecutionDraft<'a> {
    let (command, command_hash, diagnostic) = if check_id == "tests" {
        (TEST_COMMAND, TEST_COMMAND_HASH, Some(SAFE_DIAGNOSTIC))
    } else {
        (CLIPPY_COMMAND, CLIPPY_COMMAND_HASH, None)
    };
    AgentCheckExecutionDraft {
        check_id,
        status,
        command,
        command_hash,
        subject_hash: selection.final_subject_hash(),
        registry_hash: selection.registry_hash(),
        selection_hash: selection.selection_hash(),
        policy_hash: selection.policy_hash(),
        evidence_hash: HASH_A,
        execution_receipt_hash: HASH_B,
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        cleanup_complete: true,
        stdout_bytes: 0,
        stderr_bytes: 0,
        stdout_hash: HASH_D,
        stderr_hash: HASH_E,
        artifacts: RESULT_ARTIFACTS,
        diagnostic,
    }
}

fn candidate() -> AuthoritativeCompletionCandidate {
    admit_completion_candidate(&AuthoritativeCompletionCandidateDraft {
        source: CompletionCandidateSource::SteelCompatibility,
        request: "COMPLETE",
        work_id: "work-1",
        subject_hash: HASH_C,
    })
    .expect("exact candidate should validate")
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.registry]
// r[verify agent_harness_runtime.authoritative_completion.registry.row]
// r[verify agent_harness_runtime.authoritative_completion.verification]
// r[verify agent_harness_runtime.authoritative_completion.verification.positive]
fn registry_and_cairn_hash_parity_are_deterministic() {
    let registry = registry();
    assert!(super::validation::is_hash(registry.registry_hash()));
    assert_eq!(
        cairn_command_hash(
            TEST_COMMAND
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
                .as_slice()
        ),
        TEST_COMMAND_HASH
    );
    assert_eq!(
        cairn_command_hash(
            CLIPPY_COMMAND
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
                .as_slice()
        ),
        CLIPPY_COMMAND_HASH
    );
    let selection = rust_selection();
    assert!(selection.is_ready(), "{:?}", selection.blockers());
    assert_eq!(selection.selection_hash(), CAIRN_SELECTION_HASH);
    assert_eq!(selection.required_checks().len(), EXPECTED_SELECTION_CHECKS);
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.verification.negative]
// r[verify agent_harness_runtime.authoritative_completion.registry.drift]
// r[verify agent_harness_runtime.authoritative_completion.registry.unsafe_command]
fn registry_rejects_duplicate_unknown_cycle_unsafe_and_stale_rows() {
    let duplicate_rows = [test_row(), test_row()];
    assert!(validate_agent_check_registry(&registry_draft(&duplicate_rows, FALLBACKS)).is_err());

    let mut unknown = clippy_row();
    unknown.prerequisites = &["missing"];
    let unknown_rows = [test_row(), unknown, cairn_row()];
    assert!(validate_agent_check_registry(&registry_draft(&unknown_rows, FALLBACKS)).is_err());

    let mut first = test_row();
    first.prerequisites = &["clippy"];
    let cycle_rows = [first, clippy_row(), cairn_row()];
    assert!(validate_agent_check_registry(&registry_draft(&cycle_rows, FALLBACKS)).is_err());

    let mut unsafe_row = test_row();
    unsafe_row.argv = &["bash", "-c", "cargo test"];
    unsafe_row.reproduction_argv = unsafe_row.argv;
    let unsafe_rows = [unsafe_row, clippy_row(), cairn_row()];
    assert!(validate_agent_check_registry(&registry_draft(&unsafe_rows, FALLBACKS)).is_err());

    let rows = [test_row(), clippy_row(), cairn_row()];
    let mut stale = registry_draft(&rows, FALLBACKS);
    for rejected_policy in [
        HASH_A,
        "2c0b44345b9d2e2b3eff5fa7db7a14b9681c6bfcf7a4bb651f13bcadc1f91757",
    ] {
        stale.cairn_policy_hash = rejected_policy;
        assert!(validate_agent_check_registry(&stale).is_err());
    }
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.selection]
// r[verify agent_harness_runtime.authoritative_completion.selection.direct]
// r[verify agent_harness_runtime.authoritative_completion.selection.unknown]
// r[verify agent_harness_runtime.authoritative_completion.selection.pure]
fn selector_adds_prerequisites_and_uses_conservative_unknown_fallback() {
    let direct = rust_selection();
    assert_eq!(
        direct
            .required_checks()
            .iter()
            .map(CairnRequiredCheck::check_id)
            .collect::<Vec<_>>(),
        vec!["tests", "clippy"]
    );
    let fallback = select_required_agent_checks(AgentCheckSelectionRequest {
        registry: &registry(),
        changed_artifacts: &[ChangedArtifactDraft {
            path: "unknown/new.surface",
            surface: "unknown",
            content_hash: HASH_A,
        }],
        available_capabilities: CAPABILITIES,
        maximum_tier: CheckTier::Standard,
        final_subject_hash: HASH_C,
    });
    assert!(fallback.is_ready());
    assert_eq!(
        fallback
            .required_checks()
            .iter()
            .map(CairnRequiredCheck::check_id)
            .collect::<Vec<_>>(),
        vec!["cairn", "tests"]
    );
}

#[test]
fn selector_keeps_unavailable_capability_and_tier_as_blockers() {
    let capability = select_required_agent_checks(AgentCheckSelectionRequest {
        registry: &registry(),
        changed_artifacts: &[ChangedArtifactDraft {
            path: ".cairn/specs/agent-harness-runtime/spec.md",
            surface: "lifecycle",
            content_hash: HASH_A,
        }],
        available_capabilities: &[CheckCapability::LocalProcess],
        maximum_tier: CheckTier::Full,
        final_subject_hash: HASH_C,
    });
    assert!(!capability.is_ready());
    assert!(
        capability
            .blockers()
            .iter()
            .any(|issue| { issue.code() == AgentCheckDiagnosticCode::CapabilityUnavailable })
    );
    let tier = select_required_agent_checks(AgentCheckSelectionRequest {
        registry: &registry(),
        changed_artifacts: &[ChangedArtifactDraft {
            path: "crates/animus-runtime/src/lib.rs",
            surface: "rust",
            content_hash: HASH_A,
        }],
        available_capabilities: CAPABILITIES,
        maximum_tier: CheckTier::Quick,
        final_subject_hash: HASH_C,
    });
    assert!(!tier.is_ready());
    assert!(
        tier.blockers()
            .iter()
            .any(|issue| issue.code() == AgentCheckDiagnosticCode::TierUnavailable)
    );
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.execution_receipts]
// r[verify agent_harness_runtime.authoritative_completion.execution_receipts.passed]
// r[verify agent_harness_runtime.authoritative_completion.pass_only]
// r[verify agent_harness_runtime.authoritative_completion.pass_only.complete]
fn exact_execution_coverage_passes_only_when_every_required_row_passes() {
    let selection = rust_selection();
    let report = admit_execution_results(
        &selection,
        &[
            execution(&selection, "tests", CheckStatus::Passed),
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    assert!(report.is_passing(), "{:?}", report.diagnostics());
    assert_eq!(report.passed_checks(), &["clippy", "tests"]);
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.execution_receipts.failure]
// r[verify agent_harness_runtime.authoritative_completion.pass_only.skipped]
// r[verify agent_harness_runtime.authoritative_completion.pass_only.dry_run]
// r[verify agent_harness_runtime.authoritative_completion.feedback.failed]
fn every_non_passing_status_remains_a_completion_blocker() {
    let statuses = [
        CheckStatus::Failed,
        CheckStatus::Blocked,
        CheckStatus::Skipped,
        CheckStatus::Deferred,
        CheckStatus::TimedOut,
        CheckStatus::Cancelled,
        CheckStatus::DryRun,
        CheckStatus::CleanupFailed,
        CheckStatus::InfrastructureError,
        CheckStatus::NotSelected,
        CheckStatus::Unknown,
    ];
    for status in statuses {
        let selection = rust_selection();
        let report = admit_execution_results(
            &selection,
            &[
                execution(&selection, "tests", status),
                execution(&selection, "clippy", CheckStatus::Passed),
            ],
        );
        assert!(!report.is_passing(), "status {} passed", status.as_str());
        assert!(
            report
                .non_passing_checks()
                .iter()
                .any(|(check, actual)| check == "tests" && *actual == status)
        );
    }
}

#[test]
fn missing_duplicate_unknown_stale_and_contradictory_results_fail_closed() {
    let selection = rust_selection();
    let missing = admit_execution_results(
        &selection,
        &[execution(&selection, "tests", CheckStatus::Passed)],
    );
    assert!(!missing.is_passing());

    let test = execution(&selection, "tests", CheckStatus::Passed);
    let duplicate = admit_execution_results(
        &selection,
        &[
            test,
            test,
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    assert!(!duplicate.is_passing());

    let unknown = execution(&selection, "unknown", CheckStatus::Passed);
    let unknown_report = admit_execution_results(
        &selection,
        &[
            execution(&selection, "tests", CheckStatus::Passed),
            execution(&selection, "clippy", CheckStatus::Passed),
            unknown,
        ],
    );
    assert!(!unknown_report.is_passing());

    let mut stale = execution(&selection, "tests", CheckStatus::Passed);
    stale.subject_hash = HASH_D;
    let stale_report = admit_execution_results(
        &selection,
        &[stale, execution(&selection, "clippy", CheckStatus::Passed)],
    );
    assert!(!stale_report.is_passing());

    let mut contradiction = execution(&selection, "tests", CheckStatus::Passed);
    contradiction.exit_code = Some(1);
    let contradiction_report = admit_execution_results(
        &selection,
        &[
            contradiction,
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    assert!(!contradiction_report.is_passing());
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.feedback.safe]
fn unsafe_artifact_and_diagnostic_payloads_are_rejected() {
    const UNSAFE_ARTIFACTS: &[AgentCheckArtifactDraft<'_>] = &[AgentCheckArtifactDraft {
        path: "/home/operator/raw.log",
        hash: HASH_A,
    }];
    const UNSAFE_DIAGNOSTIC: AgentCheckDiagnosticDraft<'_> = AgentCheckDiagnosticDraft {
        remediation_label: "rerun-check",
        summary: "raw prompt and credential follow",
        reproduction_argv: TEST_COMMAND,
        working_root: "<repository-root>",
        source: None,
    };
    let selection = rust_selection();
    let mut result = execution(&selection, "tests", CheckStatus::Passed);
    result.artifacts = UNSAFE_ARTIFACTS;
    result.diagnostic = Some(UNSAFE_DIAGNOSTIC);
    let report = admit_execution_results(
        &selection,
        &[result, execution(&selection, "clippy", CheckStatus::Passed)],
    );
    assert!(!report.is_passing());
    assert!(report.diagnostics().iter().all(|issue| {
        !issue.summary().contains("credential") && !issue.summary().contains("raw prompt")
    }));
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.candidate]
// r[verify agent_harness_runtime.authoritative_completion.candidate.exact]
// r[verify agent_harness_runtime.authoritative_completion.candidate.substring]
fn exact_completion_candidate_is_advisory_and_incidental_substrings_fail() {
    let exact = candidate();
    assert_eq!(exact.request(), "COMPLETE");
    assert_eq!(exact.subject_hash(), HASH_C);
    for request in [
        "not COMPLETE",
        "COMPLETE now",
        "quoted COMPLETE",
        "INCOMPLETE",
    ] {
        let rejected = admit_completion_candidate(&AuthoritativeCompletionCandidateDraft {
            source: CompletionCandidateSource::SteelCompatibility,
            request,
            work_id: "work-1",
            subject_hash: HASH_C,
        });
        assert!(rejected.is_err(), "incidental request passed: {request}");
    }
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.receipt_gate]
// r[verify agent_harness_runtime.authoritative_completion.receipt_gate.pass]
// r[verify agent_harness_runtime.authoritative_completion.receipt_gate.host_adapter]
fn matching_cairn_summary_is_the_only_verified_completion_path() {
    let selection = rust_selection();
    let evidence = admit_execution_results(
        &selection,
        &[
            execution(&selection, "tests", CheckStatus::Passed),
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    let required = ["tests", "clippy"];
    let passed = ["clippy", "tests"];
    let summary = CairnCompletionSummaryDraft {
        schema: CAIRN_COMPLETION_SCHEMA,
        classification: "authoritative",
        repository_id: selection.repository_id(),
        registry_id: selection.registry_id(),
        registry_hash: selection.registry_hash(),
        selection_hash: selection.selection_hash(),
        policy_hash: selection.policy_hash(),
        final_subject_hash: selection.final_subject_hash(),
        required_checks: &required,
        passed_checks: &passed,
        non_passing_count: 0,
        diagnostic_count: 0,
        valid: true,
        verdict: "PASS",
        input_hash: HASH_D,
        receipt_hash: HASH_E,
    };
    let outcome =
        evaluate_authoritative_completion(&candidate(), &selection, &evidence, Some(&summary));
    let AuthoritativeCompletionOutcome::Verified(receipt) = outcome else {
        panic!("matching summary should verify");
    };
    assert!(receipt.authoritative_completion());
    assert_eq!(receipt.cairn_receipt_hash(), HASH_E);
    assert!(super::validation::is_hash(receipt.receipt_hash()));
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.candidate.self_attestation]
// r[verify agent_harness_runtime.authoritative_completion.receipt_gate.missing]
// r[verify agent_harness_runtime.authoritative_completion.boundary.no_override]
fn missing_stale_failing_and_incomplete_cairn_summaries_block() {
    let selection = rust_selection();
    let evidence = admit_execution_results(
        &selection,
        &[
            execution(&selection, "tests", CheckStatus::Passed),
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    assert!(
        !evaluate_authoritative_completion(&candidate(), &selection, &evidence, None).is_verified()
    );
    let required = ["tests", "clippy"];
    let passed = ["tests"];
    let stale = CairnCompletionSummaryDraft {
        schema: CAIRN_COMPLETION_SCHEMA,
        classification: "authoritative",
        repository_id: selection.repository_id(),
        registry_id: selection.registry_id(),
        registry_hash: selection.registry_hash(),
        selection_hash: selection.selection_hash(),
        policy_hash: selection.policy_hash(),
        final_subject_hash: HASH_D,
        required_checks: &required,
        passed_checks: &passed,
        non_passing_count: 1,
        diagnostic_count: 1,
        valid: false,
        verdict: "FAIL",
        input_hash: HASH_D,
        receipt_hash: HASH_E,
    };
    assert!(
        !evaluate_authoritative_completion(&candidate(), &selection, &evidence, Some(&stale))
            .is_verified()
    );
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.execution_receipts.stale]
fn post_check_mutation_invalidates_candidate_results_and_cairn_summary() {
    let selection = rust_selection();
    let changed_selection = select_required_agent_checks(AgentCheckSelectionRequest {
        registry: &registry(),
        changed_artifacts: &[ChangedArtifactDraft {
            path: "crates/animus-runtime/src/lib.rs",
            surface: "rust",
            content_hash: HASH_B,
        }],
        available_capabilities: CAPABILITIES,
        maximum_tier: CheckTier::Standard,
        final_subject_hash: HASH_D,
    });
    let stale_evidence = admit_execution_results(
        &changed_selection,
        &[
            execution(&selection, "tests", CheckStatus::Passed),
            execution(&selection, "clippy", CheckStatus::Passed),
        ],
    );
    assert!(!stale_evidence.is_passing());
    assert!(!evaluate_authoritative_completion(
        &candidate(),
        &changed_selection,
        &stale_evidence,
        None,
    )
    .is_verified());
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.workspace_supervisor]
// r[verify agent_harness_runtime.authoritative_completion.workspace_supervisor.progress]
// r[verify agent_harness_runtime.authoritative_completion.workspace_supervisor.archive]
// r[verify agent_harness_runtime.authoritative_completion.workspace_supervisor.non_authority]
fn workspace_progress_and_archive_observations_have_no_completion_authority() {
    let selection = rust_selection();
    let incomplete_evidence = admit_execution_results(&selection, &[]);
    let outcome =
        evaluate_authoritative_completion(&candidate(), &selection, &incomplete_evidence, None);

    assert!(!outcome.is_verified());
    assert!(!incomplete_evidence.is_passing());
    assert!(PROHIBITED_AUTHORITATIVE_COMPLETION_EFFECTS.contains(&"workspace mutation"));
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.boundary]
// r[verify agent_harness_runtime.authoritative_completion.boundary.pure]
// r[verify agent_harness_runtime.authoritative_completion.verification.non_claims]
fn pure_core_declares_no_host_effects() {
    assert!(PROHIBITED_AUTHORITATIVE_COMPLETION_EFFECTS.contains(&"filesystem read"));
    assert!(PROHIBITED_AUTHORITATIVE_COMPLETION_EFFECTS.contains(&"process execution"));
    assert!(PROHIBITED_AUTHORITATIVE_COMPLETION_EFFECTS.contains(&"workspace mutation"));
}
