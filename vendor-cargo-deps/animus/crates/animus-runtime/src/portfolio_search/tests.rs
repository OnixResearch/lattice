#![cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]

use std::collections::BTreeMap;

use super::*;

const MAX_ROUNDS: u32 = 8;
const MAX_APPROACHES: u32 = 6;
const MAX_TOOL_CALLS: u32 = 32;
const MAX_TOKENS: u64 = 16_384;
const DISPATCH_TOOL_CALLS: u32 = 2;
const DISPATCH_TOKENS: u64 = 1_024;
const EXPECTED_TWO_ROUTES: usize = 2;
const EXPECTED_THREE_ROUTES: usize = 3;
const ONE_APPROACH_LIMIT: u32 = 1;

fn fingerprint(label: &str) -> PortfolioFingerprint {
    blake3_portfolio_fingerprint(label, label.as_bytes())
}

fn contract() -> PortfolioContract {
    PortfolioContract {
        contract_id: "portfolio.proof-review".to_string(),
        objective: fingerprint("objective"),
        completion_evidence: vec![
            "focused-validator-pass".to_string(),
            "negative-fixture-pass".to_string(),
        ],
        false_completion_cases: vec![
            "equivalent-helper-restatement".to_string(),
            "unchecked-model-consensus".to_string(),
        ],
        required_audit_risks: vec![
            PortfolioAuditRisk::Preconditions,
            PortfolioAuditRisk::TrustBoundary,
        ],
        source_policy: fingerprint("source-policy"),
        validator_policy: fingerprint("validator-policy"),
        budget: PortfolioBudget {
            rounds: MAX_ROUNDS,
            approach_capacity: MAX_APPROACHES,
            tool_calls: MAX_TOOL_CALLS,
            tokens: MAX_TOKENS,
        },
    }
}

fn dispatch(disclosed_approach_ids: Vec<String>) -> PortfolioDispatch {
    PortfolioDispatch {
        dispatch_id: "dispatch.independent".to_string(),
        family_hint: "algebraic-identity".to_string(),
        disclosed_approach_ids,
        requested_tool_calls: DISPATCH_TOOL_CALLS,
        requested_tokens: DISPATCH_TOKENS,
    }
}

#[derive(Clone, Copy)]
struct ApproachFixture<'a> {
    approach_id: &'a str,
    family: &'a str,
    mechanism_label: &'a str,
    state: ApproachState,
}

fn active_approach(fixture: ApproachFixture<'_>) -> ApproachRecord {
    ApproachRecord {
        approach_id: fixture.approach_id.to_string(),
        family: fixture.family.to_string(),
        mechanism: fingerprint(fixture.mechanism_label),
        claim: fingerprint(&format!("claim.{}", fixture.approach_id)),
        artifact_refs: vec![format!("artifact.{}", fixture.approach_id)],
        evidence_refs: vec![format!("evidence.{}", fixture.approach_id)],
        gap_strength: GapStrength::Simpler,
        state: fixture.state,
        blocker: None,
        next_check: format!("check.{}", fixture.approach_id),
    }
}

fn equivalent_blocked_approach(approach_id: &str) -> ApproachRecord {
    ApproachRecord {
        approach_id: approach_id.to_string(),
        family: "helper-reduction".to_string(),
        mechanism: fingerprint("mechanism.equivalent"),
        claim: fingerprint("claim.equivalent"),
        artifact_refs: vec!["artifact.equivalent".to_string()],
        evidence_refs: Vec::new(),
        gap_strength: GapStrength::Equivalent,
        state: ApproachState::Blocked,
        blocker: Some(PortfolioBlocker {
            class: PortfolioBlockerClass::EquivalentStrength,
            detail: "helper restates the selected theorem".to_string(),
            evidence_ref: Some("obligation.equivalent".to_string()),
        }),
        next_check: "find a strictly weaker intermediate invariant".to_string(),
    }
}

fn passing_finding(risk: PortfolioAuditRisk) -> (PortfolioAuditRisk, PortfolioAuditFinding) {
    let finding = PortfolioAuditFinding {
        risk: risk.clone(),
        outcome: AuditOutcome::Pass,
        evidence_ref: "audit.focused-check".to_string(),
        residual_uncertainty: "bounded-to-selected-fixtures".to_string(),
    };
    (risk, finding)
}

