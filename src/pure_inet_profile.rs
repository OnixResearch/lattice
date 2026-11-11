//! Pure interaction-net arithmetic profile.
//!
//! This module owns only the closed, side-effect-free vocabulary shared by the
//! future producer adapter: two signed 64-bit inputs, one signed 64-bit result,
//! and finite `Input`/`Const`/`Add`/`Sub`/`Mul` programs with wrapping
//! arithmetic. It performs no I/O and does not define a producer wire format.

use thiserror::Error;

/// The first profile admits exactly two signed 64-bit inputs.
pub const INPUT_COUNT: usize = 2;

/// The first profile produces exactly one signed 64-bit result.
pub const OUTPUT_COUNT: usize = 1;

/// Bounds the closed program shape before evaluation.
pub const MAX_PROGRAM_OPERATIONS: usize = 1_024;

/// A closed pure interaction-net operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PureOperation {
    /// Read one of the two admitted inputs.
    Input(u8),
    /// Push a signed 64-bit constant.
    Const(i64),
    /// Wrapping two's-complement addition.
    Add,
    /// Wrapping two's-complement subtraction.
    Sub,
    /// Wrapping two's-complement multiplication.
    Mul,
}

/// Exactly two signed 64-bit inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PureInputs {
    pub first: i64,
    pub second: i64,
}

impl PureInputs {
    /// Parse both inputs from canonical decimal strings.
    pub fn from_canonical(first: &str, second: &str) -> Result<Self, PureProfileError> {
        Ok(Self {
            first: parse_canonical_i64(first)?,
            second: parse_canonical_i64(second)?,
        })
    }
}

/// A finite, acyclic arithmetic program over the admitted inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureProgram {
    operations: Vec<PureOperation>,
}

impl PureProgram {
    /// Admit a program only when it has one result and never underflows.
    pub fn new(operations: Vec<PureOperation>) -> Result<Self, PureProfileError> {
        if operations.is_empty() {
            return Err(PureProfileError::EmptyProgram);
        }
        if operations.len() > MAX_PROGRAM_OPERATIONS {
            return Err(PureProfileError::ProgramTooLarge);
        }
        let mut depth = 0usize;
        for operation in &operations {
            match operation {
                PureOperation::Input(index) if *index >= INPUT_COUNT as u8 => {
                    return Err(PureProfileError::InvalidInputIndex);
                }
                PureOperation::Input(_) | PureOperation::Const(_) => {
                    depth = depth.checked_add(1).ok_or(PureProfileError::StackOverflow)?;
                }
                PureOperation::Add | PureOperation::Sub | PureOperation::Mul => {
                    depth = depth.checked_sub(2).ok_or(PureProfileError::StackUnderflow)?;
                    depth = depth.checked_add(1).ok_or(PureProfileError::StackOverflow)?;
                }
            }
        }
        if depth != OUTPUT_COUNT {
            return Err(PureProfileError::WrongResultArity);
        }
        Ok(Self { operations })
    }

    /// Evaluate with wrapping two's-complement arithmetic.
    pub fn evaluate(&self, inputs: PureInputs) -> Result<i64, PureProfileError> {
        let mut stack = Vec::with_capacity(self.operations.len().min(MAX_PROGRAM_OPERATIONS));
        for operation in &self.operations {
            match *operation {
                PureOperation::Input(0) => stack.push(inputs.first),
                PureOperation::Input(1) => stack.push(inputs.second),
                PureOperation::Input(_) => return Err(PureProfileError::InvalidInputIndex),
                PureOperation::Const(value) => stack.push(value),
                PureOperation::Add => {
                    let right = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    let left = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    stack.push(left.wrapping_add(right));
                }
                PureOperation::Sub => {
                    let right = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    let left = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    stack.push(left.wrapping_sub(right));
                }
                PureOperation::Mul => {
                    let right = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    let left = stack.pop().ok_or(PureProfileError::StackUnderflow)?;
                    stack.push(left.wrapping_mul(right));
                }
            }
        }
        if stack.len() != OUTPUT_COUNT {
            return Err(PureProfileError::WrongResultArity);
        }
        stack.pop().ok_or(PureProfileError::WrongResultArity)
    }

