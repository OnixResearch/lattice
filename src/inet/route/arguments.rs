//! Decoding of one invocation's exact arguments.
//!
//! Locators name files inside the admitted artifact root; they are not
//! identities or authority. Each artifact is bound by its BLAKE3 identity and
//! byte length, and every producer ceiling may only lower the profile ceiling.

const MAXIMUM_FILE_NAME_BYTES: usize = 128;

/// The admitted arguments of one invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub artifact_root: std::path::PathBuf,
    pub program: String,
    pub program_identity: String,
    pub program_bytes: u64,
    pub inputs: String,
    pub inputs_identity: String,
    pub inputs_bytes: u64,
    pub ceilings: crate::inet::route::Ceilings,
    pub attempt: String,
    pub predecessor: Option<String>,
    pub consumer_binding: Option<String>,
}

/// Decode and check a plan's arguments against the profile ceilings and
/// the plan's process bounds.
pub fn decode(plan: &crate::operation::InvocationPlan) -> Result<Request, crate::inet::Error> {
    let is_plan_exact = plan.type_id == crate::inet::route::TYPE_ID
        && crate::inet::route::uses_runtime(plan)
        && plan.method == crate::inet::route::METHOD;
    if !is_plan_exact || !crate::inet::route::target_is_supported(&plan.target) {
        return Err(crate::inet::Error::InvalidArguments);
    }
    let text = |key: &str| plan.arguments.get(key).and_then(serde_json::Value::as_str).map(str::to_string);
    let number = |key: &str| plan.arguments.get(key).and_then(serde_json::Value::as_u64);
    let (Some(program), Some(program_identity), Some(inputs), Some(inputs_identity), Some(attempt)) =
        (text("program"), text("program_identity"), text("inputs"), text("inputs_identity"), text("attempt"))
    else {
        return Err(crate::inet::Error::InvalidArguments);
    };
    let ceilings = crate::inet::route::Ceilings {
        fuel_interactions: number("fuel_interactions").ok_or(crate::inet::Error::InvalidArguments)?,
        max_input_bytes: number("max_input_bytes").ok_or(crate::inet::Error::InvalidArguments)?,
        max_retained_bytes: number("max_retained_bytes").ok_or(crate::inet::Error::InvalidArguments)?,
        max_output_bytes: number("max_output_bytes").ok_or(crate::inet::Error::InvalidArguments)?,
    };
    validate_ceilings(ceilings, plan.bounds)?;
    let request = Request {
        artifact_root: std::path::PathBuf::from(
            plan.target[crate::inet::route::ARTIFACT_ROOT_TARGET].as_str().unwrap_or_default(),
        ),
        program,
        program_identity,
        program_bytes: number("program_bytes").ok_or(crate::inet::Error::InvalidArguments)?,
        inputs,
        inputs_identity,
        inputs_bytes: number("inputs_bytes").ok_or(crate::inet::Error::InvalidArguments)?,
        ceilings,
        attempt,
        predecessor: text("predecessor"),
        consumer_binding: text("consumer_binding"),
    };
    validate_names(&request)?;
    debug_assert!(request.ceilings.is_within_producer_defaults());
    debug_assert!(!request.attempt.is_empty());
    Ok(request)
}

fn validate_ceilings(
    ceilings: crate::inet::route::Ceilings,
    bounds: crate::operation::Bounds,
) -> Result<(), crate::inet::Error> {
    let is_positive = ceilings.fuel_interactions > 0
        && ceilings.max_input_bytes > 0
        && ceilings.max_retained_bytes > 0
        && ceilings.max_output_bytes > 0;
    let is_within_profile = ceilings.fuel_interactions <= crate::inet::route::CEILINGS.fuel_interactions
        && ceilings.max_input_bytes <= crate::inet::route::CEILINGS.max_input_bytes
        && ceilings.max_retained_bytes <= crate::inet::route::CEILINGS.max_retained_bytes
        && ceilings.max_output_bytes <= crate::inet::route::CEILINGS.max_output_bytes
        && usize::try_from(ceilings.max_output_bytes).is_ok_and(|output_bytes| output_bytes <= bounds.output_bytes);
    if !is_positive || !is_within_profile || !ceilings.is_within_producer_defaults() {
        return Err(crate::inet::Error::InvalidCeilings);
    }
    Ok(())
}

fn validate_names(request: &Request) -> Result<(), crate::inet::Error> {
    let is_file_name = |name: &str| {
        name.len() <= MAXIMUM_FILE_NAME_BYTES
            && !matches!(name, "" | "." | "..")
            && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    };
    let is_identity = |value: &str| value.len() == 64 && value.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    let is_lineage_valid = crate::operation::identity::valid_identifier(&request.attempt)
        && request.predecessor.as_deref().is_none_or(crate::operation::identity::valid_identifier)
        && request.consumer_binding.as_deref().is_none_or(is_identity);
    let is_artifact_set_valid = is_file_name(&request.program)
        && is_file_name(&request.inputs)
        && request.program != request.inputs
        && is_identity(&request.program_identity)
        && is_identity(&request.inputs_identity);
    let total_bytes = request.program_bytes.checked_add(request.inputs_bytes);
    let is_size_valid = total_bytes.is_some_and(|total| total <= crate::inet::route::CEILINGS.max_input_bytes);
    debug_assert!(request.program.len() <= MAXIMUM_FILE_NAME_BYTES || !is_artifact_set_valid);
    debug_assert!(request.program_bytes <= crate::inet::route::CEILINGS.max_input_bytes || !is_size_valid);
    if !is_lineage_valid || !is_artifact_set_valid || !is_size_valid {
        return Err(crate::inet::Error::InvalidArguments);
    }
    Ok(())
}