fn pass_audit(approach_id: &str) -> PortfolioAudit {
    let findings = BTreeMap::from([
        passing_finding(PortfolioAuditRisk::Preconditions),
        passing_finding(PortfolioAuditRisk::TrustBoundary),
    ]);
    PortfolioAudit {
        approach_id: approach_id.to_string(),
        findings,
    }
}

fn state_with_audited_candidate() -> PortfolioState {
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let dispatched =
        record_portfolio_dispatch(&exploring, &dispatch(Vec::new())).expect("record dispatch");
    let registered = register_approach(
        &dispatched,
        active_approach(ApproachFixture {
            approach_id: "approach.algebraic",
            family: "algebraic-identity",
            mechanism_label: "mechanism.algebraic",
            state: ApproachState::Independent,
        }),
    )
    .expect("register candidate");
    let synthesis = begin_synthesis(&registered).expect("begin synthesis");
    let auditing =
        begin_adversarial_audit(&synthesis, "approach.algebraic").expect("select audit candidate");
    let audited = record_adversarial_audit(&auditing, &pass_audit("approach.algebraic"))
        .expect("record complete audit");
    assert_eq!(audited.phase, PortfolioPhase::AdversarialAudit);
    assert!(audited.audit.is_some());
    audited
}

fn state_with_independent_routes() -> PortfolioState {
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    assert_eq!(framed.phase, PortfolioPhase::Framing);
    assert!(framed.approaches.is_empty());
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let dispatched =
        record_portfolio_dispatch(&exploring, &dispatch(Vec::new())).expect("record dispatch");
    assert_eq!(dispatched.usage.rounds, 1);
    assert_eq!(dispatched.usage.tool_calls, DISPATCH_TOOL_CALLS);
    assert_eq!(dispatched.usage.tokens, DISPATCH_TOKENS);
    let first = register_approach(
        &dispatched,
        active_approach(ApproachFixture {
            approach_id: "approach.algebraic",
            family: "algebraic-identity",
            mechanism_label: "mechanism.algebraic",
            state: ApproachState::Independent,
        }),
    )
    .expect("register first route");
    let second = register_approach(
        &first,
        active_approach(ApproachFixture {
            approach_id: "approach.induction",
            family: "induction",
            mechanism_label: "mechanism.induction",
            state: ApproachState::Independent,
        }),
    )
    .expect("register second route");
    assert_eq!(second.approaches.len(), EXPECTED_TWO_ROUTES);
    second
}

#[test]
fn validated_result_requires_independence_audit_and_matching_receipt() {
    // r[agent_harness_runtime.portfolio_search.independence]
    // r[agent_harness_runtime.portfolio_search.audit]
    // r[agent_harness_runtime.portfolio_search.validation.match]
    // r[agent_harness_runtime.portfolio_search.receipts]
    let second = state_with_independent_routes();

    let synthesis = begin_synthesis(&second).expect("begin synthesis");
    assert!(
        synthesis
            .approaches
            .values()
            .all(|approach| approach.state == ApproachState::Active)
    );
    let auditing =
        begin_adversarial_audit(&synthesis, "approach.algebraic").expect("select audit candidate");
    let audited = record_adversarial_audit(&auditing, &pass_audit("approach.algebraic"))
        .expect("record audit");
    let validating = begin_validation(&audited).expect("audit passes");
    let candidate = validating
        .approaches
        .get("approach.algebraic")
        .expect("selected candidate");
    let audit = validating.audit.as_ref().expect("audit retained");
    let receipt = PortfolioValidationReceipt {
        contract: portfolio_contract_fingerprint(&validating.contract),
        approach_id: candidate.approach_id.clone(),
        mechanism: candidate.mechanism.clone(),
        evidence: portfolio_evidence_fingerprint(candidate, audit),
        validator_policy: validating.contract.validator_policy.clone(),
        status: PortfolioValidationStatus::Passed,
    };
    let validated = validate_portfolio_result(&validating, &receipt).expect("matching receipt");
    assert_eq!(validated.phase, PortfolioPhase::Validated);
    assert_eq!(
        validated
            .approaches
            .get("approach.algebraic")
            .expect("selected route")
            .state,
        ApproachState::Validated
    );

    let projected = project_portfolio_receipt(&validated).expect("safe terminal receipt");
    assert_eq!(projected.phase, PortfolioPhase::Validated);
    assert_eq!(
        projected.selected_approach.as_deref(),
        Some("approach.algebraic")
    );
    assert!(projected.blocker.is_none());
    assert!(validate_portfolio_receipt(&projected).is_ok());
}

