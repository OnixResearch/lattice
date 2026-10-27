use alloc::{string::String, vec, vec::Vec};

use crate::*;

const REPOSITORY_ID: &str = "repo:animus";
const POLICY_ID: &str = "policy:review";
const PLAN_ID: &str = "plan:review";
const IMPLEMENTATION_ID: &str = "impl:fixture";
const CORRELATION_ID: &str = "correlation:review";
const CHANGE_REQUEST_ID: &str = "cr:7";
const SOURCE_BRANCH: &str = "topic/update";
const TARGET_BRANCH: &str = "main";
const COMMIT_ID: &str = "commit:fixture";
const FILE_ID: &str = "src/lib.rs";
const REVIEW_LINE: u32 = 7;
const REVIEW_HUNKS: u32 = 2;
const REVIEW_ADDED_LINES: u32 = 4;
const REVIEW_DELETED_LINES: u32 = 1;
const HIGH_CONFIDENCE: u8 = 95;
const LOW_CONFIDENCE: u8 = 20;
const NEXT_GENERATION: u64 = 2;
const SOURCE_COHORT_FILES: usize = 4;
const HASH: &str = "b3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UPPER_HASH: &str = "b3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn binding() -> ChangeRequestBinding {
    ChangeRequestBinding::new(
        String::from(REPOSITORY_ID),
        String::from(POLICY_ID),
        String::from(PLAN_ID),
        1,
        String::from(IMPLEMENTATION_ID),
        String::from(CORRELATION_ID),
    )
    .expect("fixture binding must validate")
}

fn repository() -> RepositoryRef {
    RepositoryRef {
        identity: String::from(REPOSITORY_ID),
        forge: ForgeKind::GitHub,
        default_branch: String::from(TARGET_BRANCH),
    }
}

fn change_request() -> ChangeRequestRef {
    ChangeRequestRef {
        identity: String::from(CHANGE_REQUEST_ID),
        source_branch: String::from(SOURCE_BRANCH),
        target_branch: String::from(TARGET_BRANCH),
        commit_identity: String::from(COMMIT_ID),
    }
}

fn anchor() -> ReviewInlineAnchor {
    ReviewInlineAnchor {
        commit_identity: String::from(COMMIT_ID),
        file_identity: String::from(FILE_ID),
        line: REVIEW_LINE,
        side: ReviewSide::Right,
        start_line: None,
    }
}

fn diff(with_anchor: bool) -> DiffObservation {
    DiffObservation {
        commit_identity: String::from(COMMIT_ID),
        files: vec![DiffFileObservation {
            file_identity: String::from(FILE_ID),
            hunk_count: REVIEW_HUNKS,
            added_line_count: REVIEW_ADDED_LINES,
            deleted_line_count: REVIEW_DELETED_LINES,
            inline_anchors: if with_anchor {
                vec![anchor()]
            } else {
                Vec::new()
            },
        }],
        total_hunk_count: REVIEW_HUNKS,
        total_added_line_count: REVIEW_ADDED_LINES,
        total_deleted_line_count: REVIEW_DELETED_LINES,
    }
}

fn finding(confidence: u8) -> ReviewFinding {
    ReviewFinding {
        source: ReviewFindingSource::Deterministic,
        severity: ReviewSeverity::Warning,
        category: String::from("correctness"),
        message: String::from("The branch can return stale state."),
        file_identity: Some(String::from(FILE_ID)),
        line_range: Some(LineRange {
            start: REVIEW_LINE,
            end: REVIEW_LINE,
        }),
        side: Some(ReviewSide::Right),
        confidence,
    }
}

fn review_input(with_anchor: bool, confidence: u8) -> CodeReviewInput {
    CodeReviewInput {
        binding: binding(),
        repository: repository(),
        change_request: change_request(),
        diff: diff(with_anchor),
        deterministic_findings: vec![finding(confidence)],
        ci_failures: vec![CheckFailureObservation {
            profile_identity: String::from("validation:quick"),
            output_identity: String::from(HASH),
            repair_attempts: 0,
        }],
        policy: ReviewPolicy::conservative(),
        bounds: ReviewBounds::conservative(),
        forge_capabilities: ForgeCapabilities::full_review(),
    }
}

