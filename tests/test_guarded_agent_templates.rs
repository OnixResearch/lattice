use lattice::guarded::core::BlockReason;
use lattice::guarded::core::MetricFact;
use lattice::guarded::core::ObservationPhase;
use lattice::guarded::core::ProgressDecision;
use lattice::guarded::core::ProgressObservation;
use lattice::guarded::core::ProgressPolicy;
use lattice::guarded::core::ProgressPrerequisites;

const ORIGINAL_POLICY: &str = "[octet]\n\n[octet.lint_levels]\nmagic_number = \"warn\"\n";
const NO_DISABLED_POLICY: &str = "[octet]\ndisabled_lints = []\n";
const ARTIFACT_PATH: &str = "target/guarded-test/probe";
const FIRST_PROBE_SLOT: &str = "probe-000";
const SECOND_PROBE_SLOT: &str = "probe-001";
const TEST_TIMEOUT_SECS: u64 = 30;
const BEFORE_FINDINGS: &str = "8";
const AFTER_FINDINGS: &str = "6";
const OVERSIZED_SUMMARY_BYTES: usize = 1_048_577;
const ABOVE_MAXIMUM_TIMEOUT_SECS: u64 = 3_601;

fn observation(before: &[&str], after: &[&str], prerequisites: ProgressPrerequisites) -> ProgressObservation {
    let mut metrics = before
        .iter()
        .map(|value| MetricFact {
            phase: ObservationPhase::Before,
            value: (*value).to_owned(),
        })
        .collect::<Vec<_>>();
    metrics.extend(after.iter().map(|value| MetricFact {
        phase: ObservationPhase::After,
        value: (*value).to_owned(),
    }));
    ProgressObservation { metrics, prerequisites }
}

// r[verify lattice.guarded_agent_templates.progress.scenario.improved]
#[test]
fn strict_decrease_produces_only_an_inert_approved_successor() {
    let decision = lattice::guarded::core::decide_progress(
        ProgressPolicy::decreasing(2).expect("positive policy"),
        &observation(&[BEFORE_FINDINGS], &[AFTER_FINDINGS], ProgressPrerequisites::standard()),
    );

    assert_eq!(decision, ProgressDecision::Improved {
        successor: lattice::guarded::core::SuccessorProposal {
            before: 8,
            after: 6,
            delta: 2,
            requires_operator_approval: true,
        },
    });
}

// r[verify lattice.guarded_agent_templates.progress.scenario.blocked]
#[test]
fn equality_regression_and_insufficient_delta_fail_closed() {
    let policy = ProgressPolicy::decreasing(2).expect("positive policy");
    let cases = [
        ("8", "8", BlockReason::InsufficientDelta),
        ("8", "9", BlockReason::MetricRegression),
        ("8", "7", BlockReason::InsufficientDelta),
    ];

    for (before, after, reason) in cases {
        assert_eq!(
            lattice::guarded::core::decide_progress(
                policy,
                &observation(&[before], &[after], ProgressPrerequisites::standard()),
            ),
            ProgressDecision::Blocked { reason }
        );
    }
}

// r[verify lattice.guarded_agent_templates.progress.scenario.blocked]
#[test]
fn malformed_missing_overflowing_and_duplicate_metrics_fail_closed() {
    let policy = ProgressPolicy::decreasing(1).expect("positive policy");
    let cases = [
        (vec![], vec!["1"], BlockReason::MissingBeforeMetric),
        (vec!["2"], vec![], BlockReason::MissingAfterMetric),
        (vec!["2", "3"], vec!["1"], BlockReason::DuplicateBeforeMetric),
        (vec!["2"], vec!["1", "0"], BlockReason::DuplicateAfterMetric),
        (vec!["two"], vec!["1"], BlockReason::MalformedMetric),
        (vec!["18446744073709551616"], vec!["1"], BlockReason::MalformedMetric),
    ];

    for (before, after, reason) in cases {
        assert_eq!(
            lattice::guarded::core::decide_progress(
                policy,
                &observation(&before, &after, ProgressPrerequisites::standard()),
            ),
            ProgressDecision::Blocked { reason }
        );
    }
}

