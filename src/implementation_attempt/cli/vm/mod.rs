//! VM-bound request validation (Lattice #10): `vm_binding` (C2) and
//! `vm_profile` (C1) of a `lattice.implementation-attempt-run.v3` request.
//! Both are per-attempt values bound by the exact request bytes, not the
//! profile revision; absent, the request decodes, serializes and behaves
//! exactly as before.

// r[impl lattice.vm_profile.request.scenario.absent]

#[cfg(test)]
mod tests;

/// A VM-bound request runs only through its host gateway: a declared
/// private network, the binding's own envelope, attempt and fence, an
/// offered mobility mode, and a forwarding broker of exactly this binding
/// without a credential. A request without a binding cannot forward, and
/// a bound request never falls back to a host-process broker.
// r[impl lattice.vm_profile.request]
pub(super) fn validate(request: &super::Request) -> Result<(), String> {
    let broker = &request.omp.model_broker;
    let (binding, profile, forwarded) = match (&request.vm_binding, &request.vm_profile, &broker.forwarded) {
        (None, None, None) => return Ok(()),
        (Some(binding), Some(profile), Some(forwarded)) => (binding, profile, forwarded),
        (Some(_) | None, Some(_) | None, Some(_) | None) => {
            return Err("VM binding, VM profile and forwarding model broker must be present together".into());
        }
    };
    if request.network != Some(false) {
        return Err("a VM-bound request must declare `network: false`".into());
    }
    let admitted = crate::vm_profile::admit_binding(binding, &crate::vm_profile::Expected {
        profile,
        envelope: Some(request.admitted_envelope.as_str()),
        attempt_id: Some(request.envelope.attempt_id.as_str()),
        fence: Some(request.envelope.fence),
        guest_image: None,
        environment: None,
        execution: None,
        workspace: None,
        deadline_unix_ms: None,
    })
    .map_err(|refusal| refusal.to_string())?;
    // An attempt has no node-boundary checkpoint hooks: it never restores.
    if binding.requested_mobility == crate::vm_profile::Mobility::ApplicationCheckpoint {
        return Err("an implementation attempt offers no application checkpoint; `application_checkpoint` \
                    mobility is refused"
            .into());
    }
    if forwarded.binding != admitted.binding_identity || forwarded.vsock_port != profile.gateway.model_vsock_port {
        return Err("forwarding model broker names another VM binding or gateway port".into());
    }
    if broker.credential.is_some() {
        return Err("a VM-bound request carries no model credential; the host gateway holds it".into());
    }
    debug_assert_eq!(binding.envelope, request.admitted_envelope);
    debug_assert!(profile.mobility.contains(&binding.requested_mobility));
    Ok(())
}
