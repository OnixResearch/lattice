//! Pure durable local session-store framing, recovery, and evidence contracts.
//!
//! Reusable code accepts already-loaded values and bytes. Capability roots,
//! filesystem handles, locking, permissions, synchronization, encryption
//! services, quarantine, and lifecycle orchestration remain host-owned.

mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;
#[path = "wire.rs"]
mod protocol;

pub use core::{
    durable_session_store_policy_content_hash, plan_durable_session_transaction,
    validate_durable_session_store_descriptor, validate_durable_session_store_policy,
};
pub use data::{
    DURABLE_SESSION_COMMIT_HASH_DOMAIN, DURABLE_SESSION_EMPTY_HEAD_HASH,
    DURABLE_SESSION_FRAME_HASH_DOMAIN, DURABLE_SESSION_RECEIPT_HASH_DOMAIN,
    DURABLE_SESSION_RECEIPT_SCHEMA, DURABLE_SESSION_SIDECAR_HASH_DOMAIN,
    DURABLE_SESSION_STORE_POLICY_HASH_DOMAIN, DURABLE_SESSION_STORE_POLICY_SCHEMA,
    DURABLE_SESSION_TRANSACTION_HASH_DOMAIN, DurableSessionCommit, DurableSessionDurability,
    DurableSessionEncryptionPosture, DurableSessionFrame, DurableSessionLockPolicy,
    DurableSessionPrivacyPosture, DurableSessionPrivateEntryDraft, DurableSessionRecordKind,
    DurableSessionRecoveryReport, DurableSessionRecoveryStatus, DurableSessionSafeSidecar,
    DurableSessionStoreDescriptor, DurableSessionStoreError, DurableSessionStoreIssue,
    DurableSessionStoreOperation, DurableSessionStorePolicy, DurableSessionStorePolicyInput,
    DurableSessionStoreReceipt, DurableSessionStoreReceiptInput, DurableSessionStoreSensitiveDraft,
    DurableSessionTransaction, DurableSessionTransactionDraft,
};
pub use evidence::project_durable_session_store_receipt;
pub use protocol::{encode_durable_session_transaction, recover_durable_session_log};

#[cfg(test)]
mod tests;