#[test]
fn underexplored_selection_is_dynamic_and_lexically_deterministic() {
    // r[agent_harness_runtime.portfolio_search.redirection.underexplored]
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let one = register_approach(
        &exploring,
        active_approach(ApproachFixture {
            approach_id: "approach.induction",
            family: "induction",
            mechanism_label: "mechanism.induction",
            state: ApproachState::Independent,
        }),
    )
    .expect("first route");
    let two = register_approach(
        &one,
        active_approach(ApproachFixture {
            approach_id: "approach.decompose-a",
            family: "decomposition",
            mechanism_label: "mechanism.decompose-a",
            state: ApproachState::Independent,
        }),
    )
    .expect("second route");
    let three = register_approach(
        &two,
        active_approach(ApproachFixture {
            approach_id: "approach.decompose-b",
            family: "decomposition",
            mechanism_label: "mechanism.decompose-b",
            state: ApproachState::Independent,
        }),
    )
    .expect("third route");

    assert_eq!(three.approaches.len(), EXPECTED_THREE_ROUTES);
    assert_eq!(
        underexplored_live_families(&three).expect("valid registry"),
        vec!["induction".to_string()]
    );
}

#[test]
fn equivalent_gap_stays_blocked_until_real_novelty_reclassifies_it() {
    // r[agent_harness_runtime.portfolio_search.gap_strength.equivalent]
    // r[agent_harness_runtime.portfolio_search.redirection.no_novelty]
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let registered = register_approach(
        &exploring,
        equivalent_blocked_approach("approach.equivalent"),
    )
    .expect("register honestly blocked route");
    let synthesis = begin_synthesis(&registered).expect("synthesis keeps blocker");
    let targeted = begin_targeted_redirection(&synthesis).expect("target redirection");

    let no_novelty = ReopenNovelty {
        mechanism: None,
        evidence_ref: Some("obligation.equivalent".to_string()),
        gap_strength: GapStrength::Simpler,
        next_check: "retry same helper".to_string(),
    };
    let error = reopen_approach(&targeted, "approach.equivalent", &no_novelty)
        .expect_err("rephrasing must not reopen route");
    assert_eq!(error.kind, PortfolioErrorKind::MissingNovelty);
    assert_eq!(
        targeted
            .approaches
            .get("approach.equivalent")
            .expect("blocked route")
            .state,
        ApproachState::Blocked
    );

    let novelty = ReopenNovelty {
        mechanism: Some(fingerprint("mechanism.strictly-smaller")),
        evidence_ref: Some("counterexample-boundary.fixed".to_string()),
        gap_strength: GapStrength::Simpler,
        next_check: "verify strictly smaller induction lemma".to_string(),
    };
    let reopened = reopen_approach(&targeted, "approach.equivalent", &novelty)
        .expect("new mechanism and evidence reopen route");
    let route = reopened
        .approaches
        .get("approach.equivalent")
        .expect("reopened route");
    assert_eq!(route.state, ApproachState::Active);
    assert_eq!(route.gap_strength, GapStrength::Simpler);
    assert!(route.blocker.is_none());
}

#[test]
fn contracts_phases_and_independence_fail_closed() {
    // r[agent_harness_runtime.portfolio_search.contract.rejects_incomplete]
    // r[agent_harness_runtime.portfolio_search.phases.illegal]
    // r[agent_harness_runtime.portfolio_search.independence.rejects_disclosure]
    let mut incomplete = contract();
    incomplete.false_completion_cases.clear();
    let error = initialize_portfolio(&incomplete).expect_err("missing non-solution criteria");
    assert_eq!(error.kind, PortfolioErrorKind::UnsafeMetadata);

    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let error = begin_synthesis(&framed).expect_err("cannot skip independent exploration");
    assert_eq!(error.kind, PortfolioErrorKind::IllegalPhase);

    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let error =
        validate_portfolio_dispatch(&exploring, &dispatch(vec!["approach.favored".to_string()]))
            .expect_err("independent worker must not see favored route");
    assert_eq!(error.kind, PortfolioErrorKind::CrossPollination);
}

