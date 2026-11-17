// r[verify lattice.workflow_intent.planning.scenario.finite]
// r[verify lattice.workflow_intent.planning.scenario.denial]
// r[verify lattice.workflow_intent.bindings.scenario.drift]
// r[verify lattice.workflow_intent.proposals.scenario.readmission]
use std::collections::BTreeSet;

use lattice::workflow::intent::*;

fn digest(value: &str) -> String {
    format!("b3:{}", blake3::hash(value.as_bytes()).to_hex())
}

fn obligation(id: &str, class: &str, prerequisites: &[&str]) -> Obligation {
    Obligation {
        id: id.into(),
        obligation_identity: digest(&format!("obligation:{id}")),
        guidance_identity: digest(&format!("guidance:{id}")),
        remaining_condition_identity: digest(&format!("remaining:{id}")),
        precondition_identity: digest(&format!("precondition:{id}")),
        operation_class: class.into(),
        action_identity: digest(&format!("action:{id}")),
        prerequisites: prerequisites.iter().map(|value| (*value).to_string()).collect(),
        required_observations: vec![ObservationContract {
            id: format!("evidence-{id}"),
            schema: "lattice.workflow-intent-fixture-result.v1".into(),
            predicate_identity: digest(&format!("predicate:{id}")),
        }],
        required_capabilities: BTreeSet::from(["native-cairn".into()]),
    }
}

fn request() -> CompileRequest {
    let cohort = SourceCohort {
        producer_identity: digest("producer"),
        selection_identity: digest("selection"),
        obligations_identity: digest("obligations"),
        profile_identity: digest("profile"),
        context_identity: digest("context"),
        source_identity: digest("source"),
        policy_identity: digest("policy"),
        dependency_identity: digest("dependencies"),
    };
    CompileRequest {
        intent: Definition {
            schema: INTENT_SCHEMA.into(),
            cohort: cohort.clone(),
            coverage: ContextCoverage::Complete,
            unsupported_fields: vec![],
            obligations: vec![
                obligation("A", "cairn_validate", &[]),
                obligation("B", "review_wait", &["A"]),
                obligation("C", "named_check", &[]),
                obligation("D", "completion_only", &["B", "C"]),
            ],
            alternatives: vec![],
            selected_alternative: None,
        },
        facts: CompilerFacts {
            expected_cohort: cohort,
            compiler_artifact_identity: digest("compiler-artifact"),
            runtime_profile_identity: digest("runtime-profile"),
            available_capabilities: BTreeSet::from(["native-cairn".into()]),
            available_implementations: BTreeSet::from([digest("implementation-1"), digest("implementation-2")]),
        },
    }
}

#[test]
fn finite_verify_dag_preserves_origins_review_and_completion_contracts() {
    let input = request();
    let plan = compile(input.clone()).unwrap();
    assert_eq!(plan.nodes(), ["A", "B", "C", "D"]);
    assert_eq!(plan.intent().obligations, input.intent.obligations);
    let edges = plan
        .graph()
        .edges
        .iter()
        .map(|edge| (edge.source.as_str(), edge.target.as_str()))
        .collect::<BTreeSet<_>>();
    assert_eq!(edges, BTreeSet::from([("A", "B"), ("B", "D"), ("C", "D")]));
    assert!(plan.graph().effects.is_empty(), "a compile must not declare executable effects");
    assert_eq!(plan.intent().obligations[1].operation_class, "review_wait");
    assert_eq!(
        plan.intent().obligations[3].required_observations,
        input.intent.obligations[3].required_observations
    );
}

#[test]
fn unordered_inputs_preserve_identity_and_bound_changes_invalidate_it() {
    let original = request();
    let base = compile(original.clone()).unwrap();
    let mut reordered = original.clone();
    reordered.intent.obligations.reverse();
    for value in &mut reordered.intent.obligations {
        value.prerequisites.reverse();
    }
    assert_eq!(base.identity(), compile(reordered).unwrap().identity());
    let mut policy = original.clone();
    policy.intent.cohort.policy_identity = digest("policy-2");
    policy.facts.expected_cohort = policy.intent.cohort.clone();
    let changed_policy = compile(policy).unwrap();
    assert_ne!(base.identity(), changed_policy.identity());
    assert_eq!(base.graph(), changed_policy.graph(), "policy binding is distinct from graph shape");
    let mut compiler = original.clone();
    compiler.facts.compiler_artifact_identity = digest("different-compiler");
    assert_ne!(base.identity(), compile(compiler).unwrap().identity());
    let mut profile = original;
    profile.facts.runtime_profile_identity = digest("different-runtime-profile");
    assert_ne!(base.identity(), compile(profile).unwrap().identity());
}