    /// Return the exact canonical decimal result for the supplied inputs.
    pub fn evaluate_canonical(&self, inputs: PureInputs) -> Result<String, PureProfileError> {
        self.evaluate(inputs).map(format_canonical_i64)
    }
}

/// Reject non-string JSON integers before they can be parsed through a float.
pub fn canonical_i64_from_json(value: &serde_json::Value) -> Result<i64, PureProfileError> {
    value.as_str().ok_or(PureProfileError::NonStringInteger).and_then(parse_canonical_i64)
}

/// Parse only `0` or a signed nonzero decimal with no leading `+` or zeroes.
pub fn parse_canonical_i64(text: &str) -> Result<i64, PureProfileError> {
    if text.is_empty() || text.len() > 20 {
        return Err(PureProfileError::NonCanonicalInteger);
    }
    if text == "0" {
        return Ok(0);
    }
    let bytes = text.as_bytes();
    let digits = match bytes.first() {
        Some(b'-') => &bytes[1..],
        Some(_) => bytes,
        None => return Err(PureProfileError::NonCanonicalInteger),
    };
    if digits.is_empty() || digits[0] == b'0' || !digits.iter().all(u8::is_ascii_digit) {
        return Err(PureProfileError::NonCanonicalInteger);
    }
    text.parse::<i64>().map_err(|_| PureProfileError::IntegerOutOfRange)
}

/// Format a signed 64-bit value in the same canonical form.
pub fn format_canonical_i64(value: i64) -> String {
    value.to_string()
}

/// Normalized evaluation observation, separate from process disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PureEvaluationOutcome {
    /// Complete evaluation satisfying the admitted output contract.
    Complete { result: i64 },
    /// Explicitly partial evaluation, even when the process exited zero.
    Partial,
    /// Invalid program, output shape, or identity binding.
    Invalid,
    /// Work, arena, memory, output, or deadline ceiling exhausted.
    Exhausted,
    /// Ordinary worker failure.
    Failed,
    /// Observed cancellation.
    Cancelled,
    /// Lost, interrupted, or otherwise unobserved outcome.
    Unknown,
}

impl PureEvaluationOutcome {
    /// Only a complete, identity-checked evaluation may supply a usable result.
    pub fn usable_result(self) -> Option<i64> {
        match self {
            Self::Complete { result } => Some(result),
            Self::Partial | Self::Invalid | Self::Exhausted | Self::Failed | Self::Cancelled | Self::Unknown => None,
        }
    }
}

/// Version of the host-side invocation envelope.
pub const INVOCATION_SCHEMA: &str = "lattice.pure-inet-invocation.v1";

/// Bounds free-form binding identifiers before admission.
pub const MAX_IDENTIFIER_BYTES: usize = 256;

/// Exact immutable bindings for one pure computation invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureComputationBinding {
    pub workflow_revision: String,
    pub instance_id: String,
    pub dispatch_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub program_identity: String,
    pub input_identity: String,
    pub producer_revision: String,
    pub rule_revision: String,
    pub runtime_identity: String,
    pub executable_identity: String,
    pub profile_identity: String,
    pub extrinsic_profile_identity: String,
    pub output_contract_identity: String,
}

impl PureComputationBinding {
    /// Admit only bounded identifiers and exact lowercase BLAKE3 identities.
    pub fn validate(&self) -> Result<(), PureProfileError> {
        for identifier in [
            &self.workflow_revision,
            &self.instance_id,
            &self.dispatch_id,
            &self.run_id,
            &self.attempt_id,
            &self.producer_revision,
            &self.rule_revision,
        ] {
            validate_identifier(identifier)?;
        }
        for identity in [
            &self.program_identity,
            &self.input_identity,
            &self.runtime_identity,
            &self.executable_identity,
            &self.profile_identity,
            &self.extrinsic_profile_identity,
            &self.output_contract_identity,
        ] {
            if !is_lowercase_blake3(identity) {
                return Err(PureProfileError::InvalidBindingIdentity);
            }
        }
        Ok(())
    }
}

/// Effective profile limits admitted before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PureLimits {
    pub max_program_operations: usize,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub deadline_millis: u64,
}

