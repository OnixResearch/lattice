use crate::{ActivationReceipt, AdmissionError, BindingPlan, BindingRequest, PaneLifecycle, ReceiptClaim, ReceiptSchema, SourceIdentity, Trigger};
use alloc::string::String;
use lattice_workflow_contract::Identity;

/// Domain-separated, labelled, length-prefixed bytes; independent of JSON order.
struct Frame(blake3::Hasher);

impl Frame {
    fn new(domain: &[u8]) -> Self {
        let mut frame = Self(blake3::Hasher::new());
        frame.part(domain);
        frame
    }

    fn part(&mut self, bytes: &[u8]) {
        self.0.update(&(bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }

    fn field(&mut self, label: &[u8], bytes: &[u8]) {
        self.part(label);
        self.part(bytes);
    }

    fn number(&mut self, label: &[u8], value: u64) {
        self.field(label, &value.to_le_bytes());
    }

    fn source(&mut self, source: &SourceIdentity) {
        self.field(b"producer_revision", source.producer_revision.as_bytes());
        self.field(b"daemon_identity", source.daemon_identity.as_str().as_bytes());
        self.number(b"session_id", source.session_id);
        self.number(b"pane_id", source.pane_id);
        self.number(b"workspace_revision", source.workspace_revision);
    }

    fn finish(self) -> Result<Identity, AdmissionError> {
        let digest = self.0.finalize();
        let mut encoded = String::with_capacity(67);
        encoded.push_str("b3:");
        encoded.push_str(digest.to_hex().as_str());
        Identity::new(encoded).map_err(|_| AdmissionError::InvalidIdentity)
    }
}

pub(crate) const fn lifecycle_name(state: PaneLifecycle) -> &'static [u8] {
    match state {
        PaneLifecycle::Starting => b"starting",
        PaneLifecycle::Running => b"running",
        PaneLifecycle::Exited => b"exited",
        PaneLifecycle::NeedsRestart => b"needs_restart",
    }
}

// r[impl lattice.compose_surface.identity]
pub(crate) fn binding(request: &BindingRequest) -> Result<Identity, AdmissionError> {
    let mut frame = Frame::new(b"lattice.pane-binding.v1");
    frame.source(&request.source);
    frame.field(b"workflow_identity", request.workflow_identity.as_str().as_bytes());
    match request.trigger {
        Trigger::Lifecycle { state } => {
            frame.field(b"trigger", b"lifecycle");
            frame.field(b"state", lifecycle_name(state));
        }
        Trigger::Focused => frame.field(b"trigger", b"focused"),
    }
    frame.field(b"capability_identity", request.capability_context.identity.as_str().as_bytes());
    frame.field(b"metadata_binding_allowed", &[u8::from(request.capability_context.metadata_binding_allowed)]);
    frame.finish()
}

pub(crate) fn receipt(plan: &BindingPlan) -> Result<ActivationReceipt, AdmissionError> {
    let request = &plan.admitted.request;
    let observation = &plan.admitted.observation;
    let mut frame = Frame::new(b"lattice.pane-binding.activation.v1");
    frame.field(b"binding_identity", plan.binding_identity.as_str().as_bytes());
    frame.field(b"activation_identity", request.activation_identity.as_str().as_bytes());
    frame.field(b"observed_lifecycle", lifecycle_name(observation.lifecycle));
    frame.field(b"observed_focused", &[u8::from(observation.focused)]);
    frame.field(b"claim", b"binding_activated_metadata_only");
    Ok(ActivationReceipt {
        schema: ReceiptSchema::V1,
        identity: frame.finish()?,
        binding_identity: plan.binding_identity.clone(),
        activation_identity: request.activation_identity.clone(),
        source: request.source.clone(),
        workflow_identity: request.workflow_identity.clone(),
        capability_identity: request.capability_context.identity.clone(),
        observed_lifecycle: observation.lifecycle,
        observed_focused: observation.focused,
        claim: ReceiptClaim::BindingActivatedMetadataOnly,
    })
}
