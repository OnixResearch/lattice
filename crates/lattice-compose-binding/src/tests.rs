#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "closed boundary fixtures fail immediately on setup or rejection errors"
)]

use super::*;
use alloc::{format, string::ToString};
use lattice_workflow_contract::Identity;

fn identity(byte: char) -> Identity {
    Identity::new(format!("b3:{}", byte.to_string().repeat(64))).unwrap()
}

fn fixture() -> (PaneObservation, BindingRequest) {
    (
        serde_json::from_str(include_str!("../fixtures/observation.json")).unwrap(),
        serde_json::from_str(include_str!("../fixtures/request.json")).unwrap(),
    )
}

fn planned(observation: PaneObservation, request: BindingRequest) -> BindingPlan {
    plan(admit(observation, request).unwrap()).unwrap()
}

// r[verify lattice.compose_surface.identity]
// r[verify lattice.compose_surface.identity.scenario.equal]
// r[verify lattice.compose_surface.identity.scenario.distinct]
// r[verify lattice.compose_surface.plan.scenario.deterministic]
#[test]
fn binding_identity_covers_all_authority_dimensions_but_not_activation() {
    let (observation, request) = fixture();
    let baseline = planned(observation.clone(), request.clone());
    assert_eq!(baseline, planned(observation.clone(), request.clone()));
    let mut changes = alloc::vec::Vec::new();
    let mut changed = request.clone();
    changed.source.daemon_identity = identity('2');
    changes.push(changed);
    let mut changed = request.clone();
    changed.source.session_id += 1;
    changes.push(changed);
    let mut changed = request.clone();
    changed.source.pane_id += 1;
    changes.push(changed);
    let mut changed = request.clone();
    changed.source.workspace_revision += 1;
    changes.push(changed);
    let mut changed = request.clone();
    changed.workflow_identity = identity('3');
    changes.push(changed);
    let mut changed = request.clone();
    changed.trigger = Trigger::Focused;
    changes.push(changed);
    let mut changed = request.clone();
    changed.capability_context.identity = identity('4');
    changes.push(changed);
    for changed in changes {
        let mut observed = observation.clone();
        observed.source = changed.source.clone();
        assert_ne!(baseline.binding_identity(), planned(observed, changed).binding_identity());
    }
    let mut next_activation = request;
    next_activation.activation_identity = identity('5');
    let next = planned(observation, next_activation);
    assert_eq!(baseline.binding_identity(), next.binding_identity());
    assert_ne!(baseline.receipt().unwrap().identity, next.receipt().unwrap().identity);
}

// r[verify lattice.compose_surface.admission]
// r[verify lattice.compose_surface.admission.scenario.valid]
// r[verify lattice.compose_surface.admission.scenario.malformed]
#[test]
fn every_dto_field_is_required_and_unknown_fields_are_closed() {
    let (observation, request) = fixture();
    assert!(admit(observation.clone(), request.clone()).is_ok());
    let observation_json = serde_json::to_value(observation).unwrap();
    let request_json = serde_json::to_value(request).unwrap();
    for (wire, observation_type) in [(observation_json, true), (request_json, false)] {
        let mut paths = alloc::vec![alloc::vec::Vec::<alloc::string::String>::new(), alloc::vec!["source".to_string()]];
        if !observation_type {
            paths.push(alloc::vec!["capability_context".to_string()]);
            paths.push(alloc::vec!["trigger".to_string()]);
        }
        for path in paths {
            let mut at = &wire;
            for segment in &path { at = &at[segment]; }
            let keys = at.as_object().unwrap().keys().cloned().collect::<alloc::vec::Vec<_>>();
            for key in keys {
                let mut bad = wire.clone();
                let mut at = &mut bad;
                for segment in &path { at = &mut at[segment]; }
                at.as_object_mut().unwrap().remove(&key);
                if observation_type {
                    assert!(serde_json::from_value::<PaneObservation>(bad).is_err(), "missing {path:?}/{key}");
                } else {
                    assert!(serde_json::from_value::<BindingRequest>(bad).is_err(), "missing {path:?}/{key}");
                }
            }
            let mut bad = wire.clone();
            let mut at = &mut bad;
            for segment in &path { at = &mut at[segment]; }
            at.as_object_mut().unwrap().insert("raw_screen".to_string(), "secret".into());
            if observation_type { assert!(serde_json::from_value::<PaneObservation>(bad).is_err()); }
            else { assert!(serde_json::from_value::<BindingRequest>(bad).is_err()); }
        }
    }
}