#[test]
fn duplicate_mechanisms_and_active_equivalent_gaps_are_rejected() {
    // r[agent_harness_runtime.portfolio_search.registry.duplicate]
    // r[agent_harness_runtime.portfolio_search.gap_strength.equivalent]
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let first = register_approach(
        &exploring,
        active_approach(ApproachFixture {
            approach_id: "approach.first",
            family: "induction",
            mechanism_label: "mechanism.shared",
            state: ApproachState::Independent,
        }),
    )
    .expect("first mechanism");
    let duplicate = active_approach(ApproachFixture {
        approach_id: "approach.paraphrase",
        family: "renamed-induction",
        mechanism_label: "mechanism.shared",
        state: ApproachState::Independent,
    });
    let error = register_approach(&first, duplicate).expect_err("duplicate mechanism rejected");
    assert_eq!(error.kind, PortfolioErrorKind::DuplicateMechanism);
    assert!(error.detail.contains("approach.first"));

    let mut reduction_theater = active_approach(ApproachFixture {
        approach_id: "approach.theater",
        family: "helper-reduction",
        mechanism_label: "mechanism.theater",
        state: ApproachState::Independent,
    });
    reduction_theater.gap_strength = GapStrength::Equivalent;
    let error = validate_approach_record(&reduction_theater)
        .expect_err("equivalent gap cannot remain independently active");
    assert_eq!(error.kind, PortfolioErrorKind::BlockerMismatch);
}

#[test]
fn incomplete_failed_and_stale_audits_cannot_validate() {
    // r[agent_harness_runtime.portfolio_search.audit.fail_closed]
    // r[agent_harness_runtime.portfolio_search.validation]
    let framed = initialize_portfolio(&contract()).expect("valid contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let registered = register_approach(
        &exploring,
        active_approach(ApproachFixture {
            approach_id: "approach.algebraic",
            family: "algebraic-identity",
            mechanism_label: "mechanism.algebraic",
            state: ApproachState::Independent,
        }),
    )
    .expect("register route");
    let synthesis = begin_synthesis(&registered).expect("begin synthesis");
    let auditing = begin_adversarial_audit(&synthesis, "approach.algebraic").expect("begin audit");

    let mut incomplete = pass_audit("approach.algebraic");
    incomplete
        .findings
        .remove(&PortfolioAuditRisk::TrustBoundary);
    let error = record_adversarial_audit(&auditing, &incomplete)
        .expect_err("missing required risk rejected");
    assert_eq!(error.kind, PortfolioErrorKind::AuditIncomplete);

    let mut failed = pass_audit("approach.algebraic");
    failed
        .findings
        .get_mut(&PortfolioAuditRisk::TrustBoundary)
        .expect("required finding")
        .outcome = AuditOutcome::Fail;
    let audited = record_adversarial_audit(&auditing, &failed).expect("record failed audit");
    let error = begin_validation(&audited).expect_err("failed audit cannot validate");
    assert_eq!(error.kind, PortfolioErrorKind::AuditFailed);

    let audited = record_adversarial_audit(&auditing, &pass_audit("approach.algebraic"))
        .expect("record passing audit");
    let validating = begin_validation(&audited).expect("enter validation");
    let candidate = validating
        .approaches
        .get("approach.algebraic")
        .expect("candidate");
    let matching = PortfolioValidationReceipt {
        contract: portfolio_contract_fingerprint(&validating.contract),
        approach_id: candidate.approach_id.clone(),
        mechanism: candidate.mechanism.clone(),
        evidence: portfolio_evidence_fingerprint(
            candidate,
            validating.audit.as_ref().expect("audit"),
        ),
        validator_policy: validating.contract.validator_policy.clone(),
        status: PortfolioValidationStatus::Passed,
    };
    let mut stale = matching.clone();
    stale.contract = fingerprint("stale-contract");
    let error =
        validate_portfolio_result(&validating, &stale).expect_err("stale contract cannot validate");
    assert_eq!(error.kind, PortfolioErrorKind::ReceiptMismatch);

    let mut unadmitted_policy = matching;
    unadmitted_policy.validator_policy = fingerprint("unadmitted-validator-policy");
    let error = validate_portfolio_result(&validating, &unadmitted_policy)
        .expect_err("unadmitted validator policy cannot validate");
    assert_eq!(error.kind, PortfolioErrorKind::ReceiptMismatch);
    assert_eq!(validating.phase, PortfolioPhase::Validation);
}

