//! Renderer-neutral layered key-sequence catalogs, resolution, and editing.
//!
//! Physical keyboard events, clocks, action callbacks, and executors remain in
//! renderer/application shells. This module accepts only logical key values and
//! returns immutable snapshots, deterministic projections, or route receipts.

semantic_string_id! {
    /// Stable identity for one action binding.
    pub struct ActionBindingId;
}

semantic_string_id! {
    /// Stable identity for one binding layer.
    pub struct BindingLayerId;
}

#[path = "key_sequence/adapters.rs"]
mod adapters;
#[path = "key_sequence/catalog.rs"]
mod catalog;
#[path = "key_sequence/dispatch.rs"]
mod dispatch;
#[path = "key_sequence/editing.rs"]
mod editing;
#[path = "key_sequence/introspection.rs"]
mod introspection;
#[path = "key_sequence/logical.rs"]
mod logical;
#[path = "key_sequence/reducer.rs"]
mod reducer;
#[path = "key_sequence/replay.rs"]
mod replay;
#[path = "key_sequence/resolution.rs"]
mod resolution;
#[path = "key_sequence/surfaces.rs"]
mod surfaces;
#[path = "key_sequence/validation.rs"]
mod validation;

pub use adapters::KeyAdapterDiagnostic;
pub use adapters::KeyAdapterDiagnosticCode;
pub use adapters::KeyCapabilityStatus;
pub use adapters::RendererKeyCapabilities;
pub use adapters::RendererKeyInput;
pub use adapters::normalize_renderer_key_input;
pub use catalog::ActionBinding;
pub use catalog::ActionBindingSource;
pub use catalog::ActionBindingSourceKind;
pub use catalog::ActiveBindingLayer;
pub use catalog::ActiveBindingLayers;
pub use catalog::BindingCatalogBuild;
pub use catalog::BindingCatalogCandidate;
pub use catalog::BindingCatalogDiagnostic;
pub use catalog::BindingCatalogDiagnosticCode;
pub use catalog::BindingCatalogLimits;
pub use catalog::BindingCatalogSnapshot;
pub use catalog::BindingLayer;
pub use catalog::BindingLayerKind;
pub use catalog::ReservedKeySequence;
pub use catalog::accept_binding_catalog;
pub use catalog::binding_catalog_from_registry_defaults;
pub use dispatch::ResolvedBindingDispatch;
pub use dispatch::dispatch_resolved_binding;
pub use editing::BindingEditCommand;
pub use editing::BindingEditOutcome;
pub use editing::SequenceCapture;
pub use editing::SequenceCaptureDiagnostic;
pub use editing::SequenceCaptureOutcome;
pub use editing::apply_binding_edit;
pub use introspection::BindingInspection;
pub use introspection::BindingInspectionState;
pub use introspection::DescribeSequenceProjection;
pub use introspection::RendererBindingSupport;
pub use introspection::describe_key_sequence;
pub use introspection::where_is_action;
pub use logical::KeyModifier;
pub use logical::KeySequence;
pub use logical::KeySequenceDiagnostic;
pub use logical::KeySequenceDiagnosticCode;
pub use logical::LogicalKey;
pub use logical::LogicalKeystroke;
pub use reducer::KeySequenceInput;
pub use reducer::KeySequenceReducerOutcome;
pub use reducer::KeySequenceState;
pub use reducer::KeySequenceTransition;
pub use reducer::reduce_key_sequence;
pub use replay::KeySequenceReplay;
pub use replay::KeySequenceReplayReceipt;
pub use replay::KeySequenceReplayStep;
pub use replay::replay_key_sequences;
pub use resolution::BindingConsideration;
pub use resolution::BindingDecision;
pub use resolution::BindingResolution;
pub use resolution::BindingResolutionOutcome;
pub use resolution::BindingResolutionProvenance;
pub use resolution::PendingContinuation;
pub use resolution::ResolvedActionBinding;
pub use resolution::resolve_key_sequence;
pub use surfaces::ActionBindingsProjection;
pub use surfaces::BindingCommandViews;
pub use surfaces::project_binding_command_views;

#[cfg(test)]
#[path = "key_sequence/tests.rs"]
mod tests;