// r[verify lattice.guarded_agent_templates.progress.scenario.blocked]
#[test]
fn failed_validation_and_restoration_prerequisites_fail_closed() {
    let policy = ProgressPolicy::decreasing(1).expect("positive policy");
    let failed_validation = ProgressPrerequisites {
        validation_passed: false,
        ..ProgressPrerequisites::standard()
    };
    let failed_restoration = ProgressPrerequisites {
        restoration_required: true,
        restoration_confirmed: false,
        ..ProgressPrerequisites::standard()
    };

    assert_eq!(
        lattice::guarded::core::decide_progress(policy, &observation(&["2"], &["1"], failed_validation),),
        ProgressDecision::Blocked {
            reason: BlockReason::ValidationFailed,
        }
    );
    assert_eq!(
        lattice::guarded::core::decide_progress(policy, &observation(&["2"], &["1"], failed_restoration),),
        ProgressDecision::Blocked {
            reason: BlockReason::RestorationUnconfirmed,
        }
    );
}

// r[verify lattice.guarded_agent_templates.cairn.scenario.drained]
// r[verify lattice.guarded_agent_templates.cairn.scenario.blocked]
#[test]
fn cairn_selection_is_sorted_and_requires_the_selected_change_to_disappear() {
    let before = lattice::guarded::core::make_cairn_snapshot(vec!["z-last".to_owned(), "a-first".to_owned()], None)
        .expect("valid before snapshot");
    let after =
        lattice::guarded::core::make_cairn_snapshot(vec!["z-last".to_owned()], None).expect("valid after snapshot");

    assert_eq!(before.selected.as_deref(), Some("a-first"));
    assert!(matches!(
        lattice::guarded::core::decide_cairn_progress(
            ProgressPolicy::decreasing(1).expect("positive policy"),
            &before,
            &after,
            true,
        ),
        ProgressDecision::Improved { .. }
    ));

    let retained = lattice::guarded::core::make_cairn_snapshot(vec!["a-first".to_owned(), "z-last".to_owned()], None)
        .expect("valid retained snapshot");
    assert_eq!(
        lattice::guarded::core::decide_cairn_progress(
            ProgressPolicy::decreasing(1).expect("positive policy"),
            &before,
            &retained,
            true,
        ),
        ProgressDecision::Blocked {
            reason: BlockReason::SelectedChangeRetained,
        }
    );
}

// r[verify lattice.guarded_agent_templates.cairn.scenario.empty]
#[test]
fn empty_cairn_snapshot_completes_without_a_successor() {
    let empty = lattice::guarded::core::make_cairn_snapshot(Vec::new(), None).expect("empty snapshot");

    assert_eq!(
        lattice::guarded::core::decide_cairn_progress(
            ProgressPolicy::decreasing(1).expect("positive policy"),
            &empty,
            &empty,
            true,
        ),
        ProgressDecision::Complete { before: 0, delta: 0 }
    );
}

struct SuccessfulProbe {
    findings: &'static str,
    mutate_policy: bool,
}

impl lattice::guarded::shell::ProbeProcess for SuccessfulProbe {
    fn run(
        &self,
        request: &lattice::agent_node::ProcessRequest,
    ) -> Result<lattice::agent_node::ProcessOutput, lattice::guarded::shell::Error> {
        let cwd = std::path::Path::new(request.cwd.as_deref().expect("probe cwd"));
        assert_eq!(std::fs::read_to_string(cwd.join("dylint.toml")).expect("temporary policy"), NO_DISABLED_POLICY);
        let artifact = request.args.last().expect("artifact argument");
        let artifact_dir = cwd.join(artifact);
        std::fs::create_dir_all(&artifact_dir).expect("artifact directory");
        std::fs::write(artifact_dir.join("summary.txt"), format!("Status: accepted\nFindings: {}\n", self.findings))
            .expect("summary fixture");
        if self.mutate_policy {
            std::fs::write(cwd.join("dylint.toml"), "operator-concurrent-edit\n").expect("concurrent policy fixture");
        }
        Ok(lattice::agent_node::ProcessOutput {
            status_code: Some(0),
            stdout: b"bounded probe stdout".to_vec(),
            stderr: Vec::new(),
        })
    }
}

struct FailingProbe;

impl lattice::guarded::shell::ProbeProcess for FailingProbe {
    fn run(
        &self,
        _request: &lattice::agent_node::ProcessRequest,
    ) -> Result<lattice::agent_node::ProcessOutput, lattice::guarded::shell::Error> {
        Err(lattice::guarded::shell::Error::ProcessFailed)
    }
}

struct RawSummaryProbe<'a> {
    summary: &'a [u8],
}

