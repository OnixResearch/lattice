//! Owner-only source edit types; CAS expectations and native receipts are distinct.
use super::CompilerIdentity;
use super::ContentIdentity;
use super::SourceDraft;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Rejection {
    InvalidRequest,
    RequestTooLarge,
    UnsupportedSchema,
    WorkflowNotFound,
    SourceUnavailable,
    HeadConflict,
    DraftConflict,
    DraftTargetMismatch,
    MemberNotFound,
    ValidationFailed,
    StoreError,
    CommitUnknown,
}

#[derive(serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct ExpectedDraft(pub Option<ContentIdentity>);

impl ExpectedDraft {
    // deserialize_with makes the field required while permitting an explicit null.
    pub(crate) fn required<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde::Deserialize::deserialize(d)
    }
}

#[derive(serde::Serialize)]
pub(crate) struct NativeReceipt {
    pub compiler: CompilerIdentity,
    pub evaluated_output: ContentIdentity,
    pub expanded_artifact: ContentIdentity,
}

pub(crate) fn draft_revision(draft: &SourceDraft) -> Result<ContentIdentity, &'static str> {
    super::identity("lattice.source-draft.v1", draft)
}
