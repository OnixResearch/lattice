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

pub use core::durable_session_store_policy_content_hash;
pub use core::plan_durable_session_transaction;
pub use core::validate_durable_session_store_descriptor;
pub use core::validate_durable_session_store_policy;

pub use data::DURABLE_SESSION_COMMIT_HASH_DOMAIN;
pub use data::DURABLE_SESSION_EMPTY_HEAD_HASH;
pub use data::DURABLE_SESSION_FRAME_HASH_DOMAIN;
pub use data::DURABLE_SESSION_RECEIPT_HASH_DOMAIN;
pub use data::DURABLE_SESSION_RECEIPT_SCHEMA;
pub use data::DURABLE_SESSION_SIDECAR_HASH_DOMAIN;
pub use data::DURABLE_SESSION_STORE_POLICY_HASH_DOMAIN;
pub use data::DURABLE_SESSION_STORE_POLICY_SCHEMA;
pub use data::DURABLE_SESSION_TRANSACTION_HASH_DOMAIN;
pub use data::DurableSessionCommit;
pub use data::DurableSessionDurability;
pub use data::DurableSessionEncryptionPosture;
pub use data::DurableSessionFrame;
pub use data::DurableSessionLockPolicy;
pub use data::DurableSessionPrivacyPosture;
pub use data::DurableSessionPrivateEntryDraft;
pub use data::DurableSessionRecordKind;
pub use data::DurableSessionRecoveryReport;
pub use data::DurableSessionRecoveryStatus;
pub use data::DurableSessionSafeSidecar;
pub use data::DurableSessionStoreDescriptor;
pub use data::DurableSessionStoreError;
pub use data::DurableSessionStoreIssue;
pub use data::DurableSessionStoreOperation;
pub use data::DurableSessionStorePolicy;
pub use data::DurableSessionStorePolicyInput;
pub use data::DurableSessionStoreReceipt;
pub use data::DurableSessionStoreReceiptInput;
pub use data::DurableSessionStoreSensitiveDraft;
pub use data::DurableSessionTransaction;
pub use data::DurableSessionTransactionDraft;
pub use evidence::project_durable_session_store_receipt;
pub use protocol::encode_durable_session_transaction;
pub use protocol::recover_durable_session_log;

#[cfg(test)]
mod tests;
