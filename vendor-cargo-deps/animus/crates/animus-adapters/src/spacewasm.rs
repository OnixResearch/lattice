//! Pure admission and receipt contracts for the opt-in SpaceWasm diagnostic lane.

use alloc::string::ToString;

/// Exact reviewed NASA `SpaceWasm` revision.
pub const SPACEWASM_REVISION: &str = "e24cf09355a90497148eb5029fdb8e3400bd63e3";
/// Exact Octet runtime cohort identity.
pub const SPACEWASM_COHORT: &str =
    "b3:7c537eef7bff66d0a90508715de08de79c9aaac23352e1007a6d47e7fb4674a2";
/// Exact upstream lock identity.
pub const SPACEWASM_LOCK: &str =
    "b3:4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1";
/// Exact supported producer target.
pub const PRODUCER_TARGET: &str = "wasm32v1-none";
/// Exact required export.
pub const REQUIRED_EXPORT: &str = "run";
/// Maximum admitted artifact size.
pub const MAX_ARTIFACT_BYTES: u64 = 1_048_576;
/// Maximum admitted input size.
pub const MAX_INPUT_BYTES: u64 = 4_096;
/// Maximum admitted output size.
pub const MAX_OUTPUT_BYTES: u64 = 4_096;
/// Maximum admitted fuel observation.
pub const MAX_FUEL: u64 = 1_000_000;
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_PREFIX_BYTES: usize = 3;
const BLAKE3_IDENTITY_BYTES: usize = 67;
const OCTET_PROFILE: &str = "spacewasm-mvp";
const REQUIRED_PROPOSAL: &str = "wasm1";
#[cfg(test)]
const RECEIPT_NON_CLAIM_COUNT: usize = 5;

/// Static profile supplied by the checked Nickel projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile<'a> {
    /// Explicit enablement.
    pub enabled: bool,
    /// Producer target.
    pub target: &'a str,
    /// Runtime source revision.
    pub runtime_revision: &'a str,
    /// Runtime cohort identity.
    pub runtime_cohort: &'a str,
    /// Upstream lock identity.
    pub source_lock: &'a str,
    /// Declared exports.
    pub exports: &'a [&'a str],
    /// Declared imports.
    pub imports: &'a [&'a str],
    /// Artifact byte limit.
    pub max_artifact_bytes: u64,
    /// Input byte limit.
    pub max_input_bytes: u64,
    /// Output byte limit.
    pub max_output_bytes: u64,
    /// Fuel limit.
    pub max_fuel: u64,
}

/// Static exact-byte and Octet observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactFacts<'a> {
    /// Domain-separated exact-byte identity.
    pub artifact_identity: &'a str,
    /// Observed artifact length.
    pub artifact_bytes: u64,
    /// Octet profile identifier.
    pub octet_profile: &'a str,
    /// Octet-reported artifact identity.
    pub octet_artifact_identity: &'a str,
    /// Octet-reported cohort identity.
    pub octet_cohort: &'a str,
    /// Observed exports.
    pub exports: &'a [&'a str],
    /// Observed imports.
    pub imports: &'a [&'a str],
    /// Observed proposal labels.
    pub proposals: &'a [&'a str],
}

/// Normalized result returned by an injected runtime service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation<'a> {
    /// Exact artifact identity executed.
    pub artifact_identity: &'a str,
    /// Exact runtime cohort used.
    pub runtime_cohort: &'a str,
    /// Input identity.
    pub input_identity: &'a str,
    /// Output identity when output exists.
    pub output_identity: Option<&'a str>,
    /// Input byte count.
    pub input_bytes: u64,
    /// Output byte count.
    pub output_bytes: u64,
    /// Fuel observation.
    pub fuel: u64,
    /// Result class.
    pub result: ResultClass,
}

/// Bounded runtime result classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultClass {
    /// Declared operation returned an admitted output.
    Returned,
    /// Declared trap fixture trapped.
    ExpectedTrap,
    /// Runtime exhausted its declared resource bound.
    ResourceExhausted,
    /// Runtime setup or execution failed.
    RuntimeFailure,
}

