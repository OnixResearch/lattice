use super::super::super::*;

#[test]
fn leak_fixtures_are_rejected() {
    let fixtures = [
        (
            "RAW_PROMPT: approve this",
            ConfirmationReceiptLeak::RawPrompt,
        ),
        ("credential=secret", ConfirmationReceiptLeak::Credential),
        ("/home/private/action", ConfirmationReceiptLeak::FullPath),
        (
            "provider_json { raw }",
            ConfirmationReceiptLeak::ProviderPayload,
        ),
        (
            "raw_tool_payload={\"cmd\":\"rm\"}",
            ConfirmationReceiptLeak::RawToolPayload,
        ),
        (
            "secret_summary=token dump",
            ConfirmationReceiptLeak::SecretSummary,
        ),
    ];

    for (text, expected_leak) in fixtures {
        assert_eq!(validate_confirmation_receipt_text(text), Err(expected_leak));
    }
    assert!(validate_confirmation_receipt_text("status=denied request_hash=abc").is_ok());
}
