//! Pure admission and identities of C1 and C2. No I/O.

// r[impl lattice.vm_profile.admission]
// r[impl lattice.vm_profile.binding]
// r[impl lattice.vm_profile.identity]

/// Every profile states at least these non-claims.
pub const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "vm execution profile admission does not prove cross-host mobility",
    "vm execution profile admission does not prove live migration",
    "vm execution profile admission does not qualify a production placement authority",
    "vm execution profile admission does not prove a live model endpoint",
];
const MAX_NON_CLAIMS: usize = 16;
const MAX_TEXT_BYTES: usize = 256;
const MAX_VERSION_BYTES: usize = 64;
const MAX_ATTEMPT_ID_BYTES: usize = 128;
const IDENTITY_PREFIX: &str = "b3:";

/// BLAKE3 over `u64be(len(domain)) ‖ domain ‖ u64be(len(json)) ‖ json`,
/// rendered `b3:<64 hex>`. The domain is the record's schema string.
pub fn framed_identity<T: serde::Serialize>(domain: &str, value: &T) -> Result<String, super::Refusal> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::Refusal::Identity)?;
    let domain_bytes = u64::try_from(domain.len()).map_err(|_| super::Refusal::Identity)?;
    let value_bytes = u64::try_from(bytes.len()).map_err(|_| super::Refusal::Identity)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_bytes.to_be_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(&value_bytes.to_be_bytes());
    hasher.update(&bytes);
    let identity = format!("{IDENTITY_PREFIX}{}", hasher.finalize().to_hex());
    debug_assert!(is_identity(&identity));
    Ok(identity)
}

/// The `b3:` identity of a profile, admitted or not.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "C1's identity is the profile identity named by the relocation contract"
)]
pub fn profile_identity(profile: &super::ExecutionProfile) -> Result<String, super::Refusal> {
    framed_identity(super::PROFILE_SCHEMA, profile)
}

/// The `b3:` identity of a binding, admitted or not.
pub fn binding_identity(binding: &super::AttemptBinding) -> Result<String, super::Refusal> {
    framed_identity(super::BINDING_SCHEMA, binding)
}

/// 64 lowercase hexadecimal characters (a Kiln `ContentId` or an envelope identity).
#[must_use]
pub fn is_content_id(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A `b3:`-prefixed content identity.
#[must_use]
pub fn is_identity(value: &str) -> bool {
    value.strip_prefix(IDENTITY_PREFIX).is_some_and(is_content_id)
}

fn is_bounded_text(value: &str, maximum_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= maximum_bytes && !value.chars().any(char::is_control)
}

/// Admit one profile and return its identity.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "C1 admission is the profile admission named by the relocation contract"
)]
pub fn admit_profile(profile: &super::ExecutionProfile) -> Result<String, super::Refusal> {
    admit_guest(profile)?;
    admit_gateway(&profile.gateway)?;
    admit_modes(profile)?;
    if profile.deadline_authority != super::DEADLINE_AUTHORITY_HOST {
        return Err(super::Refusal::DeadlineAuthorityNotHost);
    }
    let resources = &profile.resources;
    if resources.vcpus == 0 || resources.memory_mib == 0 || resources.workspace_mib == 0 {
        return Err(super::Refusal::InvalidResources);
    }
    admit_devices(&profile.devices)?;
    admit_non_claims(&profile.non_claims)?;
    debug_assert!(!profile.mobility.contains(&super::Mobility::LivePrecopy));
    debug_assert!(profile.devices.iter().any(|device| device == "vsock"));
    profile_identity(profile)
}

/// Schema, VMM, guest manifest and the absent guest network.
fn admit_guest(profile: &super::ExecutionProfile) -> Result<(), super::Refusal> {
    if profile.schema != super::PROFILE_SCHEMA {
        return Err(super::Refusal::UnsupportedSchema);
    }
    let version = profile.vmm.version.as_str();
    if !is_bounded_text(version, MAX_VERSION_BYTES) || version.contains(char::is_whitespace) {
        return Err(super::Refusal::UnsupportedVmm);
    }
    if profile.guest_manifest_schema != super::GUEST_MANIFEST_SCHEMA {
        return Err(super::Refusal::UnsupportedGuestManifest);
    }
    if profile.guest_network != super::GUEST_NETWORK_NONE {
        return Err(super::Refusal::GuestNetworkNotNone);
    }
    Ok(())
}

fn admit_gateway(gateway: &super::Gateway) -> Result<(), super::Refusal> {
    if gateway.exchange_transport_schema != super::EXCHANGE_TRANSPORT_SCHEMA {
        return Err(super::Refusal::UnsupportedExchangeTransport);
    }
    if gateway.host_peer_cid != super::HOST_PEER_CID {
        return Err(super::Refusal::HostPeerCid);
    }
    let ports = [gateway.exchange_vsock_port, gateway.model_vsock_port];
    if !ports.iter().all(|port| port.is_concrete()) || gateway.exchange_vsock_port == gateway.model_vsock_port {
        return Err(super::Refusal::InvalidGatewayPorts);
    }
    Ok(())
}

