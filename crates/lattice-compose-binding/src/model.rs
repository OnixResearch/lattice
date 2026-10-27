use alloc::string::String;
use lattice_workflow_contract::Identity;
use serde::{Deserialize, Serialize};

/// Exact reviewed producer of WorkspaceInspectResponse; not a daemon-truth claim.
// r[impl lattice.compose_surface.dependency]
pub const TILE_REVISION: &str = "5e703953840bff72a91fec3e928682e9efb607ab";
pub const MAXIMUM_REQUEST_BYTES: usize = 4096;
pub const MAXIMUM_RECEIPT_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationSchema {
    #[serde(rename = "lattice.tile-pane-observation.v1")]
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneLifecycle {
    Starting,
    Running,
    Exited,
    NeedsRestart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub producer_revision: String,
    /// Explicit host-selected daemon authority identity, never inferred from UI.
    pub daemon_identity: Identity,
    pub session_id: u64,
    pub pane_id: u64,
    pub workspace_revision: u64,
}

/// A Lattice-owned projection of Tile's validated metadata-only inspection.
// r[impl lattice.compose_surface.admission]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaneObservation {
    pub schema: ObservationSchema,
    pub source: SourceIdentity,
    pub lifecycle: PaneLifecycle,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields, rename_all = "snake_case")]
pub enum Trigger {
    Lifecycle { state: PaneLifecycle },
    Focused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityContext {
    pub identity: Identity,
    pub metadata_binding_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingRequest {
    pub source: SourceIdentity,
    pub workflow_identity: Identity,
    pub trigger: Trigger,
    pub capability_context: CapabilityContext,
    /// Host-supplied activation identity; not part of stable binding identity.
    pub activation_identity: Identity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBinding {
    pub(crate) request: BindingRequest,
    pub(crate) observation: PaneObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectIntent {
    PublishMetadataReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingPlan {
    pub(crate) admitted: AdmittedBinding,
    pub(crate) binding_identity: Identity,
}

impl BindingPlan {
    pub fn binding_identity(&self) -> &Identity {
        &self.binding_identity
    }

    pub const fn effect_intent(&self) -> EffectIntent {
        EffectIntent::PublishMetadataReceipt
    }

    pub fn receipt(&self) -> Result<ActivationReceipt, AdmissionError> {
        crate::identity::receipt(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptClaim {
    #[serde(rename = "binding_activated_metadata_only")]
    BindingActivatedMetadataOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptSchema {
    #[serde(rename = "lattice.pane-binding-receipt.v1")]
    V1,
}

// r[impl lattice.compose_surface.receipt]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationReceipt {
    pub schema: ReceiptSchema,
    pub identity: Identity,
    pub binding_identity: Identity,
    pub activation_identity: Identity,
    pub source: SourceIdentity,
    pub workflow_identity: Identity,
    pub capability_identity: Identity,
    pub observed_lifecycle: PaneLifecycle,
    pub observed_focused: bool,
    pub claim: ReceiptClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    MissingSourceRevision,
    UnknownSourceRevision,
    InvalidSourceIdentity,
    SourceMismatch,
    CapabilityDenied,
    TriggerNotSatisfied,
    ReceiptMismatch,
    InvalidIdentity,
}

impl core::fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
