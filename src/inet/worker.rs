//! Thin imperative adapter between the existing operation process runtime
//! and the pinned Noble executable.
//!
//! Before dispatch it remeasures the located program and input bytes against
//! their admitted identities and lengths, re-admits them through the closed
//! vocabulary and the host envelope, and builds the producer argv. After the
//! existing runner returns, it normalizes the report and builds one bounded
//! result or non-authoritative diagnostic document.

pub mod document;

const VOCABULARY: &[u8] = b"lattice.pure-inet-vocabulary.v1:input,const,i64.add,i64.sub,i64.mul;wrapping-i64";
const MILLIS_PER_SECOND: u64 = 1_000;

/// Remeasure the artifacts and build the exact producer process request.
pub fn process_request(
    plan: &crate::operation::InvocationPlan,
) -> Result<crate::agent_node::ProcessRequest, super::Error> {
    let request = crate::inet::route::arguments::decode(plan)?;
    let program_path = request.artifact_root.join(&request.program);
    let inputs_path = request.artifact_root.join(&request.inputs);
    let program_bytes = read_exact(&program_path, request.program_bytes)?;
    let inputs_bytes = read_exact(&inputs_path, request.inputs_bytes)?;
    crate::pure_inet_profile::verify_identity(&request.program_identity, &program_bytes)?;
    crate::pure_inet_profile::verify_identity(&request.inputs_identity, &inputs_bytes)?;
    binding(plan, &request)?.validate()?;
    limits(plan, &request)?.validate()?;
    super::text::parse_program(&program_bytes)?;
    super::text::parse_inputs(&inputs_bytes)?;
    let path = |path: &std::path::Path| path.to_str().map(str::to_string).ok_or(super::Error::ArtifactUnavailable);
    let mut args = vec![
        "ngraph".to_string(),
        "eval".to_string(),
        path(&program_path)?,
        path(&inputs_path)?,
    ];
    args.push("--net".to_string());
    args.extend(request.ceilings.flags());
    debug_assert_eq!(args.len(), 13);
    Ok(crate::agent_node::ProcessRequest {
        program: plan.runtime.executable_path.clone(),
        args,
        cwd: Some(path(&request.artifact_root)?),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: plan.bounds.timeout_seconds,
        output_limit_bytes: plan.bounds.output_bytes,
    })
}

