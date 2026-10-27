use crate::{ActivationReceipt, AdmissionError, AdmittedBinding, BindingPlan, BindingRequest, PaneObservation, SourceIdentity, TILE_REVISION, Trigger};

fn validate_source(source: &SourceIdentity) -> Result<(), AdmissionError> {
    if source.producer_revision.is_empty() {
        return Err(AdmissionError::MissingSourceRevision);
    }
    if source.producer_revision != TILE_REVISION {
        return Err(AdmissionError::UnknownSourceRevision);
    }
    if source.session_id == 0 || source.pane_id == 0 || source.workspace_revision == 0 {
        return Err(AdmissionError::InvalidSourceIdentity);
    }
    Ok(())
}
pub fn validate_request(request: &BindingRequest) -> Result<(), AdmissionError> {
    validate_source(&request.source)?;
    if !request.capability_context.metadata_binding_allowed {
        return Err(AdmissionError::CapabilityDenied);
    }
    Ok(())
}


// r[impl lattice.compose_surface.admission]
pub fn admit(observation: PaneObservation, request: BindingRequest) -> Result<AdmittedBinding, AdmissionError> {
    validate_request(&request)?;
    validate_source(&observation.source)?;
    if observation.source != request.source {
        return Err(AdmissionError::SourceMismatch);
    }
    let matches = match request.trigger {
        Trigger::Lifecycle { state } => observation.lifecycle == state,
        Trigger::Focused => observation.focused,
    };
    if !matches {
        return Err(AdmissionError::TriggerNotSatisfied);
    }
    Ok(AdmittedBinding { request, observation })
}

// r[impl lattice.compose_surface.plan]
pub fn plan(admitted: AdmittedBinding) -> Result<BindingPlan, AdmissionError> {
    let binding_identity = crate::identity::binding(&admitted.request)?;
    Ok(BindingPlan { admitted, binding_identity })
}

/// Checks all fields against the exact admitted plan, not just a supplied hash.
// r[impl lattice.compose_surface.receipt]
pub fn validate_receipt(receipt: &ActivationReceipt, plan: &BindingPlan) -> Result<(), AdmissionError> {
    if *receipt != plan.receipt()? {
        return Err(AdmissionError::ReceiptMismatch);
    }
    Ok(())
}