// r[verify lattice.compose_surface.dependency]
// r[verify lattice.compose_surface.dependency.scenario.missing]
// r[verify lattice.compose_surface.dependency.scenario.recorded]
// r[verify lattice.compose_surface.plan]
#[test]
fn stale_missing_unknown_and_unauthorized_sources_reject() {
    let (observation, request) = fixture();
    for (revision, error) in [("", AdmissionError::MissingSourceRevision), ("main", AdmissionError::UnknownSourceRevision), (&"a".repeat(40), AdmissionError::UnknownSourceRevision)] {
        let mut bad = request.clone();
        bad.source.producer_revision = revision.to_string();
        assert_eq!(admit(observation.clone(), bad), Err(error));
    }
    for dimension in 0..3 {
        let mut bad = request.clone();
        match dimension { 0 => bad.source.session_id = 0, 1 => bad.source.pane_id = 0, _ => bad.source.workspace_revision = 0 }
        assert_eq!(admit(observation.clone(), bad), Err(AdmissionError::InvalidSourceIdentity));
    }
    let mut stale = observation.clone();
    stale.source.workspace_revision += 1;
    assert_eq!(admit(stale, request.clone()), Err(AdmissionError::SourceMismatch));
    let mut foreign = observation.clone();
    foreign.source.daemon_identity = identity('7');
    assert_eq!(admit(foreign, request.clone()), Err(AdmissionError::SourceMismatch));
    let mut denied = request.clone();
    denied.capability_context.metadata_binding_allowed = false;
    assert_eq!(admit(observation.clone(), denied), Err(AdmissionError::CapabilityDenied));
    let mut no_match = observation;
    no_match.lifecycle = PaneLifecycle::Exited;
    assert_eq!(admit(no_match, request), Err(AdmissionError::TriggerNotSatisfied));
}

// r[verify lattice.compose_surface.receipt]
// r[verify lattice.compose_surface.receipt.scenario.issued]
// r[verify lattice.compose_surface.receipt.scenario.overclaim]
// r[verify lattice.compose_surface.plan.scenario.intent]
// r[verify lattice.compose_surface.verification]
// r[verify lattice.compose_surface.verification.scenario.matrix]
#[test]
fn receipts_bind_exact_metadata_and_reject_tampering_or_success_claims() {
    let (observation, request) = fixture();
    let plan = planned(observation, request.clone());
    assert_eq!(plan.effect_intent(), EffectIntent::PublishMetadataReceipt);
    let receipt = plan.receipt().unwrap();
    assert_eq!(receipt.source, request.source);
    assert_eq!(receipt.workflow_identity, request.workflow_identity);
    assert_eq!(receipt.capability_identity, request.capability_context.identity);
    assert_eq!(validate_receipt(&receipt, &plan), Ok(()));
    let encoded = serde_json::to_vec(&receipt).unwrap();
    assert!(encoded.len() <= MAXIMUM_RECEIPT_BYTES);
    let mut bad = receipt.clone();
    bad.workflow_identity = identity('8');
    assert_eq!(validate_receipt(&bad, &plan), Err(AdmissionError::ReceiptMismatch));
    let mut wire = serde_json::to_value(&receipt).unwrap();
    wire["claim"] = "workflow_correct".into();
    assert!(serde_json::from_value::<ActivationReceipt>(wire).is_err());
    let mut wire = serde_json::to_value(&receipt).unwrap();
    wire["effect_success"] = true.into();
    assert!(serde_json::from_value::<ActivationReceipt>(wire).is_err());
}

/// Fixed compatibility vectors independently computed with b3sum over the
/// documented length-prefixed canonical frames (612 and 369 bytes).
#[test]
fn canonical_identity_bytes_are_stable_and_upper_numeric_bounds_remain_admissible() {
    let (observation, request) = fixture();
    let baseline = planned(observation.clone(), request.clone());
    assert_eq!(baseline.binding_identity().as_str(), "b3:ae290bfbb5090098b04fefeef43e2b9159f1c463c38ad5971db55716b19332a3");
    assert_eq!(baseline.receipt().unwrap().identity.as_str(), "b3:7f6b771f92c3432cdf17e29e71cce31d838a87a0adb9305843cabe917af328f7");
    let mut upper = request;
    upper.source.session_id = u64::MAX;
    upper.source.pane_id = u64::MAX;
    upper.source.workspace_revision = u64::MAX;
    let mut observed = observation;
    observed.source = upper.source.clone();
    let boundary = planned(observed, upper.clone());
    let receipt = boundary.receipt().unwrap();
    assert_eq!(receipt.source, upper.source);
    assert!(serde_json::to_vec(&receipt).unwrap().len() <= MAXIMUM_RECEIPT_BYTES);
}
