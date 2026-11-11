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
}