/// Deterministic admission failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Profile is not explicitly enabled.
    Disabled,
    /// Producer target or runtime cohort drifted.
    CohortDrift,
    /// Surface contains an undeclared export, import, or proposal.
    SurfaceWidening,
    /// A finite byte or fuel limit was violated.
    ResourceLimit,
    /// Exact-byte evidence disagrees.
    EvidenceMismatch,
    /// A safe identity or result is malformed.
    MalformedObservation,
    /// Receipt text would leak unsafe material or overclaim.
    UnsafeReceipt,
}

/// Metadata-only compatibility receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// Exact guest identity.
    pub artifact_identity: alloc::string::String,
    /// Exact runtime cohort.
    pub runtime_cohort: alloc::string::String,
    /// Octet profile.
    pub octet_profile: alloc::string::String,
    /// Input identity.
    pub input_identity: alloc::string::String,
    /// Optional output identity.
    pub output_identity: Option<alloc::string::String>,
    /// Result class.
    pub result: ResultClass,
    /// Stable non-claims.
    pub non_claims: alloc::vec::Vec<&'static str>,
}

/// Admit a checked profile and exact static artifact report.
///
/// # Errors
///
/// Returns a deterministic denial when profile, surface, resource, or evidence facts drift.
pub fn admit(profile: &Profile<'_>, artifact: &ArtifactFacts<'_>) -> Result<(), Error> {
    if !profile.enabled {
        return Err(Error::Disabled);
    }
    validate_profile_cohort(profile)?;
    validate_surface(profile, artifact)?;
    validate_resource_limits(profile, artifact)?;
    validate_artifact_evidence(profile, artifact)
}

fn validate_profile_cohort(profile: &Profile<'_>) -> Result<(), Error> {
    if profile.target != PRODUCER_TARGET {
        return Err(Error::CohortDrift);
    }
    if profile.runtime_revision != SPACEWASM_REVISION {
        return Err(Error::CohortDrift);
    }
    if profile.runtime_cohort != SPACEWASM_COHORT {
        return Err(Error::CohortDrift);
    }
    if profile.source_lock != SPACEWASM_LOCK {
        return Err(Error::CohortDrift);
    }
    debug_assert_eq!(profile.target, PRODUCER_TARGET);
    debug_assert_eq!(profile.runtime_revision, SPACEWASM_REVISION);
    debug_assert_eq!(profile.runtime_cohort, SPACEWASM_COHORT);
    debug_assert_eq!(profile.source_lock, SPACEWASM_LOCK);
    Ok(())
}

fn validate_surface(profile: &Profile<'_>, artifact: &ArtifactFacts<'_>) -> Result<(), Error> {
    if profile.exports != [REQUIRED_EXPORT] {
        return Err(Error::SurfaceWidening);
    }
    if !profile.imports.is_empty() {
        return Err(Error::SurfaceWidening);
    }
    if artifact.exports != [REQUIRED_EXPORT] {
        return Err(Error::SurfaceWidening);
    }
    if !artifact.imports.is_empty() {
        return Err(Error::SurfaceWidening);
    }
    if artifact.proposals != [REQUIRED_PROPOSAL] {
        return Err(Error::SurfaceWidening);
    }
    debug_assert_eq!(profile.exports, [REQUIRED_EXPORT]);
    debug_assert!(profile.imports.is_empty());
    debug_assert_eq!(artifact.exports, [REQUIRED_EXPORT]);
    debug_assert!(artifact.imports.is_empty());
    debug_assert_eq!(artifact.proposals, [REQUIRED_PROPOSAL]);
    Ok(())
}

fn validate_resource_limits(
    profile: &Profile<'_>,
    artifact: &ArtifactFacts<'_>,
) -> Result<(), Error> {
    if profile.max_artifact_bytes != MAX_ARTIFACT_BYTES {
        return Err(Error::ResourceLimit);
    }
    if profile.max_input_bytes != MAX_INPUT_BYTES {
        return Err(Error::ResourceLimit);
    }
    if profile.max_output_bytes != MAX_OUTPUT_BYTES {
        return Err(Error::ResourceLimit);
    }
    if profile.max_fuel != MAX_FUEL {
        return Err(Error::ResourceLimit);
    }
    if artifact.artifact_bytes > profile.max_artifact_bytes {
        return Err(Error::ResourceLimit);
    }
    debug_assert_eq!(profile.max_artifact_bytes, MAX_ARTIFACT_BYTES);
    debug_assert_eq!(profile.max_input_bytes, MAX_INPUT_BYTES);
    debug_assert_eq!(profile.max_output_bytes, MAX_OUTPUT_BYTES);
    debug_assert_eq!(profile.max_fuel, MAX_FUEL);
    debug_assert!(artifact.artifact_bytes <= profile.max_artifact_bytes);
    Ok(())
}

