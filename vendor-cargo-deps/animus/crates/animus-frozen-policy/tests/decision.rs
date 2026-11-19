//! Selections: fixed-point tree and MLP scores, the best eligible action with stable ties, and
//! abstention to a declared fallback.

mod support;

#[test]
// r[verify agent_harness_runtime.frozen_policy.selection]
// r[verify agent_harness_runtime.frozen_policy.model]
// r[verify agent_harness_runtime.frozen_policy.boundary]
fn the_tree_selects_the_best_eligible_action() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let decision = animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 200, 10]));
    assert_eq!(decision.outcome, support::selected("proceed", 100));
    let body = &decision.receipt.body;
    assert_eq!(body.scores, vec![100, 20, 0]);
    assert_eq!(body.cost.normalization_operations, 3);
    assert_eq!(body.cost.inference_operations, 2);
    assert_eq!(body.cost.fallback_decisions, 0);
    assert_eq!(body.artifact.as_deref(), Some(tree.identity()));
    assert_eq!(body.model.as_deref(), Some(tree.model_identity()));
    assert_eq!(body.policy_kind.as_deref(), Some("decision-tree-v1"));
    assert_eq!(body.features, vec![2, 200, 10]);
    let escalated = animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[4, 0, 0]));
    assert_eq!(
        escalated.outcome,
        support::selected("escalate", 90),
        "400 per mille of attempts exceeds the 300 threshold"
    );
    let again = animus_frozen_policy::decide(Ok(&tree), &support::request(&tree, &[2, 200, 10]));
    assert_eq!(
        again.receipt.identity, decision.receipt.identity,
        "a frozen policy decides one request identically"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.model]
fn the_mlp_computes_fixed_point_scores() -> Result<(), support::Failure> {
    let mlp = support::admitted(support::MLP)?;
    let decision = animus_frozen_policy::decide(Ok(&mlp), &support::request(&mlp, &[5, 800, 250]));
    assert_eq!(decision.outcome, support::selected("retry", 380));
    assert_eq!(decision.receipt.body.scores, vec![-150, 380, 200]);
    assert_eq!(decision.receipt.body.cost.inference_operations, 24);
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.selection]
fn only_eligible_actions_are_selected_and_ties_take_the_earliest() -> Result<(), support::Failure> {
    let mlp = support::admitted(support::MLP)?;
    let mut request = support::request(&mlp, &[5, 800, 250]);
    request.eligible = support::labels(&["escalate", "proceed"]);
    assert_eq!(
        animus_frozen_policy::decide(Ok(&mlp), &request).outcome,
        support::selected("escalate", 200),
        "the best action is not eligible, so the best eligible one answers"
    );
    let tied = support::admitted_edit(support::TREE, |value| {
        value["model"]["nodes"][3]["leaf"]["scores"] = serde_json::json!([60, 60, 60]);
    })?;
    let mut request = support::request(&tied, &[2, 200, 10]);
    request.eligible = support::labels(&["escalate", "retry"]);
    assert_eq!(
        animus_frozen_policy::decide(Ok(&tied), &request).outcome,
        support::selected("retry", 60),
        "equal scores select the earliest declared action, whatever the eligible order"
    );
    Ok(())
}

#[test]
// r[verify agent_harness_runtime.frozen_policy.selection]
// r[verify agent_harness_runtime.frozen_policy.failure]
fn a_weak_best_eligible_score_abstains_to_the_declared_fallback() -> Result<(), support::Failure> {
    let tree = support::admitted(support::TREE)?;
    let mut request = support::request(&tree, &[2, 200, 10]);
    request.eligible = support::labels(&["retry", "escalate"]);
    assert_eq!(
        animus_frozen_policy::decide(Ok(&tree), &request).outcome,
        animus_frozen_policy::Outcome::Refused {
            failure: animus_frozen_policy::Failure::Abstained,
        },
        "retry's 20 lies below the threshold of 50 and no fallback is declared"
    );
    request.fallback = Some(String::from("escalate"));
    let decision = animus_frozen_policy::decide(Ok(&tree), &request);
    assert_eq!(
        decision.outcome,
        support::fallback("escalate", animus_frozen_policy::Failure::Abstained)
    );
    let body = &decision.receipt.body;
    assert_eq!(body.outcome, animus_frozen_policy::ReceiptOutcome::Fallback);
    assert_eq!(body.failure.as_deref(), Some("abstained"));
    assert_eq!(body.cost.fallback_decisions, 1);
    assert_eq!(
        body.scores,
        vec![100, 20, 0],
        "the scores that abstained stay recorded"
    );
    Ok(())
}