#[test]
fn plans_analysis_ci_repair_inline_comment_and_summary() {
    // r[verify agent_harness_runtime.remora_adapters.change_request.selected]
    // r[verify agent_harness_runtime.remora_adapters.change_request.anchor_missing]
    let plan = plan_code_review(review_input(true, HIGH_CONFIDENCE))
        .expect("matching bounded review input must plan");

    assert_eq!(plan.findings.len(), 1);
    assert!(
        plan.effects
            .iter()
            .any(|effect| matches!(effect, ReviewEffect::RequestAnalysis { .. }))
    );
    assert!(
        plan.effects
            .iter()
            .any(|effect| matches!(effect, ReviewEffect::RequestCiRepair { .. }))
    );
    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        ReviewEffect::PrepareInlineComment { anchor, .. } if anchor.line == REVIEW_LINE
    )));
    assert!(
        plan.effects
            .iter()
            .any(|effect| matches!(effect, ReviewEffect::PrepareSummary { finding_count: 1 }))
    );
}

#[test]
fn missing_anchor_falls_back_or_fails_closed() {
    let fallback = plan_code_review(review_input(false, HIGH_CONFIDENCE))
        .expect("summary fallback must keep the plan bounded");
    assert!(fallback.effects.iter().any(|effect| matches!(
        effect,
        ReviewEffect::Escalate {
            reason: ReviewEscalationReason::MissingInlineAnchor,
            ..
        }
    )));

    let mut denied = review_input(false, HIGH_CONFIDENCE);
    denied.policy.allow_summary_fallback = false;
    assert_eq!(plan_code_review(denied), Err(Error::MissingInlineAnchor));

    let mut wrong_side = review_input(true, HIGH_CONFIDENCE);
    wrong_side.deterministic_findings[0].side = Some(ReviewSide::Left);
    wrong_side.policy.allow_summary_fallback = false;
    assert_eq!(
        plan_code_review(wrong_side),
        Err(Error::MissingInlineAnchor)
    );
}

#[test]
fn low_confidence_and_exhausted_repair_attempts_escalate() {
    let mut input = review_input(true, LOW_CONFIDENCE);
    input.ci_failures[0].repair_attempts = input.policy.max_repair_attempts;
    let plan = plan_code_review(input).expect("bounded escalations must plan");

    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        ReviewEffect::Escalate {
            reason: ReviewEscalationReason::LowConfidence,
            ..
        }
    )));
    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        ReviewEffect::Escalate {
            reason: ReviewEscalationReason::RepairAttemptsExhausted,
            ..
        }
    )));
}

#[test]
fn independent_service_catalog_fails_closed_for_missing_services() {
    // r[verify agent_harness_runtime.remora_adapters.ports.subset]
    // r[verify agent_harness_runtime.remora_adapters.ports.absent]
    let catalog = ServiceCatalog {
        services: vec![ServiceDescriptor {
            kind: ServiceKind::ChangeRequestSource,
            implementation_identity: String::from(IMPLEMENTATION_ID),
            availability: ServiceAvailability::Available,
        }],
    };

    assert_eq!(
        catalog
            .admit(ServiceKind::ChangeRequestSource)
            .map(|item| item.kind),
        Ok(ServiceKind::ChangeRequestSource)
    );
    assert_eq!(
        catalog.admit(ServiceKind::ReviewPublication),
        Err(Error::MissingService(ServiceKind::ReviewPublication))
    );

    let denied = ServiceCatalog {
        services: vec![ServiceDescriptor {
            kind: ServiceKind::ReviewPublication,
            implementation_identity: String::from(IMPLEMENTATION_ID),
            availability: ServiceAvailability::Denied,
        }],
    };
    assert_eq!(
        denied.admit(ServiceKind::ReviewPublication),
        Err(Error::ServiceBlocked {
            service: ServiceKind::ReviewPublication,
            availability: ServiceAvailability::Denied,
        })
    );
}

#[test]
fn stale_or_cross_repository_binding_is_rejected() {
    // r[verify agent_harness_runtime.remora_adapters.binding.replay]
    let stale = ChangeRequestBinding::new(
        String::from(REPOSITORY_ID),
        String::from(POLICY_ID),
        String::from(PLAN_ID),
        NEXT_GENERATION,
        String::from(IMPLEMENTATION_ID),
        String::from(CORRELATION_ID),
    )
    .expect("stale fixture binding remains structurally valid");
    assert_eq!(
        validate_binding(&binding(), &stale),
        Err(Error::BindingMismatch)
    );
}