#[test]
fn exhaustion_and_blocking_are_honest_safe_terminal_states() {
    // r[agent_harness_runtime.portfolio_search.validation.exhausted]
    // r[agent_harness_runtime.portfolio_search.receipts.rejects_raw]
    let mut bounded_contract = contract();
    bounded_contract.budget.approach_capacity = ONE_APPROACH_LIMIT;
    let framed = initialize_portfolio(&bounded_contract).expect("valid bounded contract");
    let exploring = begin_independent_exploration(&framed).expect("start exploration");
    let registered = register_approach(
        &exploring,
        equivalent_blocked_approach("approach.equivalent"),
    )
    .expect("register one blocked route");
    assert!(is_portfolio_budget_exhausted(&registered));
    let exhausted = exhaust_portfolio(&registered).expect("record honest exhaustion");
    assert_eq!(exhausted.phase, PortfolioPhase::Exhausted);
    assert!(exhausted.validation_receipt.is_none());
    let receipt = project_portfolio_receipt(&exhausted).expect("exhaustion receipt");
    assert_eq!(receipt.phase, PortfolioPhase::Exhausted);
    assert!(receipt.blocker.is_none());

    let error = begin_independent_exploration(&exhausted)
        .expect_err("terminal exhaustion cannot resume implicitly");
    assert_eq!(error.kind, PortfolioErrorKind::TerminalState);

    let mut unsafe_receipt = receipt.clone();
    unsafe_receipt
        .evidence_refs
        .push("/home/operator/raw prompt".to_string());
    let error = validate_portfolio_receipt(&unsafe_receipt)
        .expect_err("raw path and prompt marker rejected");
    assert_eq!(error.kind, PortfolioErrorKind::UnsafeMetadata);

    let blocked = block_portfolio(
        &exploring,
        PortfolioBlocker {
            class: PortfolioBlockerClass::UserDecisionRequired,
            detail: "choose between incompatible public contracts".to_string(),
            evidence_ref: Some("decision.contract-shape".to_string()),
        },
    )
    .expect("record exact user blocker");
    let blocked_receipt = project_portfolio_receipt(&blocked).expect("blocked receipt");
    assert_eq!(blocked_receipt.phase, PortfolioPhase::Blocked);
    assert_eq!(
        blocked_receipt
            .blocker
            .as_ref()
            .map(|blocker| blocker.class),
        Some(PortfolioBlockerClass::UserDecisionRequired)
    );
}

#[test]
fn receipt_identity_detects_field_tampering() {
    // r[agent_harness_runtime.portfolio_search.receipts]
    let audited = state_with_audited_candidate();
    let validating = begin_validation(&audited).expect("enter validation");
    let candidate = validating
        .approaches
        .get("approach.algebraic")
        .expect("candidate");
    let validation_receipt = PortfolioValidationReceipt {
        contract: portfolio_contract_fingerprint(&validating.contract),
        approach_id: candidate.approach_id.clone(),
        mechanism: candidate.mechanism.clone(),
        evidence: portfolio_evidence_fingerprint(
            candidate,
            validating.audit.as_ref().expect("audit"),
        ),
        validator_policy: validating.contract.validator_policy.clone(),
        status: PortfolioValidationStatus::Passed,
    };
    let validated =
        validate_portfolio_result(&validating, &validation_receipt).expect("validate result");
    let mut projected = project_portfolio_receipt(&validated).expect("project receipt");
    projected
        .evidence_refs
        .push("evidence.tampered".to_string());
    assert_eq!(
        projected.evidence_refs.last().map(String::as_str),
        Some("evidence.tampered")
    );
    let error = validate_portfolio_receipt(&projected).expect_err("identity must detect mutation");
    assert_eq!(error.kind, PortfolioErrorKind::ReceiptMismatch);
}