/// Read one located artifact without following links, bounded by its
/// admitted length.
fn read_exact(path: &std::path::Path, expected_bytes: u64) -> Result<Vec<u8>, super::Error> {
    use std::io::Read as _;
    let metadata = std::fs::symlink_metadata(path).map_err(|_| super::Error::ArtifactUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != expected_bytes {
        return Err(super::Error::Core(crate::pure_inet_profile::PureProfileError::IdentityMismatch));
    }
    let file = std::fs::File::open(path).map_err(|_| super::Error::ArtifactUnavailable)?;
    let capacity_bytes = usize::try_from(expected_bytes).map_err(|_| super::Error::ArtifactUnavailable)?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.take(expected_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| super::Error::ArtifactUnavailable)?;
    if u64::try_from(bytes.len()).ok() != Some(expected_bytes) {
        return Err(super::Error::Core(crate::pure_inet_profile::PureProfileError::IdentityMismatch));
    }
    Ok(bytes)
}

fn hex(identity: &str) -> Result<String, super::Error> {
    identity
        .strip_prefix(crate::operation::model::BLAKE3_PREFIX)
        .map(str::to_string)
        .ok_or(super::Error::Core(crate::pure_inet_profile::PureProfileError::InvalidBindingIdentity))
}

/// The host envelope binding of this invocation.
fn binding(
    plan: &crate::operation::InvocationPlan,
    request: &crate::inet::route::arguments::Request,
) -> Result<crate::pure_inet_profile::PureComputationBinding, super::Error> {
    let contract = serde_json::to_vec(&super::report::NET_CONTRACT)
        .map_err(|_| super::Error::Core(crate::pure_inet_profile::PureProfileError::InvalidSchema))?;
    debug_assert!(!request.attempt.is_empty());
    debug_assert!(!contract.is_empty());
    Ok(crate::pure_inet_profile::PureComputationBinding {
        workflow_revision: hex(&plan.catalog_identity)?,
        instance_id: plan.instance_id.clone(),
        dispatch_id: plan.identity.clone(),
        run_id: request.attempt.clone(),
        attempt_id: request.attempt.clone(),
        program_identity: request.program_identity.clone(),
        input_identity: request.inputs_identity.clone(),
        producer_revision: super::report::PRODUCER_SOURCE_REVISION.to_string(),
        rule_revision: super::report::NET_CONTRACT[2].1.to_string(),
        runtime_identity: crate::pure_inet_profile::blake3_identity(&contract),
        executable_identity: hex(&plan.runtime.executor_identity)?,
        profile_identity: hex(&plan.type_revision)?,
        extrinsic_profile_identity: crate::pure_inet_profile::blake3_identity(VOCABULARY),
        output_contract_identity: hex(&plan.output_contract_identity)?,
    })
}

fn limits(
    plan: &crate::operation::InvocationPlan,
    request: &crate::inet::route::arguments::Request,
) -> Result<crate::pure_inet_profile::PureLimits, super::Error> {
    let bytes = |value: u64| {
        usize::try_from(value)
            .map_err(|_| super::Error::Core(crate::pure_inet_profile::PureProfileError::InvalidLimits))
    };
    Ok(crate::pure_inet_profile::PureLimits {
        max_program_operations: crate::pure_inet_profile::MAX_PROGRAM_OPERATIONS,
        max_input_bytes: bytes(request.ceilings.max_input_bytes)?,
        max_output_bytes: bytes(request.ceilings.max_output_bytes)?,
        deadline_millis: plan.bounds.timeout_seconds.saturating_mul(MILLIS_PER_SECOND),
    })
}

/// Classify one completed process observation of this runtime. Process
/// timeouts, cancellation and truncation never reach this function; they
/// stay with the existing process-failure classification.
pub fn observe(
    plan: &crate::operation::InvocationPlan,
    output: &crate::agent_node::ProcessOutput,
) -> crate::operation::runtime::Observation {
    let class_and_document = crate::inet::route::arguments::decode(plan).and_then(|request| {
        let expectation = super::report::Expectation {
            source_revision: super::report::PRODUCER_SOURCE_REVISION,
            input_bytes: request.program_bytes.saturating_add(request.inputs_bytes),
            ceilings: request.ceilings,
        };
        let normalized = super::report::normalize(&output.stdout, output.status_code, &expectation);
        document::build(plan, &request, &normalized).map(|document| (class(&normalized), document))
    });
    let (stdout_bytes, stderr_bytes) = (output.stdout.len(), output.stderr.len());
    let Ok((class, document)) = class_and_document else {
        return refused("pure interaction-net observation could not be bound to its invocation", output);
    };
    let has_violation = crate::operation::runtime::output_contains_prohibited_field(&document)
        || !crate::operation::schema::validate_value(
            &plan.output_contract,
            &document,
            "output",
            crate::operation::model::DiagnosticCode::OutputRejected,
        )
        .is_empty();
    if has_violation {
        return refused("pure interaction-net document violates its declared contract", output);
    }
    debug_assert!(document.is_object());
    let is_completed = class == crate::operation::runtime::ObservationClass::Completed;
    let identity = is_completed
        .then(|| {
            crate::operation::identity::hash_value(crate::operation::runtime::OUTPUT_IDENTITY_DOMAIN, &document).ok()
        })
        .flatten();
    if is_completed && identity.is_none() {
        return refused("pure interaction-net output identity failed", output);
    }
    debug_assert!(!is_completed || identity.is_some());
    crate::operation::runtime::Observation {
        class,
        output: Some(document),
        output_identity: identity,
        output_bytes: stdout_bytes,
        stderr_bytes,
        diagnostic: format!("pure interaction-net evaluation observed as {}", label(&class)),
        redacted: true,
    }
}

fn refused(message: &str, output: &crate::agent_node::ProcessOutput) -> crate::operation::runtime::Observation {
    crate::operation::runtime::Observation {
        class: crate::operation::runtime::ObservationClass::DeterministicFailure,
        output: None,
        output_identity: None,
        output_bytes: output.stdout.len(),
        stderr_bytes: output.stderr.len(),
        diagnostic: message.to_string(),
        redacted: true,
    }
}

fn class(normalized: &super::report::Normalized) -> crate::operation::runtime::ObservationClass {
    match normalized.outcome {
        crate::pure_inet_profile::PureEvaluationOutcome::Complete { .. } => {
            crate::operation::runtime::ObservationClass::Completed
        }
        crate::pure_inet_profile::PureEvaluationOutcome::Cancelled
        | crate::pure_inet_profile::PureEvaluationOutcome::Unknown => {
            crate::operation::runtime::ObservationClass::Unknown
        }
        crate::pure_inet_profile::PureEvaluationOutcome::Partial
        | crate::pure_inet_profile::PureEvaluationOutcome::Invalid
        | crate::pure_inet_profile::PureEvaluationOutcome::Exhausted
        | crate::pure_inet_profile::PureEvaluationOutcome::Failed => {
            crate::operation::runtime::ObservationClass::DeterministicFailure
        }
    }
}

fn label(class: &crate::operation::runtime::ObservationClass) -> &'static str {
    match class {
        crate::operation::runtime::ObservationClass::Completed => "complete",
        crate::operation::runtime::ObservationClass::DeterministicFailure => "non-success",
        crate::operation::runtime::ObservationClass::Unknown => "unknown",
    }
}
