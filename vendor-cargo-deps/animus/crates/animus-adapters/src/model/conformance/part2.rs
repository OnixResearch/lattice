#![allow(unknown_lints)]
#![allow(non_trait_imports)]
use super::*;

#[test]
fn response_normalization_fails_closed_for_malformed_and_leaky_fixtures() {
    // r[verify agent_harness_runtime.provider_projection_conformance.response_fail_closed]
    let missing_finish =
        normalize_provider_response_fixture(text_response_fixture(RESPONSE_TEXT, None))
            .expect_err("missing finish should fail");
    assert!(matches!(
        missing_finish,
        ProviderProjectionError::CanonicalContract { .. }
    ));

    let malformed_usage = normalize_provider_response_fixture(ProviderResponseFixtureDraft {
        usage: Some(CanonicalUsageDraft {
            input: 0,
            output: 0,
            cached_input: None,
        }),
        ..text_response_fixture(RESPONSE_TEXT, Some(CanonicalFinishReason::Stop.as_str()))
    })
    .expect_err("malformed usage should fail");
    assert!(matches!(
        malformed_usage,
        ProviderProjectionError::CanonicalContract { .. }
    ));

    let impossible = normalize_provider_response_fixture(ProviderResponseFixtureDraft {
        text: Some(RESPONSE_TEXT),
        refusal: Some(REFUSAL_REASON),
        ..base_response_fixture(Some(CanonicalFinishReason::Refusal.as_str()))
    })
    .expect_err("text plus refusal should fail");
    assert!(matches!(
        impossible,
        ProviderProjectionError::CanonicalContract { .. }
    ));

    for draft in [
        ProviderResponseFixtureDraft {
            raw_provider_payload: Some(RAW_PAYLOAD),
            ..base_response_fixture(Some(CanonicalFinishReason::Stop.as_str()))
        },
        ProviderResponseFixtureDraft {
            hidden_reasoning: Some(HIDDEN_REASONING),
            ..base_response_fixture(Some(CanonicalFinishReason::Stop.as_str()))
        },
        text_response_fixture(
            CREDENTIAL_SENTINEL,
            Some(CanonicalFinishReason::Stop.as_str()),
        ),
    ] {
        let error = normalize_provider_response_fixture(draft).expect_err("leak should fail");
        assert!(matches!(
            error,
            ProviderProjectionError::CanonicalContract { .. }
        ));
    }
}

#[test]
fn streaming_fixtures_use_the_production_parser_entrypoint() {
    // r[verify agent_harness_runtime.provider_projection_conformance.streaming_entrypoint]
    let chunks = [
        "content|call-1|1|he",
        "llo\n",
        "tool|call-1|2|tool-1|arguments-hash\n",
        "usage|call-1|3|12|8\n",
        "finish|call-1|4\n",
    ];
    let expected = [
        wire::parts::Kind::ContentDelta,
        wire::parts::Kind::ToolCallDelta,
        wire::parts::Kind::Usage,
        wire::parts::Kind::Finish,
    ];
    let frames = assert_provider_stream_fixture(&chunks, &expected)
        .expect("stream fixture should parse through production entrypoint");
    assert_eq!(frames.len(), expected.len());
    assert_eq!(frames[1].call_id().as_str(), STREAM_CALL_ID);
    assert_eq!(
        frames[1]
            .tool_call_id()
            .expect("tool id should exist")
            .as_str(),
        STREAM_TOOL_ID
    );

    let cancelled = assert_provider_stream_fixture(
        &[
            "content|call-1|1|partial\n",
            "cancelled|call-1|2|user_cancelled\n",
        ],
        &[
            wire::parts::Kind::ContentDelta,
            wire::parts::Kind::Cancelled,
        ],
    )
    .expect("cancelled stream fixture should parse");
    assert_eq!(cancelled[1].kind(), wire::parts::Kind::Cancelled);

    let failed = assert_provider_stream_fixture(
        &["failure|call-1|1|provider_error\n"],
        &[wire::parts::Kind::Failure],
    )
    .expect("failure stream fixture should parse");
    assert_eq!(failed[0].kind(), wire::parts::Kind::Failure);
}