#[test]
fn invalid_incomplete_and_over_budget_inputs_do_not_yield_a_plan() {
    let mut invalid = Vec::new();
    let mut missing = request();
    missing.intent.obligations[0].required_observations.clear();
    invalid.push((missing, DiagnosticCode::MissingContract));
    let mut cycle = request();
    cycle.intent.obligations[0].prerequisites.push("B".into());
    invalid.push((cycle, DiagnosticCode::GraphAdmission));
    let mut crossed = request();
    crossed.facts.expected_cohort.source_identity = digest("stale-source");
    invalid.push((crossed, DiagnosticCode::BindingMismatch));
    let mut restricted = request();
    restricted.intent.coverage = ContextCoverage::Restricted;
    invalid.push((restricted, DiagnosticCode::IncompleteContext));
    let mut incomplete = request();
    incomplete.intent.coverage = ContextCoverage::Incomplete;
    invalid.push((incomplete, DiagnosticCode::IncompleteContext));
    let mut unsupported = request();
    unsupported.intent.obligations[0].operation_class = "arbitrary_shell".into();
    invalid.push((unsupported, DiagnosticCode::UnsupportedOperation));
    let mut capability = request();
    capability.facts.available_capabilities.clear();
    invalid.push((capability, DiagnosticCode::MissingCapability));
    let mut too_many = request();
    too_many.intent.obligations = (0..=MAX_OBLIGATIONS)
        .map(|index| obligation(&format!("check-{index}"), "named_check", &[]))
        .collect();
    invalid.push((too_many, DiagnosticCode::LimitExceeded));
    for (input, expected) in invalid {
        assert_eq!(compile(input).unwrap_err().code, expected);
    }
}

#[test]
fn alternatives_change_exact_bindings_not_required_evidence_or_nodes() {
    let mut input = request();
    input.intent.alternatives = ["implementation-1", "implementation-2"]
        .into_iter()
        .map(|id| Alternative {
            id: id.into(),
            implementation_identity: digest(id),
            obligations: BTreeSet::from(["C".into()]),
        })
        .collect();
    assert_eq!(compile(input.clone()).unwrap_err().code, DiagnosticCode::AmbiguousAlternative);
    input.intent.selected_alternative = Some("implementation-1".into());
    let first = compile(input.clone()).unwrap();
    input.intent.selected_alternative = Some("implementation-2".into());
    let second = compile(input.clone()).unwrap();
    assert_ne!(first.identity(), second.identity());
    assert_ne!(first.graph(), second.graph());
    assert_eq!(first.nodes(), second.nodes());
    assert_eq!(first.intent().obligations, second.intent().obligations);
    input.facts.available_implementations.clear();
    assert_eq!(compile(input).unwrap_err().code, DiagnosticCode::MissingCapability);
}

#[test]
fn proposed_extra_work_or_weaker_evidence_requires_owner_readmission() {
    let input = request();
    let base = compile(input.clone()).unwrap();
    let mut proposal = input.clone();
    proposal.intent.obligations.push(obligation("investigate", "named_check", &["A"]));
    proposal.intent.obligations[0].required_observations[0].predicate_identity = digest("changed-evidence");
    let comparison = compare_proposal(&base, proposal).unwrap();
    assert_eq!(comparison.added_obligations, ["investigate"]);
    assert_eq!(comparison.changed_obligations, ["A"]);
    assert_eq!(comparison.disposition, "owner_readmission_required");
    assert_eq!(base.intent(), &input.intent, "a proposal cannot mutate the frozen input");
    let mut changed_policy = input;
    changed_policy.facts.expected_cohort.policy_identity = digest("other-policy");
    assert_eq!(compare_proposal(&base, changed_policy).unwrap_err().code, DiagnosticCode::BindingMismatch);
}