/// Modes are nonempty and strictly ascending in declaration order, so the
/// set has one canonical encoding.
fn admit_modes(profile: &super::ExecutionProfile) -> Result<(), super::Refusal> {
    let modes = &profile.mobility;
    if modes.is_empty() || !modes.windows(2).all(|pair| pair.first() < pair.last()) {
        return Err(super::Refusal::InvalidMobility);
    }
    if modes.contains(&super::Mobility::LivePrecopy) {
        return Err(super::Refusal::LivePrecopyUnqualified);
    }
    let needs_checkpoint = modes.contains(&super::Mobility::ApplicationCheckpoint);
    if needs_checkpoint && profile.checkpoint == super::Checkpoint::NotOffered {
        return Err(super::Refusal::CheckpointRequired);
    }
    Ok(())
}

/// Devices are a strictly ascending subset of the admitted set that includes `vsock`.
fn admit_devices(devices: &[String]) -> Result<(), super::Refusal> {
    let is_admitted = devices.iter().all(|device| super::ADMITTED_DEVICES.contains(&device.as_str()));
    let is_canonical = devices.windows(2).all(|pair| pair.first() < pair.last());
    let has_vsock = devices.iter().any(|device| device == "vsock");
    if !is_admitted || !is_canonical || !has_vsock {
        return Err(super::Refusal::UnsupportedDevice);
    }
    Ok(())
}

fn admit_non_claims(non_claims: &[String]) -> Result<(), super::Refusal> {
    if non_claims.len() > MAX_NON_CLAIMS || !non_claims.iter().all(|claim| is_bounded_text(claim, MAX_TEXT_BYTES)) {
        return Err(super::Refusal::InvalidNonClaims);
    }
    let is_unique = non_claims
        .iter()
        .enumerate()
        .all(|(index, claim)| non_claims.get(..index).is_some_and(|prior| !prior.contains(claim)));
    let is_complete = REQUIRED_NON_CLAIMS.iter().all(|required| non_claims.iter().any(|claim| claim == required));
    if !is_unique || !is_complete {
        return Err(super::Refusal::InvalidNonClaims);
    }
    Ok(())
}

/// Kiln's drain disposition of one mobility mode.
#[must_use]
pub const fn drain_disposition(mode: super::Mobility) -> super::DrainDisposition {
    match mode {
        super::Mobility::Pinned => super::DrainDisposition::Pinned,
        super::Mobility::StoppedTransfer | super::Mobility::LivePrecopy => super::DrainDisposition::Migrated,
        super::Mobility::ApplicationCheckpoint => super::DrainDisposition::Checkpointed,
    }
}

/// Admit one requested mode of an admitted profile, before any effect.
pub fn admit_mobility(
    profile: &super::ExecutionProfile,
    mode: super::Mobility,
) -> Result<super::DrainDisposition, super::Refusal> {
    admit_profile(profile)?;
    if !profile.mobility.contains(&mode) {
        return Err(super::Refusal::MobilityNotOffered);
    }
    debug_assert!(mode != super::Mobility::LivePrecopy, "admitted profiles never offer live precopy");
    Ok(drain_disposition(mode))
}

fn is_well_formed(binding: &super::AttemptBinding) -> bool {
    let content = [
        binding.guest_image.as_str(),
        binding.environment.as_str(),
        binding.envelope.as_str(),
        binding.execution.as_str(),
        binding.workspace.identity.as_str(),
    ];
    is_identity(&binding.profile)
        && content.iter().all(|value| is_content_id(value))
        && is_bounded_text(&binding.attempt_id, MAX_ATTEMPT_ID_BYTES)
        && binding.fence != 0
        && binding.deadline_unix_ms != 0
}

fn crossed<T: PartialEq + ?Sized>(
    expected: Option<&T>,
    actual: &T,
    refusal: super::Refusal,
) -> Result<(), super::Refusal> {
    match expected {
        Some(value) if value != actual => Err(refusal),
        Some(_) | None => Ok(()),
    }
}

/// Admit one binding against its profile and every value the verifier knows.
pub fn admit_binding(
    binding: &super::AttemptBinding,
    expected: &super::Expected<'_>,
) -> Result<super::AdmittedBinding, super::Refusal> {
    let profile_identity = admit_profile(expected.profile)?;
    if binding.schema != super::BINDING_SCHEMA {
        return Err(super::Refusal::UnsupportedSchema);
    }
    if !is_well_formed(binding) {
        return Err(super::Refusal::MalformedBinding);
    }
    if binding.profile != profile_identity {
        return Err(super::Refusal::CrossedProfile);
    }
    crossed(expected.guest_image, binding.guest_image.as_str(), super::Refusal::CrossedGuestImage)?;
    crossed(expected.environment, binding.environment.as_str(), super::Refusal::CrossedEnvironment)?;
    crossed(expected.envelope, binding.envelope.as_str(), super::Refusal::CrossedEnvelope)?;
    crossed(expected.attempt_id, binding.attempt_id.as_str(), super::Refusal::CrossedAttempt)?;
    crossed(expected.fence.as_ref(), &binding.fence, super::Refusal::CrossedFence)?;
    crossed(expected.execution, binding.execution.as_str(), super::Refusal::CrossedExecution)?;
    crossed(expected.workspace, &binding.workspace, super::Refusal::CrossedWorkspace)?;
    crossed(expected.deadline_unix_ms.as_ref(), &binding.deadline_unix_ms, super::Refusal::CrossedDeadline)?;
    let disposition = admit_mobility(expected.profile, binding.requested_mobility)?;
    debug_assert!(is_identity(&binding.profile));
    debug_assert!(expected.profile.mobility.contains(&binding.requested_mobility));
    Ok(super::AdmittedBinding {
        profile_identity,
        binding_identity: binding_identity(binding)?,
        disposition,
    })
}