#[test]
fn streaming_fixtures_reject_malformed_or_unexpected_order() {
    // r[verify agent_harness_runtime.provider_projection_conformance.streaming_fail_closed]
    let malformed = assert_provider_stream_fixture(
        &["tool|only-two-fields\n"],
        &[wire::parts::Kind::ToolCallDelta],
    )
    .expect_err("malformed stream chunk should fail");
    assert!(matches!(
        malformed,
        ProviderProjectionError::StreamFixture { .. }
    ));

    let truncated = assert_provider_stream_fixture(
        &["content|call-1|1|partial"],
        &[wire::parts::Kind::ContentDelta],
    )
    .expect_err("truncated stream chunk should fail");
    assert!(matches!(
        truncated,
        ProviderProjectionError::StreamFixture { .. }
    ));

    let missing_terminal = assert_provider_stream_fixture(
        &["content|call-1|1|partial\n"],
        &[wire::parts::Kind::ContentDelta],
    )
    .expect_err("stream without terminal frame should fail");
    assert!(matches!(
        missing_terminal,
        ProviderProjectionError::StreamFixture { .. }
    ));

    let unexpected_order = assert_provider_stream_fixture(
        &["finish|call-1|1\n", "content|call-1|2|late\n"],
        &[wire::parts::Kind::Finish, wire::parts::Kind::ContentDelta],
    )
    .expect_err("late content after finish should fail");
    assert!(matches!(
        unexpected_order,
        ProviderProjectionError::StreamFixture { .. }
    ));
}

#[test]
fn conformance_index_requires_evidence_or_reason_by_status() {
    // r[verify agent_harness_runtime.provider_projection_conformance.index]
    let entries = validate_provider_projection_index(&[
        ProviderProjectionIndexEntryDraft {
            family: ProviderProjectionFamily::OpenAiResponses.as_str(),
            feature: INDEX_FEATURE,
            status: ProviderProjectionIndexStatus::Covered,
            evidence: Some(INDEX_EVIDENCE),
            reason: None,
        },
        ProviderProjectionIndexEntryDraft {
            family: ProviderProjectionFamily::AnthropicMessages.as_str(),
            feature: "image",
            status: ProviderProjectionIndexStatus::Unsupported,
            evidence: None,
            reason: Some(INDEX_REASON),
        },
        ProviderProjectionIndexEntryDraft {
            family: ProviderProjectionFamily::Mock.as_str(),
            feature: "native billing hooks",
            status: ProviderProjectionIndexStatus::NotApplicable,
            evidence: None,
            reason: Some("mock provider does not bill"),
        },
    ])
    .expect("complete index entries should validate");
    assert_eq!(entries[0].feature(), INDEX_FEATURE);
    assert_eq!(
        entries[1].status(),
        ProviderProjectionIndexStatus::Unsupported
    );

    let missing_evidence =
        validate_provider_projection_index(&[ProviderProjectionIndexEntryDraft {
            family: ProviderProjectionFamily::OpenAiResponses.as_str(),
            feature: INDEX_FEATURE,
            status: ProviderProjectionIndexStatus::Covered,
            evidence: None,
            reason: None,
        }])
        .expect_err("covered case without evidence should fail");
    assert!(matches!(
        missing_evidence,
        ProviderProjectionError::MissingIndexField { .. }
    ));

    let missing_reason = validate_provider_projection_index(&[ProviderProjectionIndexEntryDraft {
        family: ProviderProjectionFamily::OpenAiChat.as_str(),
        feature: "image",
        status: ProviderProjectionIndexStatus::Unsupported,
        evidence: None,
        reason: None,
    }])
    .expect_err("unsupported case without reason should fail");
    assert!(matches!(
        missing_reason,
        ProviderProjectionError::MissingIndexField { .. }
    ));
}
pub mod part3;
pub use part3::*;
