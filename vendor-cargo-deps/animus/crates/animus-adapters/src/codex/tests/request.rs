use super::super::*;
use super::support::*;

#[test]
fn requests_accept_supported_selector_shapes() {
    let capabilities = &[
        DiffReviewSelectorKind::WorkingCopy,
        DiffReviewSelectorKind::Commit,
        DiffReviewSelectorKind::BranchComparison,
        DiffReviewSelectorKind::ExternalPatch,
    ];
    let working_copy = request(
        DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::WorkingCopy,
            primary_ref: None,
            secondary_ref: None,
        },
        capabilities,
    )
    .expect("working-copy selector should validate");
    assert_eq!(
        working_copy.selector().kind(),
        DiffReviewSelectorKind::WorkingCopy
    );
    assert_eq!(
        working_copy.target_labels(),
        &[alloc::string::String::from(TARGET_LABEL)]
    );

    let commit = request(
        DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::Commit,
            primary_ref: Some(COMMIT_REF),
            secondary_ref: None,
        },
        capabilities,
    )
    .expect("commit selector should validate");
    assert_eq!(commit.selector().primary_ref(), Some(COMMIT_REF));

    let branch_comparison = request(
        DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::BranchComparison,
            primary_ref: Some(BASE_REF),
            secondary_ref: Some(HEAD_REF),
        },
        capabilities,
    )
    .expect("branch comparison selector should validate");
    assert_eq!(branch_comparison.selector().secondary_ref(), Some(HEAD_REF));

    let external_patch = request(
        DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::ExternalPatch,
            primary_ref: Some(PATCH_REF),
            secondary_ref: None,
        },
        capabilities,
    )
    .expect("external patch selector should validate");
    assert_eq!(external_patch.selector().primary_ref(), Some(PATCH_REF));
}

#[test]
fn unsupported_or_missing_input_fails_before_review_starts() {
    assert_eq!(
        request(
            DiffReviewSelectorDraft {
                kind: DiffReviewSelectorKind::ExternalPatch,
                primary_ref: Some(PATCH_REF),
                secondary_ref: None,
            },
            &[DiffReviewSelectorKind::WorkingCopy],
        )
        .expect_err("unsupported selector should fail"),
        DiffReviewError::UnsupportedSelector {
            kind: DiffReviewSelectorKind::ExternalPatch,
        }
    );

    let missing_commit = request(
        DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::Commit,
            primary_ref: None,
            secondary_ref: None,
        },
        &[DiffReviewSelectorKind::Commit],
    )
    .expect_err("commit without revision should fail");
    assert_eq!(
        missing_commit,
        DiffReviewError::InvalidSelectorShape {
            kind: DiffReviewSelectorKind::Commit,
        }
    );

    let missing_targets = DiffReviewRequest::validate(&DiffReviewRequestDraft {
        run_id: RUN_ID,
        selector: DiffReviewSelectorDraft {
            kind: DiffReviewSelectorKind::WorkingCopy,
            primary_ref: None,
            secondary_ref: None,
        },
        adapter_capabilities: &[DiffReviewSelectorKind::WorkingCopy],
        target_labels: &[],
    })
    .expect_err("request without a bounded target should fail");
    assert_eq!(
        missing_targets,
        DiffReviewError::EmptyCollection {
            field: "diff_review_target_label",
        }
    );
}
