use super::support::*;

#[test]
fn image_descriptor_accepts_safe_metadata() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.typed_parts.validation]
    let image = valid_image_part();

    assert_eq!(image.safe_label(), IMAGE_LABEL);
    assert_eq!(image.media_type(), super::super::ImageMediaType::Png);
    assert_eq!(image.byte_count(), IMAGE_BYTE_COUNT);
    assert_eq!(image.content_hash(), VALID_HASH);
    assert_eq!(image.source_trust(), super::super::ImageSourceTrust::UserProvided);
    let metadata = image.metadata().expect("metadata should exist");
    assert_eq!(metadata.width_px(), Some(WIDTH_PX));
    assert_eq!(metadata.height_px(), Some(HEIGHT_PX));
    assert_eq!(metadata.detail(), Some(super::super::ImageDetailPreference::High));
    assert!(image.descriptor_hints().iter().any(|hint| hint == "host-admitted"));
}

#[test]
fn prompt_assembly_preserves_mixed_part_order_and_records_outcomes() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.prompt_assembly.order]
    let assembly = valid_mixed_assembly();

    assert_eq!(assembly.parts().len(), REQUEST_PART_COUNT_WITH_IMAGE);
    assert_eq!(assembly.image_outcomes().len(), IMAGE_OUTCOME_COUNT_ONE);
    assert_eq!(assembly.image_outcomes()[FIRST_OUTCOME_INDEX].status(), super::super::ImageOutcomeStatus::Included);
    assert_eq!(assembly.parts()[FIRST_PART_INDEX].as_text().expect("first part is text").text(), TEXT_BEFORE);
    assert!(assembly.parts()[IMAGE_PART_INDEX].as_image().is_some());
    assert_eq!(assembly.parts()[LAST_PART_INDEX].as_text().expect("last part is text").text(), TEXT_AFTER);
}

#[test]
fn dispatch_accepts_image_capability_and_available_adapter() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.capability_gate.unsupported]
    let assembly = valid_mixed_assembly();
    let plan = super::super::prepare_dispatch(&assembly, &vision_capabilities())
        .expect("vision model should accept image request");

    assert_eq!(plan.model_label(), MODEL_LABEL);
    assert_eq!(plan.part_count(), DISPATCH_PART_COUNT_WITH_IMAGE);
    assert_eq!(plan.image_count(), IMAGE_COUNT_ONE);
}

#[test]
fn provider_native_fixture_uses_injected_image_service() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.adapter_serialization.fixtures]
    let assembly = valid_mixed_assembly();
    let policy = native_policy();
    let service = StaticImageService { handle: valid_handle() };
    let request = super::super::serialize_provider_native_fixture(&assembly, &policy, Some(&service))
        .expect("native fixture should serialize");

    assert_eq!(request.provider_label(), PROVIDER_LABEL);
    assert_eq!(request.parts().len(), REQUEST_PART_COUNT_WITH_IMAGE);
    assert!(matches!(request.parts()[IMAGE_PART_INDEX], super::super::ProviderNativePart::Image { .. }));
}

#[test]
fn receipt_accepts_safe_image_metadata() {
    // r[verify agent_harness_runtime.multimodal_image_payloads.safe_receipts.rejects_leaks]
    let receipt = super::super::ImageReceipt::validate(&valid_receipt_draft()).expect("safe receipt should validate");

    assert_eq!(receipt.modality(), EXPECTED_MODALITY);
    assert_eq!(receipt.status(), super::super::ImageOutcomeStatus::Included);
    assert_eq!(receipt.safe_label(), IMAGE_LABEL);
    assert_eq!(receipt.media_type(), super::super::ImageMediaType::Png);
    assert_eq!(receipt.byte_count(), IMAGE_BYTE_COUNT);
    assert_eq!(receipt.content_hash(), VALID_HASH);
    assert_eq!(receipt.width_px(), Some(WIDTH_PX));
    assert_eq!(receipt.height_px(), Some(HEIGHT_PX));
    assert_eq!(receipt.adapter_class(), Some("provider-fixture"));
    assert_eq!(receipt.redaction_marker(), RECEIPT_REDACTION_MARKER);
    assert_eq!(receipt.receipt_hash().len(), super::super::BLAKE3_HEX_CHARS);
}
