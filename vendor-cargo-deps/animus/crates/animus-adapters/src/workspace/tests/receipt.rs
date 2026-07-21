const RAW_PROMPT_FIXTURE: &str = "RAW_PROMPT: hidden instruction";
const RAW_FILE_FIXTURE: &str = "RAW_FILE: body";
const SAFE_RECEIPT_FIXTURE: &str = "workspace=ws:child descriptor_hash=abc redacted";

#[test]
fn validation_rejects_raw_evidence() {
    assert_eq!(
        crate::validate_workspace_receipt_text(RAW_PROMPT_FIXTURE),
        Err(crate::WorkspaceReceiptLeak::RawPrompt)
    );
    assert_eq!(
        crate::validate_workspace_receipt_text(RAW_FILE_FIXTURE),
        Err(crate::WorkspaceReceiptLeak::RawFileContents)
    );
    assert!(crate::validate_workspace_receipt_text(SAFE_RECEIPT_FIXTURE).is_ok());
}