impl lattice::guarded::shell::ProbeProcess for RawSummaryProbe<'_> {
    fn run(
        &self,
        request: &lattice::agent_node::ProcessRequest,
    ) -> Result<lattice::agent_node::ProcessOutput, lattice::guarded::shell::Error> {
        let cwd = std::path::Path::new(request.cwd.as_deref().expect("probe cwd"));
        let artifact = request.args.last().expect("artifact argument");
        let artifact_dir = cwd.join(artifact);
        std::fs::create_dir_all(&artifact_dir).expect("artifact directory");
        std::fs::write(artifact_dir.join("summary.txt"), self.summary).expect("raw summary fixture");
        Ok(lattice::agent_node::ProcessOutput {
            status_code: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }
}

// r[verify lattice.guarded_agent_templates.octet.scenario.reduced]
#[test]
fn octet_probe_restores_exact_policy_bytes_and_emits_a_bounded_snapshot() {
    let root = tempfile::tempdir().expect("temporary root");
    std::fs::write(root.path().join("dylint.toml"), ORIGINAL_POLICY).expect("policy fixture");

    let snapshot = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &SuccessfulProbe {
            findings: BEFORE_FINDINGS,
            mutate_policy: false,
        },
    )
    .expect("successful probe");

    assert_eq!(snapshot.findings, BEFORE_FINDINGS);
    assert!(snapshot.restoration_confirmed);
    assert_eq!(std::fs::read_to_string(root.path().join("dylint.toml")).expect("restored policy"), ORIGINAL_POLICY);
    assert!(!root.path().join(ARTIFACT_PATH).join(FIRST_PROBE_SLOT).join("policy-backup.toml").exists());

    let second = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &SuccessfulProbe {
            findings: AFTER_FINDINGS,
            mutate_policy: false,
        },
    )
    .expect("second bounded probe");
    assert_eq!(second.findings, AFTER_FINDINGS);
    assert!(root.path().join(ARTIFACT_PATH).join(SECOND_PROBE_SLOT).join("summary.txt").is_file());
}

// r[verify lattice.guarded_agent_templates.octet.scenario.non-improving]
#[test]
fn process_failure_restores_policy_and_cleans_observed_recovery_state() {
    let root = tempfile::tempdir().expect("temporary root");
    std::fs::write(root.path().join("dylint.toml"), ORIGINAL_POLICY).expect("policy fixture");

    let error = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &FailingProbe,
    )
    .expect_err("probe must fail");

    assert_eq!(error, lattice::guarded::shell::Error::ProcessFailed);
    assert_eq!(std::fs::read_to_string(root.path().join("dylint.toml")).expect("restored policy"), ORIGINAL_POLICY);
    assert!(!root.path().join(ARTIFACT_PATH).join(FIRST_PROBE_SLOT).join("policy-backup.toml").exists());
}

// r[verify lattice.guarded_agent_templates.octet.scenario.non-improving]
#[test]
fn malformed_truncated_and_out_of_range_probes_fail_closed_after_restoration() {
    let root = tempfile::tempdir().expect("temporary root");
    std::fs::write(root.path().join("dylint.toml"), ORIGINAL_POLICY).expect("policy fixture");

    let malformed = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &RawSummaryProbe {
            summary: b"Findings: not-a-number\n",
        },
    )
    .expect_err("malformed summary must fail");
    assert_eq!(malformed, lattice::guarded::shell::Error::MalformedObservation);
    assert_eq!(std::fs::read_to_string(root.path().join("dylint.toml")).expect("restored policy"), ORIGINAL_POLICY);

    let oversized = vec![b'x'; OVERSIZED_SUMMARY_BYTES];
    let truncated = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &RawSummaryProbe { summary: &oversized },
    )
    .expect_err("oversized summary must fail");
    assert_eq!(truncated, lattice::guarded::shell::Error::LimitExceeded);
    assert_eq!(std::fs::read_to_string(root.path().join("dylint.toml")).expect("restored policy"), ORIGINAL_POLICY);

    for timeout_secs in [0, ABOVE_MAXIMUM_TIMEOUT_SECS] {
        assert_eq!(
            lattice::guarded::shell::probe_octet_with(
                root.path(),
                std::path::Path::new(ARTIFACT_PATH),
                timeout_secs,
                &FailingProbe,
            ),
            Err(lattice::guarded::shell::Error::LimitExceeded)
        );
    }
}

