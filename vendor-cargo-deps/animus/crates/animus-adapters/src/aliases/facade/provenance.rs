pub use crate::safe::receipt::{BoundedCount, BoundedCountDraft};
/// Largest accepted bounded count value in a safe receipt.
pub const MAX_SAFE_RECEIPT_COUNT: u32 = crate::safe::receipt::MAX_COUNT;
/// Largest accepted safe metadata label length.
pub const MAX_SAFE_RECEIPT_LABEL_CHARS: usize = crate::safe::receipt::MAX_LABEL_CHARS;
/// Schema identifier attached to safe receipt DTOs.
pub const SAFE_RECEIPT_SCHEMA: &str = crate::safe::receipt::SCHEMA;
/// Issue code recorded by a safe receipt.
pub type ReceiptIssueCode = crate::safe::receipt::IssueCode;
/// Unsafe material class rejected by safe receipt validation.
pub type ReceiptLeakClass = crate::safe::receipt::LeakClass;
/// Effect or mutation class observed by a safe receipt.
pub type ReceiptOperation = crate::safe::receipt::Operation;
/// Redaction marker recorded by a safe receipt.
pub type ReceiptRedaction = crate::safe::receipt::Redaction;
/// Status class recorded by a safe receipt.
pub type ReceiptStatus = crate::safe::receipt::Status;
/// Safe runtime/provenance receipt containing metadata, counts, hashes, and redactions only.
pub type SafeReceipt = crate::safe::receipt::Record;
/// Borrowed inputs used to construct a safe receipt.
pub type SafeReceiptDraft<'a> = crate::safe::receipt::Draft<'a>;
/// Errors raised by safe receipt validation.
pub type SafeReceiptError = crate::safe::receipt::Error;
