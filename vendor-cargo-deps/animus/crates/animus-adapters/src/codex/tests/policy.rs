use super::super::*;

#[test]
fn read_only_profile_allows_inputs_and_denies_mutations() {
    let profile = DiffReviewPolicyProfile::read_only();
    assert!(profile.evaluate_action(DiffReviewActionKind::InspectDiff).is_allowed());
    assert!(profile.evaluate_action(DiffReviewActionKind::ReadFile).is_allowed());
    assert!(profile.evaluate_action(DiffReviewActionKind::ModelReview).is_allowed());

    assert_eq!(
        profile.evaluate_action(DiffReviewActionKind::EditFile).denial_reason(),
        Some(DiffReviewDenialReason::FileMutation)
    );
    assert_eq!(
        profile.evaluate_action(DiffReviewActionKind::WriteCapableTool).denial_reason(),
        Some(DiffReviewDenialReason::WriteCapableTool)
    );
    assert_eq!(
        profile.evaluate_action(DiffReviewActionKind::ApprovalEscalation).denial_reason(),
        Some(DiffReviewDenialReason::ApprovalEscalation)
    );
    assert_eq!(
        profile.evaluate_action(DiffReviewActionKind::PullRequestManagement).denial_reason(),
        Some(DiffReviewDenialReason::PullRequestManagement)
    );
}