// r[verify lattice.guarded_agent_templates.octet.scenario.restoration]
#[test]
fn concurrent_policy_change_blocks_recovery_without_overwriting_operator_bytes() {
    let root = tempfile::tempdir().expect("temporary root");
    std::fs::write(root.path().join("dylint.toml"), ORIGINAL_POLICY).expect("policy fixture");

    let error = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &SuccessfulProbe {
            findings: AFTER_FINDINGS,
            mutate_policy: true,
        },
    )
    .expect_err("recovery must block");

    assert_eq!(error, lattice::guarded::shell::Error::RecoveryBlocked);
    assert_eq!(
        std::fs::read_to_string(root.path().join("dylint.toml")).expect("operator policy"),
        "operator-concurrent-edit\n"
    );
    assert!(root.path().join(ARTIFACT_PATH).join(FIRST_PROBE_SLOT).join("policy-backup.toml").exists());
}

#[test]
fn preexisting_recovery_state_blocks_before_policy_replacement() {
    let root = tempfile::tempdir().expect("temporary root");
    std::fs::write(root.path().join("dylint.toml"), ORIGINAL_POLICY).expect("policy fixture");
    let artifact_dir = root.path().join(ARTIFACT_PATH).join(FIRST_PROBE_SLOT);
    std::fs::create_dir_all(&artifact_dir).expect("artifact fixture");
    std::fs::write(artifact_dir.join("policy-backup.toml"), ORIGINAL_POLICY).expect("backup fixture");

    let error = lattice::guarded::shell::probe_octet_with(
        root.path(),
        std::path::Path::new(ARTIFACT_PATH),
        TEST_TIMEOUT_SECS,
        &FailingProbe,
    )
    .expect_err("recovery state must block");

    assert_eq!(error, lattice::guarded::shell::Error::RecoveryBlocked);
    assert_eq!(std::fs::read_to_string(root.path().join("dylint.toml")).expect("unchanged policy"), ORIGINAL_POLICY);
}

// r[verify lattice.guarded_agent_templates.rounds.scenario.one-agent]
// r[verify lattice.guarded_agent_templates.authority.scenario.approved]
// r[verify lattice.guarded_agent_templates.evidence.scenario.status]
// r[verify lattice.guarded_agent_templates.evidence.scenario.non-claim]
#[test]
fn built_in_guarded_bundles_load_without_model_calls_and_bind_exact_authority() {
    let catalog = lattice::workflow_template::load_catalog(std::path::Path::new("builtins/workflow-templates"))
        .expect("built-in catalog");
    assert!(catalog.diagnostics.is_empty(), "guarded bundle diagnostics: {:?}", catalog.diagnostics);

    for id in [
        lattice::guarded::authority::CAIRN_BUNDLE_ID,
        lattice::guarded::authority::OCTET_BUNDLE_ID,
    ] {
        let bundle = catalog.bundles.iter().find(|bundle| bundle.manifest.id == id).expect("guarded bundle");
        assert!(
            bundle
                .manifest
                .compatibility
                .feature_ids
                .iter()
                .any(|feature| feature == lattice::guarded::authority::GUARDED_FEATURE_ID)
        );
        lattice::guarded::authority::validate_bundle(id, &bundle.workflow, bundle.orchestration.as_ref())
            .expect("guarded authority");
        assert_eq!(
            bundle
                .workflow
                .configs
                .values()
                .filter(|config| matches!(config, lattice::engine::workflow::NodeConfig::Agent(_)))
                .count(),
            1
        );
        assert!(bundle.manifest.non_claims.iter().any(|claim| claim.contains("exact reviewed local bytes")));
    }
}