#[test]
fn source_and_publication_observations_bind_exact_subjects() {
    // r[verify agent_harness_runtime.remora_adapters.binding.match]
    let fetch_request = ChangeRequestFetchRequest {
        binding: binding(),
        repository: repository(),
        change_request_identity: String::from(CHANGE_REQUEST_ID),
        bounds: ReviewBounds::conservative(),
    };
    let fetch_observation = ChangeRequestFetchObservation {
        binding: binding(),
        change_request: change_request(),
        diff: diff(true),
        ci_failures: Vec::new(),
    };
    assert_eq!(
        validate_fetch_observation(&fetch_request, &fetch_observation),
        Ok(())
    );

    let effects = vec![
        ReviewEffect::PrepareInlineComment {
            finding: finding(HIGH_CONFIDENCE),
            anchor: anchor(),
        },
        ReviewEffect::PrepareSummary { finding_count: 1 },
    ];
    let publish_request = ReviewPublishRequest {
        binding: binding(),
        change_request: change_request(),
        effects,
        max_effects: DEFAULT_MAX_REVIEW_FINDINGS,
    };
    let mut publish_observation = ReviewPublishObservation {
        binding: binding(),
        inline_comment_count: 1,
        summary_published: true,
        evidence_identity: String::from(HASH),
    };
    assert_eq!(
        validate_review_publication(&publish_request, &publish_observation),
        Ok(())
    );

    publish_observation.inline_comment_count = 0;
    assert_eq!(
        validate_review_publication(&publish_request, &publish_observation),
        Err(Error::BindingMismatch)
    );
}

#[test]
fn receipts_are_metadata_only_and_reject_overclaims() {
    // r[verify agent_harness_runtime.remora_adapters.evidence.safe]
    // r[verify agent_harness_runtime.remora_adapters.evidence.reject]
    let mut draft = AdapterReceiptDraft {
        request_binding: binding(),
        observation_binding: binding(),
        operation: ReceiptOperation::PublishReview,
        status: ReceiptStatus::Completed,
        item_count: 1,
        request_identity: String::from(HASH),
        observation_identity: String::from(HASH),
        disclosure: ContentDisclosure::MetadataOnly,
        claim: ReceiptClaim::BoundedContract,
    };
    let receipt = build_receipt(&draft).expect("safe receipt must build");
    assert!(receipt.receipt_identity.starts_with("b3:"));

    draft.request_identity = String::from(UPPER_HASH);
    assert_eq!(
        build_receipt(&draft),
        Err(Error::InvalidField("receipt.request_identity"))
    );
    draft.request_identity = String::from(HASH);
    draft.disclosure = ContentDisclosure::RawContent;
    assert_eq!(build_receipt(&draft), Err(Error::UnsafeReceipt));
    draft.disclosure = ContentDisclosure::MetadataOnly;
    draft.claim = ReceiptClaim::ProviderDelivery;
    assert_eq!(build_receipt(&draft), Err(Error::UnsupportedClaim));
}

#[test]
fn source_provenance_is_exact_and_bounded() {
    // r[verify agent_harness_runtime.remora_adapters.provenance.cohort]
    assert_eq!(
        REMORA_SOURCE_REPOSITORY,
        "git@github.com:OnixResearch/remora.git"
    );
    assert_eq!(
        REMORA_SOURCE_REVISION,
        "e3c857c857f4aa170d8a5b6286aad5855db541be"
    );
    assert_eq!(REMORA_DECLARED_LICENSE, "MIT OR Apache-2.0");
    assert_eq!(
        REMORA_SOURCE_COHORT_IDENTITY,
        "b3:a084a8a06727b1fae59377d34754f224065351e550caadf4e12c0bbcdcc26e42"
    );
    assert_eq!(REMORA_SOURCE_COHORT.len(), SOURCE_COHORT_FILES);
    assert!(REMORA_IMPORTED_SCOPE.contains("authority-free"));
    assert!(REMORA_OWNER_GRANT_BASIS.contains("owner adaptation"));
}