impl Default for PureLimits {
    fn default() -> Self {
        Self {
            max_program_operations: MAX_PROGRAM_OPERATIONS,
            max_input_bytes: 4_096,
            max_output_bytes: 4_096,
            deadline_millis: 60_000,
        }
    }
}

impl PureLimits {
    pub fn validate(&self) -> Result<(), PureProfileError> {
        if !(1..=MAX_PROGRAM_OPERATIONS).contains(&self.max_program_operations)
            || self.max_input_bytes == 0
            || self.max_input_bytes > 16 * 1024 * 1024
            || self.max_output_bytes == 0
            || self.max_output_bytes > 16 * 1024 * 1024
            || !(1..=300_000).contains(&self.deadline_millis)
        {
            return Err(PureProfileError::InvalidLimits);
        }
        Ok(())
    }
}

/// Host-side envelope that binds one pure program and inputs to an attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureInvocationEnvelope {
    pub schema: String,
    pub binding: PureComputationBinding,
    pub limits: PureLimits,
    pub program: PureProgram,
    pub inputs: PureInputs,
}

impl PureInvocationEnvelope {
    /// Validate the schema, bindings, limits, and admitted program size.
    pub fn validate(&self) -> Result<(), PureProfileError> {
        if self.schema != INVOCATION_SCHEMA {
            return Err(PureProfileError::InvalidSchema);
        }
        self.binding.validate()?;
        self.limits.validate()?;
        if self.program.operations.len() > self.limits.max_program_operations {
            return Err(PureProfileError::ProgramExceedsAdmittedLimit);
        }
        Ok(())
    }

    /// Evaluate only after the envelope is admitted.
    pub fn evaluate_canonical(&self) -> Result<String, PureProfileError> {
        self.validate()?;
        self.program.evaluate_canonical(self.inputs)
    }
}

fn validate_identifier(value: &str) -> Result<(), PureProfileError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(PureProfileError::InvalidBindingIdentifier);
    }
    Ok(())
}