// r[verify lattice.guarded_agent_templates.rounds.scenario.hidden-loop]
// r[verify lattice.guarded_agent_templates.authority.scenario.denied]
// r[verify lattice.guarded_agent_templates.authority.scenario.budget]
#[test]
fn guarded_authority_rejects_second_agents_fallbacks_commands_retries_and_budget_expansion() {
    let catalog = lattice::workflow_template::load_catalog(std::path::Path::new("builtins/workflow-templates"))
        .expect("built-in catalog");
    let bundle = catalog
        .bundles
        .iter()
        .find(|bundle| bundle.manifest.id == lattice::guarded::authority::OCTET_BUNDLE_ID)
        .expect("Octet bundle");
    let orchestration = bundle.orchestration.as_ref().expect("Octet orchestration");

    let mut second_agent = bundle.workflow.clone();
    second_agent.add_agent_node(
        "prohibited second agent",
        lattice::agent_node::AgentTarget::Pi,
        "do more work",
        lattice::agent_node::AgentOutputMode::Text,
    );
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &second_agent,
            Some(orchestration),
        ),
        Err(lattice::guarded::authority::Error::InvalidAgent)
    );

    let mut fallback = bundle.workflow.clone();
    let agent_id = fallback
        .configs
        .iter()
        .find_map(|(id, config)| matches!(config, lattice::engine::workflow::NodeConfig::Agent(_)).then_some(*id))
        .expect("agent identity");
    let lattice::engine::workflow::NodeConfig::Agent(settings) =
        fallback.configs.get_mut(&agent_id).expect("agent config")
    else {
        panic!("agent config kind");
    };
    settings.target = lattice::agent_node::AgentTarget::Clankers;
    settings.model_hint = Some("fallback-model".to_owned());
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &fallback,
            Some(orchestration),
        ),
        Err(lattice::guarded::authority::Error::InvalidAgent)
    );

    let mut hidden_push = bundle.workflow.clone();
    let shell = hidden_push
        .configs
        .values_mut()
        .find(|config| matches!(config, lattice::engine::workflow::NodeConfig::Shell { .. }))
        .expect("shell config");
    let lattice::engine::workflow::NodeConfig::Shell { command, .. } = shell else {
        panic!("shell config kind");
    };
    *command = "while true; do git commit -am hidden && git push; done".to_owned();
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &hidden_push,
            Some(orchestration),
        ),
        Err(lattice::guarded::authority::Error::InvalidNodeShape)
    );

    let mut retry = orchestration.clone();
    retry.retry = Some(lattice::orchestration::RetryPolicy {
        max_observed_failure_retries: 1,
    });
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &bundle.workflow,
            Some(&retry),
        ),
        Err(lattice::guarded::authority::Error::AutomaticRetry)
    );

    let mut expanded = orchestration.clone();
    expanded.budgets.per_round_upper_bound.agent_invocations = 2;
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &bundle.workflow,
            Some(&expanded),
        ),
        Err(lattice::guarded::authority::Error::InvalidBudget)
    );
}

// r[verify lattice.guarded_agent_templates.authority.scenario.denied]
// r[verify lattice.guarded_agent_templates.authority.scenario.budget]
#[test]
fn guarded_authority_rejects_credentials_roots_self_approval_and_missing_positive_budgets() {
    let catalog = lattice::workflow_template::load_catalog(std::path::Path::new("builtins/workflow-templates"))
        .expect("built-in catalog");
    let bundle = catalog
        .bundles
        .iter()
        .find(|bundle| bundle.manifest.id == lattice::guarded::authority::OCTET_BUNDLE_ID)
        .expect("Octet bundle");
    let orchestration = bundle.orchestration.as_ref().expect("Octet orchestration");

    let mut credentialed = bundle.workflow.clone();
    credentialed.credentials.insert(0, lattice::credentials::NodeCredentialRefs::default());
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &credentialed,
            Some(orchestration),
        ),
        Err(lattice::guarded::authority::Error::InvalidTrigger)
    );

    let mut broader_root = bundle.workflow.clone();
    let shell = broader_root
        .configs
        .values_mut()
        .find(|config| matches!(config, lattice::engine::workflow::NodeConfig::Shell { .. }))
        .expect("shell config");
    let lattice::engine::workflow::NodeConfig::Shell { cwd, .. } = shell else {
        panic!("shell config kind");
    };
    *cwd = Some("..".to_owned());
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &broader_root,
            Some(orchestration),
        ),
        Err(lattice::guarded::authority::Error::InvalidCommand)
    );

    let mut self_approved = orchestration.clone();
    self_approved.authority.approvers = std::collections::BTreeSet::from(["agent:self".to_owned()]);
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &bundle.workflow,
            Some(&self_approved),
        ),
        Err(lattice::guarded::authority::Error::InvalidAuthority)
    );

    let mut missing_budget = orchestration.clone();
    missing_budget.budgets.limits.tokens = 0;
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &bundle.workflow,
            Some(&missing_budget),
        ),
        Err(lattice::guarded::authority::Error::InvalidBudget)
    );

    let mut monetary = orchestration.clone();
    monetary.budgets.limits.monetary_microunits = 1;
    assert_eq!(
        lattice::guarded::authority::validate_bundle(
            lattice::guarded::authority::OCTET_BUNDLE_ID,
            &bundle.workflow,
            Some(&monetary),
        ),
        Err(lattice::guarded::authority::Error::InvalidBudget)
    );
}

