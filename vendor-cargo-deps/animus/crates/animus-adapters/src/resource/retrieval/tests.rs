const HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SAFE_REFERENCE: &str = "artifact:b3:bbbb";
const UNSAFE_REFERENCE: &str = "/home/example/raw-fetch.txt";
const RAW_BODY: &str = "full fetched document body";
const MISSING_FACT: &str = "owner";
const FIRST_RETRIEVAL: u32 = 1;
const SECOND_RETRIEVAL: u32 = 2;

#[test]
fn valid_budget_is_admitted_and_advertises_guidance() {
    let budget = valid_budget();
    let admission = super::contract::admit(
        super::contract::Capability::WebSearch,
        Some(&budget),
        super::contract::Policy::new(super::contract::Requirement::Required),
    )
    .expect("budgeted resource should be admitted");

    assert_eq!(admission, super::contract::Admission::Advertise);
    assert!(budget.guidance_summary().contains("first:"));
    assert_eq!(budget.repeat_conditions()[0], "missing required fact");
}

#[test]
fn required_budget_missing_diagnostics_cover_capabilities() {
    let policy = super::contract::Policy::new(super::contract::Requirement::Required);
    let capabilities = [
        super::contract::Capability::WebSearch,
        super::contract::Capability::BrowserAutomation,
        super::contract::Capability::DocumentationLookup,
        super::contract::Capability::RepositoryBrowsing,
        super::contract::Capability::MultiLensAnalysis,
        super::contract::Capability::PropertyDiscovery,
    ];

    for capability in capabilities {
        let error = super::contract::admit(capability, None, policy)
            .expect_err("retrieval-capable resource should require a budget");
        assert_eq!(error, super::failure::Error::MissingBudget { capability });
    }
    assert_eq!(
        super::contract::admit(super::contract::Capability::NonRetrieval, None, policy)
            .expect("non-retrieval resource should not need a budget"),
        super::contract::Admission::Advertise
    );
}

#[test]
fn sufficient_first_evidence_stops() {
    let budget = valid_budget();
    let evidence = super::facts::EvidenceSummary::validate(&super::facts::EvidenceSummaryDraft {
        status: super::facts::EvidenceStatus::Sufficient,
        missing_evidence: &[],
        retrievals_performed: FIRST_RETRIEVAL,
    })
    .expect("sufficient evidence should validate");

    assert_eq!(
        super::facts::plan(&budget, &evidence),
        super::facts::Decision::AnswerFromEvidence
    );
}

#[test]
fn missing_required_fact_authorizes_targeted_repeat() {
    let budget = repeat_budget();
    let evidence = missing_evidence(FIRST_RETRIEVAL);

    assert_eq!(
        super::facts::plan(&budget, &evidence),
        super::facts::Decision::TargetedRepeat {
            missing_evidence: alloc::vec![alloc::string::String::from(MISSING_FACT)],
        }
    );
}

#[test]
fn exhausted_missing_fact_requests_smallest_input() {
    let budget = repeat_budget();
    let exhausted = missing_evidence(SECOND_RETRIEVAL);

    assert_eq!(
        super::facts::plan(&budget, &exhausted),
        super::facts::Decision::RequestSmallestMissingInput {
            missing_evidence: alloc::vec![alloc::string::String::from(MISSING_FACT)],
        }
    );
}

#[test]
fn evidence_metadata_is_bounded_and_hash_only() {
    let metadata = super::facts::Metadata::validate(&super::facts::MetadataDraft {
        source_label: Some("docs"),
        safe_reference: Some(SAFE_REFERENCE),
        content_hash: Some(HASH),
        truncation: super::facts::Truncation::Truncated,
        raw_body: None,
    })
    .expect("safe metadata should validate");

    assert_eq!(metadata.source_label(), "docs");
    assert_eq!(metadata.safe_reference(), SAFE_REFERENCE);
    assert_eq!(metadata.truncation(), super::facts::Truncation::Truncated);
    assert!(metadata.content_hash().is_some());
}

#[test]
fn evidence_metadata_rejects_raw_bodies() {
    let error = super::facts::Metadata::validate(&super::facts::MetadataDraft {
        source_label: Some("docs"),
        safe_reference: Some(SAFE_REFERENCE),
        content_hash: Some(HASH),
        truncation: super::facts::Truncation::Complete,
        raw_body: Some(RAW_BODY),
    })
    .expect_err("raw body should be rejected");

    assert_eq!(error, super::failure::Error::RawBodyEmbedded);
}

#[test]
fn evidence_metadata_rejects_unsafe_references() {
    let error = super::facts::Metadata::validate(&super::facts::MetadataDraft {
        source_label: Some("docs"),
        safe_reference: Some(UNSAFE_REFERENCE),
        content_hash: Some(HASH),
        truncation: super::facts::Truncation::Complete,
        raw_body: None,
    })
    .expect_err("unsafe reference should be rejected");

    assert!(matches!(
        error,
        super::failure::Error::UnsafeReference { .. }
    ));
}

fn valid_budget() -> super::contract::Budget {
    super::contract::Budget::validate(&super::contract::Draft::default())
        .expect("budget should validate")
}

fn repeat_budget() -> super::contract::Budget {
    super::contract::Budget::validate(&super::contract::Draft {
        max_retrievals: Some(SECOND_RETRIEVAL),
        ..super::contract::Draft::default()
    })
    .expect("budget should validate")
}

fn missing_evidence(retrievals_performed: u32) -> super::facts::EvidenceSummary {
    super::facts::EvidenceSummary::validate(&super::facts::EvidenceSummaryDraft {
        status: super::facts::EvidenceStatus::MissingRequired,
        missing_evidence: &[MISSING_FACT],
        retrievals_performed,
    })
    .expect("missing evidence should validate")
}