fn validate_artifact_evidence(
    profile: &Profile<'_>,
    artifact: &ArtifactFacts<'_>,
) -> Result<(), Error> {
    if !safe_identity(artifact.artifact_identity) {
        return Err(Error::EvidenceMismatch);
    }
    if artifact.octet_profile != OCTET_PROFILE {
        return Err(Error::EvidenceMismatch);
    }
    if artifact.octet_artifact_identity != artifact.artifact_identity {
        return Err(Error::EvidenceMismatch);
    }
    if artifact.octet_cohort != profile.runtime_cohort {
        return Err(Error::EvidenceMismatch);
    }
    debug_assert!(safe_identity(artifact.artifact_identity));
    debug_assert_eq!(artifact.octet_profile, OCTET_PROFILE);
    debug_assert_eq!(artifact.octet_artifact_identity, artifact.artifact_identity);
    debug_assert_eq!(artifact.octet_cohort, profile.runtime_cohort);
    Ok(())
}

/// Classify one normalized injected-service observation and project a safe receipt.
///
/// # Errors
///
/// Returns a deterministic denial when admission or normalized observation checks fail.
pub fn classify(
    profile: &Profile<'_>,
    artifact: &ArtifactFacts<'_>,
    observation: &Observation<'_>,
) -> Result<Receipt, Error> {
    admit(profile, artifact)?;
    if observation.artifact_identity != artifact.artifact_identity {
        return Err(Error::EvidenceMismatch);
    }
    if observation.runtime_cohort != profile.runtime_cohort {
        return Err(Error::EvidenceMismatch);
    }
    if !safe_identity(observation.input_identity) {
        return Err(Error::EvidenceMismatch);
    }
    if observation
        .output_identity
        .is_some_and(|value| !safe_identity(value))
    {
        return Err(Error::EvidenceMismatch);
    }
    debug_assert_eq!(observation.artifact_identity, artifact.artifact_identity);
    debug_assert_eq!(observation.runtime_cohort, profile.runtime_cohort);
    debug_assert!(safe_identity(observation.input_identity));
    debug_assert!(observation.output_identity.is_none_or(safe_identity));

    if observation.input_bytes > profile.max_input_bytes {
        return Err(Error::ResourceLimit);
    }
    if observation.output_bytes > profile.max_output_bytes {
        return Err(Error::ResourceLimit);
    }
    if observation.fuel > profile.max_fuel {
        return Err(Error::ResourceLimit);
    }
    debug_assert!(observation.input_bytes <= profile.max_input_bytes);
    debug_assert!(observation.output_bytes <= profile.max_output_bytes);
    debug_assert!(observation.fuel <= profile.max_fuel);

    if observation.result == ResultClass::RuntimeFailure {
        return Err(Error::MalformedObservation);
    }
    debug_assert_ne!(observation.result, ResultClass::RuntimeFailure);
    Ok(Receipt {
        artifact_identity: observation.artifact_identity.to_string(),
        runtime_cohort: observation.runtime_cohort.to_string(),
        octet_profile: artifact.octet_profile.to_string(),
        input_identity: observation.input_identity.to_string(),
        output_identity: observation.output_identity.map(ToString::to_string),
        result: observation.result,
        non_claims: alloc::vec![
            "not-general-wasm-runtime-support",
            "not-plugin-activation",
            "not-runtime-correctness",
            "not-sandbox-completeness",
            "not-production-or-release-readiness",
        ],
    })
}

/// Reject unsafe receipt text and unsupported claim promotion.
///
/// # Errors
///
/// Returns [`Error::UnsafeReceipt`] when denied material or claims appear.
pub fn validate_receipt_text(text: &str) -> Result<(), Error> {
    const DENIED: [&str; 8] = [
        "password",
        "credential",
        "/home/",
        "raw_prompt",
        "provider_payload",
        "production-ready",
        "sandbox-complete",
        "plugin-enabled",
    ];
    if DENIED.iter().any(|needle| text.contains(needle)) {
        return Err(Error::UnsafeReceipt);
    }
    Ok(())
}