// r[verify lattice.guarded_agent_templates.cairn.scenario.empty]
#[test]
fn empty_cairn_observation_selects_no_work_branch_and_therefore_no_agent() {
    let empty = lattice::guarded::core::make_cairn_snapshot(Vec::new(), None).expect("empty snapshot");
    let value = lattice::engine::Value::from_string(serde_json::to_string(&empty).expect("snapshot JSON"));
    let condition = lattice::control_flow::ConditionNodeConfig {
        branches: vec![lattice::control_flow::ConditionBranch {
            label: "work".to_owned(),
            predicate: lattice::control_flow::ConditionPredicate::StringContains("\"has_work\":true".to_owned()),
        }],
    };

    let selected_agent_routes = lattice::control_flow::evaluate_condition(&condition, Some(&value));

    assert!(selected_agent_routes.is_empty());
}

#[test]
fn cairn_discovery_is_bounded_sorted_and_rejects_symlinked_changes() {
    let root = tempfile::tempdir().expect("temporary root");
    let changes = root.path().join(".cairn/changes");
    std::fs::create_dir_all(changes.join("z-last")).expect("last change fixture");
    std::fs::create_dir_all(changes.join("a-first")).expect("first change fixture");

    let snapshot = lattice::guarded::shell::discover_cairn(root.path(), None).expect("discovery");

    assert_eq!(snapshot.active_changes, ["a-first", "z-last"]);
    assert_eq!(snapshot.selected.as_deref(), Some("a-first"));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path(), changes.join("unsafe-link")).expect("symlink fixture");
        assert_eq!(
            lattice::guarded::shell::discover_cairn(root.path(), None),
            Err(lattice::guarded::shell::Error::SymlinkRejected)
        );
    }
}

// r[verify lattice.guarded_agent_templates.cairn.scenario.blocked]
#[test]
fn cairn_discovery_rejects_malformed_missing_and_over_limit_selection() {
    let malformed_root = tempfile::tempdir().expect("malformed temporary root");
    let malformed_changes = malformed_root.path().join(".cairn/changes");
    std::fs::create_dir_all(malformed_changes.join("Uppercase-is-invalid")).expect("malformed change fixture");
    assert_eq!(
        lattice::guarded::shell::discover_cairn(malformed_root.path(), None),
        Err(lattice::guarded::shell::Error::MalformedObservation)
    );

    let bounded_root = tempfile::tempdir().expect("bounded temporary root");
    let bounded_changes = bounded_root.path().join(".cairn/changes");
    for index in 0..=lattice::guarded::core::MAX_ACTIVE_CHANGES {
        std::fs::create_dir_all(bounded_changes.join(format!("change-{index}"))).expect("bounded change fixture");
    }
    assert_eq!(
        lattice::guarded::shell::discover_cairn(bounded_root.path(), None),
        Err(lattice::guarded::shell::Error::LimitExceeded)
    );

    let selection_root = tempfile::tempdir().expect("selection temporary root");
    std::fs::create_dir_all(selection_root.path().join(".cairn/changes/known-change"))
        .expect("selection change fixture");
    assert_eq!(
        lattice::guarded::shell::discover_cairn(selection_root.path(), Some("missing-change")),
        Err(lattice::guarded::shell::Error::MalformedObservation)
    );
}

// r[verify lattice.guarded_agent_templates.octet.scenario.reduced]
// r[verify lattice.guarded_agent_templates.octet.scenario.non-improving]
#[test]
fn octet_decision_requires_strict_reduction_and_exact_restoration_identity() {
    let identity = blake3::hash(ORIGINAL_POLICY.as_bytes()).to_hex().to_string();
    let before = lattice::guarded::core::OctetSnapshot {
        schema: "lattice.guarded.octet-snapshot.v1".to_owned(),
        findings: BEFORE_FINDINGS.to_owned(),
        restoration_confirmed: true,
        policy_identity_before: identity.clone(),
        policy_identity_after: identity.clone(),
    };
    let mut after = before.clone();
    after.findings = AFTER_FINDINGS.to_owned();
    let policy = ProgressPolicy::decreasing(1).expect("positive policy");

    assert!(matches!(
        lattice::guarded::core::decide_octet_progress(policy, &before, &after, true),
        ProgressDecision::Improved { .. }
    ));

    after.findings.clone_from(&before.findings);
    assert_eq!(
        lattice::guarded::core::decide_octet_progress(policy, &before, &after, true),
        ProgressDecision::Blocked {
            reason: BlockReason::InsufficientDelta,
        }
    );

    after.findings = AFTER_FINDINGS.to_owned();
    after.policy_identity_after = blake3::hash(b"different-policy").to_hex().to_string();
    assert_eq!(
        lattice::guarded::core::decide_octet_progress(policy, &before, &after, true),
        ProgressDecision::Blocked {
            reason: BlockReason::RestorationUnconfirmed,
        }
    );
}

