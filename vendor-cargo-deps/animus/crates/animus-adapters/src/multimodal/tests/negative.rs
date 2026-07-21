use super::support::*;

#[test]
fn image_descriptor_rejects_malformed_or_unsafe_fields() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.typed_parts.validation]
    let invalid_media = super::super::ImagePart::from_draft(super::super::ImagePartDraft {
        media_type: "application/octet-stream",
        ..valid_image_draft()
    })
    .expect_err("unsupported media type should fail");
    assert!(matches!(
        invalid_media,
        super::super::Error::UnsupportedMediaType { .. }
    ));

    let missing_hash = super::super::ImagePart::from_draft(super::super::ImagePartDraft {
        content_hash: "",
        ..valid_image_draft()
    })
    .expect_err("missing hash should fail");
    assert!(matches!(
        missing_hash,
        super::super::Error::AdapterField { .. }
    ));

    let missing_count = super::super::ImagePart::from_draft(super::super::ImagePartDraft {
        byte_count: 0,
        ..valid_image_draft()
    })
    .expect_err("missing byte count should fail");
    assert_eq!(missing_count, super::super::Error::MissingByteCount);

    for (hint, reason) in UNSAFE_DESCRIPTOR_FIXTURES {
        let error = super::super::ImagePart::from_draft(super::super::ImagePartDraft {
            descriptor_hints: &[hint],
            ..valid_image_draft()
        })
        .expect_err("unsafe descriptor hint should fail");
        assert_eq!(
            error,
            super::super::Error::UnsafeDescriptor {
                field: super::super::SOURCE_HINT_FIELD.as_str(),
                reason,
            }
        );
    }
}

#[test]
fn prompt_assembly_rejects_oversized_images_without_silent_drop() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.prompt_assembly.order]
    let assembly = super::super::assemble_prompt_parts(
        &[
            super::super::ContentPartDraft::Text { text: TEXT_BEFORE },
            super::super::ContentPartDraft::Image(super::super::ImagePartDraft {
                byte_count: OVER_BUDGET_BYTES,
                ..valid_image_draft()
            }),
            super::super::ContentPartDraft::Text { text: TEXT_AFTER },
        ],
        super::super::PromptAssemblyPolicy::new(SMALL_BUDGET_BYTES),
    )
    .expect("oversized image should be an explicit outcome");

    assert_eq!(assembly.parts().len(), REQUEST_PART_COUNT_TEXT_ONLY);
    assert_eq!(assembly.image_outcomes().len(), IMAGE_OUTCOME_COUNT_ONE);
    let outcome = &assembly.image_outcomes()[FIRST_OUTCOME_INDEX];
    assert_eq!(outcome.status(), super::super::ImageOutcomeStatus::Rejected);
    assert_eq!(outcome.safe_label(), IMAGE_LABEL);
    assert_eq!(outcome.content_hash(), VALID_HASH);
    assert_eq!(outcome.reason(), Some("image_byte_budget_exceeded"));
}

#[test]
fn dispatch_rejects_unsupported_model_and_unavailable_adapter() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.capability_gate.unsupported]
    let assembly = valid_mixed_assembly();
    let denied_dispatch = super::super::prepare_dispatch(&assembly, &text_only_capabilities())
        .expect_err("text-only model should fail closed");
    assert_eq!(
        denied_dispatch,
        super::super::Error::UnsupportedModality {
            model_label: alloc::string::String::from(TEXT_ONLY_MODEL_LABEL),
            modality: super::super::InputModality::Image,
        }
    );

    let unavailable = super::super::prepare_dispatch(
        &assembly,
        &super::super::ModelInputCapabilities::from_draft(
            super::super::ModelInputCapabilitiesDraft {
                model_label: MODEL_LABEL,
                supports_text: true,
                supports_images: true,
                image_adapter_available: false,
            },
        )
        .expect("capabilities should validate"),
    )
    .expect_err("missing image adapter should fail closed");
    assert_eq!(
        unavailable,
        super::super::Error::AdapterUnavailable {
            label: alloc::string::String::from(MODEL_LABEL),
        }
    );
}

#[test]
fn provider_native_fixture_rejects_lossy_missing_invalid_and_live_call_paths() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.adapter_serialization.fixtures]
    let assembly = valid_mixed_assembly();
    let generic_policy = super::super::ProviderSerializationPolicy::from_draft(
        super::super::ProviderSerializationPolicyDraft {
            provider_label: PROVIDER_LABEL,
            mode: super::super::ProviderSerializationMode::GenericJson,
        },
    )
    .expect("policy should validate");
    let service = StaticImageService {
        handle: valid_handle(),
    };
    let lossy =
        super::super::serialize_provider_native_fixture(&assembly, &generic_policy, Some(&service))
            .expect_err("generic conversion should fail for images");
    assert_eq!(lossy, super::super::Error::LossyGenericConversion);

    let missing =
        super::super::serialize_provider_native_fixture(&assembly, &native_policy(), None)
            .expect_err("missing image service should fail");
    assert_eq!(missing, super::super::Error::MissingImageService);

    let mismatched_service = StaticImageService {
        handle: super::super::ImagePayloadHandle::from_draft(
            super::super::ImagePayloadHandleDraft {
                content_hash: OTHER_VALID_HASH,
                ..valid_handle_draft()
            },
        )
        .expect("handle should validate"),
    };
    let mismatched = super::super::serialize_provider_native_fixture(
        &assembly,
        &native_policy(),
        Some(&mismatched_service),
    )
    .expect_err("mismatched handle should fail");
    assert_eq!(
        mismatched,
        super::super::Error::MismatchedImageHandle {
            field: super::super::CONTENT_HASH_FIELD.as_str(),
        }
    );

    let live_service = FailingImageService {
        error: super::super::ImagePayloadError::LiveProviderCallRequired,
    };
    let live = super::super::serialize_provider_native_fixture(
        &assembly,
        &native_policy(),
        Some(&live_service),
    )
    .expect_err("live-call service should fail fixture serialization");
    assert_eq!(live, super::super::Error::LiveProviderCallRequired);
}

#[test]
fn receipt_rejects_image_metadata_leaks() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.safe_receipts.rejects_leaks]
    for (fragment, reason) in RECEIPT_LEAK_FIXTURES {
        let error = super::super::ImageReceipt::validate(&super::super::ImageReceiptDraft {
            metadata_fragments: &[fragment],
            ..valid_receipt_draft()
        })
        .expect_err("receipt leak should fail");
        assert_eq!(
            error,
            super::super::Error::UnsafeDescriptor {
                field: super::super::METADATA_FRAGMENT_FIELD.as_str(),
                reason,
            }
        );
    }
}