fn safe_identity(value: &str) -> bool {
    value.starts_with(BLAKE3_PREFIX)
        && value.len() == BLAKE3_IDENTITY_BYTES
        && value.as_bytes()[BLAKE3_PREFIX_BYTES..]
            .iter()
            .all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "b3:0000000000000000000000000000000000000000000000000000000000000000";

    fn profile() -> Profile<'static> {
        Profile {
            enabled: true,
            target: PRODUCER_TARGET,
            runtime_revision: SPACEWASM_REVISION,
            runtime_cohort: SPACEWASM_COHORT,
            source_lock: SPACEWASM_LOCK,
            exports: &[REQUIRED_EXPORT],
            imports: &[],
            max_artifact_bytes: MAX_ARTIFACT_BYTES,
            max_input_bytes: MAX_INPUT_BYTES,
            max_output_bytes: MAX_OUTPUT_BYTES,
            max_fuel: MAX_FUEL,
        }
    }

    fn artifact() -> ArtifactFacts<'static> {
        ArtifactFacts {
            artifact_identity: ID,
            artifact_bytes: 128,
            octet_profile: OCTET_PROFILE,
            octet_artifact_identity: ID,
            octet_cohort: SPACEWASM_COHORT,
            exports: &[REQUIRED_EXPORT],
            imports: &[],
            proposals: &[REQUIRED_PROPOSAL],
        }
    }

    #[test]
    fn exact_profile_artifact_and_results_are_deterministic() {
        let observation = Observation {
            artifact_identity: ID,
            runtime_cohort: SPACEWASM_COHORT,
            input_identity: ID,
            output_identity: Some(ID),
            input_bytes: 4,
            output_bytes: 4,
            fuel: 10,
            result: ResultClass::Returned,
        };
        let first = classify(&profile(), &artifact(), &observation).expect("admitted");
        let second = classify(&profile(), &artifact(), &observation).expect("admitted");
        assert_eq!(first, second);
        assert_eq!(first.non_claims.len(), RECEIPT_NON_CLAIM_COUNT);
    }

    #[test]
    fn expected_trap_is_a_bounded_result() {
        let observation = Observation {
            artifact_identity: ID,
            runtime_cohort: SPACEWASM_COHORT,
            input_identity: ID,
            output_identity: None,
            input_bytes: 1,
            output_bytes: 0,
            fuel: 10,
            result: ResultClass::ExpectedTrap,
        };
        assert_eq!(
            classify(&profile(), &artifact(), &observation)
                .expect("trap")
                .result,
            ResultClass::ExpectedTrap
        );
    }

    #[test]
    fn disabled_drift_widening_tampering_and_limits_fail_closed() {
        let mut disabled = profile();
        disabled.enabled = false;
        assert_eq!(admit(&disabled, &artifact()), Err(Error::Disabled));
        let mut drifted = profile();
        drifted.target = "wasm32-unknown-unknown";
        assert_eq!(admit(&drifted, &artifact()), Err(Error::CohortDrift));
        let mut widened = artifact();
        widened.imports = &["wasi_snapshot_preview1"];
        assert_eq!(admit(&profile(), &widened), Err(Error::SurfaceWidening));
        let mut stale = artifact();
        stale.octet_artifact_identity =
            "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(admit(&profile(), &stale), Err(Error::EvidenceMismatch));
        let mut huge = artifact();
        huge.artifact_bytes = MAX_ARTIFACT_BYTES.saturating_add(1);
        assert_eq!(admit(&profile(), &huge), Err(Error::ResourceLimit));
    }

    #[test]
    fn service_failure_leaks_and_overclaims_are_rejected() {
        let failed = Observation {
            artifact_identity: ID,
            runtime_cohort: SPACEWASM_COHORT,
            input_identity: ID,
            output_identity: None,
            input_bytes: 0,
            output_bytes: 0,
            fuel: 0,
            result: ResultClass::RuntimeFailure,
        };
        assert_eq!(
            classify(&profile(), &artifact(), &failed),
            Err(Error::MalformedObservation)
        );
        assert_eq!(
            validate_receipt_text("credential=/home/user production-ready"),
            Err(Error::UnsafeReceipt)
        );
        assert_eq!(
            validate_receipt_text("artifact=b3:00 not-runtime-correctness"),
            Ok(())
        );
    }
}