fn is_lowercase_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Hash exact bytes with BLAKE3 and return the lowercase hex identity.
pub fn blake3_identity(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Reject changed bytes or a malformed expected identity before use.
pub fn verify_identity(expected: &str, bytes: &[u8]) -> Result<(), PureProfileError> {
    if !is_lowercase_blake3(expected) {
        return Err(PureProfileError::InvalidBindingIdentity);
    }
    if blake3_identity(bytes) != expected {
        return Err(PureProfileError::IdentityMismatch);
    }
    Ok(())
}

/// Remeasure the admitted program bytes before dispatch.
pub fn verify_program_bytes(binding: &PureComputationBinding, bytes: &[u8]) -> Result<(), PureProfileError> {
    binding.validate()?;
    verify_identity(&binding.program_identity, bytes)
}

/// Remeasure the admitted input bytes before dispatch.
pub fn verify_input_bytes(binding: &PureComputationBinding, bytes: &[u8]) -> Result<(), PureProfileError> {
    binding.validate()?;
    verify_identity(&binding.input_identity, bytes)
}

/// Validate a returned output identity before output admission.
pub fn verify_output_identity(expected: &str, bytes: &[u8]) -> Result<(), PureProfileError> {
    verify_identity(expected, bytes)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PureProfileError {
    #[error("pure interaction-net program must contain at least one operation")]
    EmptyProgram,
    #[error("pure interaction-net program exceeds the operation limit")]
    ProgramTooLarge,
    #[error("pure interaction-net input index must be 0 or 1")]
    InvalidInputIndex,
    #[error("pure interaction-net program underflows its stack")]
    StackUnderflow,
    #[error("pure interaction-net program overflows its stack")]
    StackOverflow,
    #[error("pure interaction-net program must leave exactly one result")]
    WrongResultArity,
    #[error("integer must be a canonical decimal string")]
    NonCanonicalInteger,
    #[error("integer is outside the signed 64-bit range")]
    IntegerOutOfRange,
    #[error("integer must be carried as a string, not a JSON number")]
    NonStringInteger,
    #[error("pure interaction-net invocation schema is not supported")]
    InvalidSchema,
    #[error("pure interaction-net binding identifier is empty, unbounded, or contains control characters")]
    InvalidBindingIdentifier,
    #[error("pure interaction-net binding identity must be one lowercase BLAKE3 hex digest")]
    InvalidBindingIdentity,
    #[error("pure interaction-net bytes do not match the admitted identity")]
    IdentityMismatch,
    #[error("pure interaction-net limits are zero, oversized, or exceed the profile ceiling")]
    InvalidLimits,
    #[error("pure interaction-net program exceeds the admitted operation limit")]
    ProgramExceedsAdmittedLimit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(first: i64, second: i64) -> PureInputs {
        PureInputs { first, second }
    }

    #[test]
    fn canonical_integer_boundaries_round_trip_exactly() {
        for value in [
            0,
            1,
            -1,
            i64::MIN,
            i64::MAX,
            9_007_199_254_740_993,
            -9_007_199_254_740_993,
        ] {
            let text = format_canonical_i64(value);
            assert_eq!(parse_canonical_i64(&text), Ok(value));
        }
    }

    #[test]
    fn noncanonical_and_out_of_range_integers_are_rejected() {
        for text in [
            "",
            "+1",
            "-0",
            "00",
            "01",
            "-01",
            " 1",
            "1 ",
            "1.0",
            "0x1",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            assert!(parse_canonical_i64(text).is_err(), "{text}");
        }
    }

    #[test]
    fn json_numbers_are_not_integer_transport() {
        assert_eq!(canonical_i64_from_json(&serde_json::json!("-9223372036854775808")), Ok(i64::MIN));
        assert_eq!(canonical_i64_from_json(&serde_json::json!(1)), Err(PureProfileError::NonStringInteger));
        assert_eq!(canonical_i64_from_json(&serde_json::json!(1.5)), Err(PureProfileError::NonStringInteger));
    }

    #[test]
    fn closed_program_uses_wrapping_two_complement_arithmetic() {
        let program = PureProgram::new(vec![
            PureOperation::Input(0),
            PureOperation::Input(1),
            PureOperation::Mul,
            PureOperation::Const(i64::MAX),
            PureOperation::Add,
        ])
        .expect("program admits");
        assert_eq!(program.evaluate(inputs(i64::MAX, 2)), Ok(i64::MAX.wrapping_mul(2).wrapping_add(i64::MAX)));
        assert_eq!(program.evaluate(inputs(i64::MIN, -1)), Ok(i64::MIN.wrapping_mul(-1).wrapping_add(i64::MAX)));
    }

    #[test]
    fn shared_expression_and_subtraction_are_exact() {
        let program = PureProgram::new(vec![
            PureOperation::Input(0),
            PureOperation::Input(1),
            PureOperation::Add,
            PureOperation::Input(0),
            PureOperation::Input(1),
            PureOperation::Sub,
            PureOperation::Mul,
        ])
        .expect("program admits");
        let value = program.evaluate(inputs(9_007_199_254_740_993, 1));
        assert_eq!(
            value,
            Ok((9_007_199_254_740_993i64.wrapping_add(1)).wrapping_mul(9_007_199_254_740_993i64.wrapping_sub(1)))
        );
    }

    #[test]
    fn malformed_program_shapes_are_rejected() {
        assert_eq!(PureProgram::new(vec![]), Err(PureProfileError::EmptyProgram));
        assert_eq!(PureProgram::new(vec![PureOperation::Input(2)]), Err(PureProfileError::InvalidInputIndex));
        assert_eq!(PureProgram::new(vec![PureOperation::Add]), Err(PureProfileError::StackUnderflow));
        assert_eq!(
            PureProgram::new(vec![PureOperation::Input(0), PureOperation::Input(1)]),
            Err(PureProfileError::WrongResultArity)
        );
        assert_eq!(
            PureProgram::new(vec![PureOperation::Const(1), PureOperation::Add]),
            Err(PureProfileError::StackUnderflow)
        );
    }

    fn binding() -> PureComputationBinding {
        let identity = "a".repeat(64);
        PureComputationBinding {
            workflow_revision: identity.clone(),
            instance_id: "instance-1".to_string(),
            dispatch_id: "dispatch-1".to_string(),
            run_id: "run-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            program_identity: identity.clone(),
            input_identity: identity.clone(),
            producer_revision: "noble-revision".to_string(),
            rule_revision: "rules-v1".to_string(),
            runtime_identity: identity.clone(),
            executable_identity: identity.clone(),
            profile_identity: identity.clone(),
            extrinsic_profile_identity: identity.clone(),
            output_contract_identity: identity,
        }
    }

    fn envelope() -> PureInvocationEnvelope {
        PureInvocationEnvelope {
            schema: INVOCATION_SCHEMA.to_string(),
            binding: binding(),
            limits: PureLimits::default(),
            program: PureProgram::new(vec![PureOperation::Input(0), PureOperation::Input(1), PureOperation::Add])
                .expect("program admits"),
            inputs: inputs(20, 22),
        }
    }

    #[test]
    fn invocation_envelope_admits_complete_bindings() {
        let envelope = envelope();
        assert_eq!(envelope.validate(), Ok(()));
        assert_eq!(envelope.evaluate_canonical(), Ok("42".to_string()));
    }

    #[test]
    fn invocation_envelope_rejects_crossed_or_malformed_bindings() {
        let mut wrong_schema = envelope();
        wrong_schema.schema = "other".to_string();
        assert_eq!(wrong_schema.validate(), Err(PureProfileError::InvalidSchema));

        let mut upper_identity = envelope();
        upper_identity.binding.program_identity = "A".repeat(64);
        assert_eq!(upper_identity.validate(), Err(PureProfileError::InvalidBindingIdentity));

        let mut short_identity = envelope();
        short_identity.binding.input_identity = "a".repeat(63);
        assert_eq!(short_identity.validate(), Err(PureProfileError::InvalidBindingIdentity));

        let mut empty_id = envelope();
        empty_id.binding.attempt_id.clear();
        assert_eq!(empty_id.validate(), Err(PureProfileError::InvalidBindingIdentifier));

        let mut control_id = envelope();
        control_id.binding.run_id = "run\n1".to_string();
        assert_eq!(control_id.validate(), Err(PureProfileError::InvalidBindingIdentifier));
    }

    #[test]
    fn invocation_envelope_rejects_limits_and_program_over_admission() {
        let mut bad_limits = envelope();
        bad_limits.limits.deadline_millis = 0;
        assert_eq!(bad_limits.validate(), Err(PureProfileError::InvalidLimits));

        let mut narrow = envelope();
        narrow.limits.max_program_operations = 2;
        assert_eq!(narrow.validate(), Err(PureProfileError::ProgramExceedsAdmittedLimit));
    }

    #[test]
    fn only_complete_evaluation_supplies_a_usable_result() {
        assert_eq!(PureEvaluationOutcome::Complete { result: 42 }.usable_result(), Some(42));
        for outcome in [
            PureEvaluationOutcome::Partial,
            PureEvaluationOutcome::Invalid,
            PureEvaluationOutcome::Exhausted,
            PureEvaluationOutcome::Failed,
            PureEvaluationOutcome::Cancelled,
            PureEvaluationOutcome::Unknown,
        ] {
            assert_eq!(outcome.usable_result(), None, "{outcome:?}");
        }
    }

    #[test]
    fn byte_remeasurement_rejects_changed_or_crossed_bytes() {
        let program_bytes = b"program";
        let input_bytes = b"inputs";
        let output_bytes = b"output";
        let mut binding = binding();
        binding.program_identity = blake3_identity(program_bytes);
        binding.input_identity = blake3_identity(input_bytes);

        assert_eq!(verify_program_bytes(&binding, program_bytes), Ok(()));
        assert_eq!(verify_input_bytes(&binding, input_bytes), Ok(()));
        assert_eq!(verify_output_identity(&blake3_identity(output_bytes), output_bytes), Ok(()));

        assert_eq!(verify_program_bytes(&binding, b"changed program"), Err(PureProfileError::IdentityMismatch));
        assert_eq!(verify_input_bytes(&binding, b"changed inputs"), Err(PureProfileError::IdentityMismatch));
        assert_eq!(
            verify_output_identity(&blake3_identity(output_bytes), b"changed output"),
            Err(PureProfileError::IdentityMismatch)
        );
        assert_eq!(
            verify_output_identity("A".repeat(64).as_str(), output_bytes),
            Err(PureProfileError::InvalidBindingIdentity)
        );
    }
}
