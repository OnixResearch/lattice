//! Metadata-only normalization of trusted component and teardown observations.

use hardened_wasmtime_runtime::outcomes::Class as RuntimeClass;
use hardened_wasmtime_runtime::outcomes::InvocationPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Completed,
    Denied,
    Trapped,
    FuelExhausted,
    DeadlineExceeded,
    Cancelled,
    InvalidOutput,
    ResourceLimit,
    CleanupFailure,
    Unknown,
}

impl From<RuntimeClass> for Class {
    fn from(class: RuntimeClass) -> Self {
        match class {
            RuntimeClass::Completed => Self::Completed,
            RuntimeClass::ProfileDenied
            | RuntimeClass::ArtifactDenied
            | RuntimeClass::CompileFailed
            | RuntimeClass::InstantiateFailed
            | RuntimeClass::GuestDenied => Self::Denied,
            RuntimeClass::GuestTrapped => Self::Trapped,
            RuntimeClass::FuelExhausted => Self::FuelExhausted,
            RuntimeClass::DeadlineExhausted => Self::DeadlineExceeded,
            RuntimeClass::InputLimitExceeded
            | RuntimeClass::OutputLimitExceeded
            | RuntimeClass::MemoryLimitExceeded
            | RuntimeClass::ConcurrencyExhausted => Self::ResourceLimit,
            RuntimeClass::GuestPointerRejected => Self::InvalidOutput,
            RuntimeClass::HostFailed | RuntimeClass::RuntimeFailed => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostReturn {
    NotRequired,
    Succeeded,
    Failed,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub runtime_revision: String,
    pub verifier_revision: String,
    pub runtime_profile_identity: String,
    pub component_identity: String,
    pub wit_identity: String,
    pub bundle_content_identity: String,
    pub canonical_bundle_identity: String,
    pub cohort_identity: String,
    pub runtime_class: Class,
    pub outcome: Class,
    pub store_dropped: bool,
    pub observer_joined: bool,
    pub post_return: PostReturn,
    pub cleanup_failure_cause: Option<Class>,
    pub output_content_omitted: bool,
    pub automatic_retry_permitted: bool,
}

impl Receipt {
    pub(super) fn from_observation(
        prepared: &super::verification::Prepared,
        observation: &super::control::Observation,
    ) -> Self {
        let runtime_class = observation.invocation.as_ref().map_or_else(
            || observation.interruption.unwrap_or(Class::Unknown),
            |invocation| {
                if invocation.class == RuntimeClass::DeadlineExhausted {
                    observation.interruption.unwrap_or(Class::DeadlineExceeded)
                } else {
                    Class::from(invocation.class)
                }
            },
        );
        let post_return = match observation.invocation.as_ref() {
            Some(invocation) if invocation.class == RuntimeClass::Completed => PostReturn::Succeeded,
            Some(invocation) if invocation.phase == InvocationPhase::Invocation => PostReturn::Incomplete,
            _ => PostReturn::NotRequired,
        };
        // Wasmtime combines invocation and automatic cleanup failures. Preserve
        // typed failure, but never infer a separately observed cleanup cause.
        // A completed call wins a racing interruption request.
        let outcome = if observation.observer_joined {
            runtime_class
        } else {
            Class::Unknown
        };
        let dispatch = &prepared.dispatch;
        Self {
            runtime_revision: crate::operation_pack::wasm::RUNTIME_REVISION.to_string(),
            verifier_revision: crate::operation_pack::wasm::VERIFIER_REVISION.to_string(),
            runtime_profile_identity: dispatch.descriptor.runtime_profile_identity.clone(),
            component_identity: dispatch.component_identity.clone(),
            wit_identity: dispatch.wit_identity.clone(),
            bundle_content_identity: dispatch.bundle_identity.clone(),
            canonical_bundle_identity: prepared.canonical_bundle_identity.clone(),
            cohort_identity: dispatch.cohort_identity.clone(),
            runtime_class,
            outcome,
            store_dropped: true, // run() returned after its local store was dropped, or never ran
            observer_joined: observation.observer_joined,
            post_return,
            cleanup_failure_cause: None,
            output_content_omitted: true,
            automatic_retry_permitted: false,
        }
    }
}

pub(super) fn failed(class: Class) -> crate::operation::runtime::Observation {
    let observation_class = if class == Class::Unknown {
        crate::operation::runtime::ObservationClass::Unknown
    } else {
        crate::operation::runtime::ObservationClass::DeterministicFailure
    };
    crate::operation::runtime::Observation {
        class: observation_class,
        output: None,
        output_identity: None,
        output_bytes: 0,
        stderr_bytes: 0,
        diagnostic: format!("component operation outcome: {class:?}"),
        redacted: true,
    }
}

pub(super) fn completed(
    result: serde_json::Value,
) -> Result<crate::operation::runtime::Observation, crate::operation_pack::Error> {
    let output_identity =
        crate::operation::identity::hash_value(crate::operation::runtime::OUTPUT_IDENTITY_DOMAIN, &result)?;
    let output_bytes = serde_json::to_vec(&result).map_err(|_| crate::operation_pack::Error::Encoding)?.len();
    Ok(crate::operation::runtime::Observation {
        class: crate::operation::runtime::ObservationClass::Completed,
        output: Some(result),
        output_identity: Some(output_identity),
        output_bytes,
        stderr_bytes: 0,
        diagnostic: "component result admitted after separately observed effects".to_string(),
        redacted: true,
    })
}