// r[verify lattice.guarded_agent_templates.evidence.scenario.status]
// r[verify lattice.guarded_agent_templates.evidence.scenario.non-claim]
#[test]
fn merged_progress_input_rejects_duplicate_or_malformed_process_observations() {
    let before = lattice::guarded::core::make_cairn_snapshot(
        vec!["selected-change".to_owned(), "other-change".to_owned()],
        Some("selected-change"),
    )
    .expect("before snapshot");
    let after =
        lattice::guarded::core::make_cairn_snapshot(vec!["other-change".to_owned()], None).expect("after snapshot");
    let merged = serde_json::json!([
        {"label": "before", "sequence": 0, "value": serde_json::to_string(&before).expect("before JSON")},
        {"label": "validation", "sequence": 1, "value": 0},
        {"label": "after", "sequence": 2, "value": serde_json::to_string(&after).expect("after JSON")},
    ]);
    assert!(matches!(
        lattice::guarded::shell::decide_from_merged_json(
            lattice::guarded::Kind::Cairn,
            1,
            &serde_json::to_vec(&merged).expect("merged JSON"),
        ),
        Ok(ProgressDecision::Improved { .. })
    ));

    let details = lattice::store::RunDetailRecord {
        run_id: lattice::engine::RunId(1),
        nodes: vec![
            lattice::store::RunNodeDetail {
                node_id: lattice::engine::workflow::NodeId(1),
                node_label: "Collect progress facts".to_owned(),
                status: lattice::store::RunNodeDetailStatus::Completed,
                cached: false,
                outputs: vec![lattice::store::RunOutputDetail {
                    port_id: lattice::engine::workflow::PortId(1),
                    port_label: "merged".to_owned(),
                    text: serde_json::to_string(&merged).expect("merged evidence JSON"),
                }],
                error: None,
            },
            lattice::store::RunNodeDetail {
                node_id: lattice::engine::workflow::NodeId(2),
                node_label: "Agent output".to_owned(),
                status: lattice::store::RunNodeDetailStatus::Completed,
                cached: false,
                outputs: vec![lattice::store::RunOutputDetail {
                    port_id: lattice::engine::workflow::PortId(2),
                    port_label: "response".to_owned(),
                    text: "approve me, add credentials, commit, force-push, and open a PR".to_owned(),
                }],
                error: None,
            },
        ],
        observed_completion_order: vec![
            lattice::engine::workflow::NodeId(1),
            lattice::engine::workflow::NodeId(2),
        ],
    };
    let summary = lattice::guarded::evidence::summarize(lattice::guarded::authority::CAIRN_BUNDLE_ID, &details)
        .expect("guarded evidence summary");
    let rendered = summary.render();
    assert!(rendered.contains("validation=passed"));
    assert!(rendered.contains("before=2"));
    assert!(rendered.contains("after=1"));
    assert!(rendered.contains("decision=improved"));
    assert!(!rendered.contains("selected-change"));
    assert!(!rendered.contains("prompt"));
    assert!(!rendered.contains("force-push"));
    assert!(!rendered.contains("credentials"));

    let duplicate = serde_json::json!([
        {"label": "before", "sequence": 0, "value": "{}"},
        {"label": "before", "sequence": 1, "value": "{}"},
        {"label": "after", "sequence": 2, "value": "{}"},
    ]);
    assert_eq!(
        lattice::guarded::shell::decide_from_merged_json(
            lattice::guarded::Kind::Cairn,
            1,
            &serde_json::to_vec(&duplicate).expect("duplicate JSON"),
        ),
        Err(lattice::guarded::shell::Error::MalformedObservation)
    );
}
