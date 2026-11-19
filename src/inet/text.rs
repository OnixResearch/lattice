//! Exact producer text forms admitted by the closed profile.
//!
//! The program artifact is Noble's own `ngraph-ir/0 ngraph-i64/0` graph file
//! and the input artifact is its `noble-ngraph-vectors/0` vector file; Lattice
//! defines no competing net format. Admission accepts only the canonical
//! printed subset of those formats: two I64 input slots, `input`, `const i64`,
//! `i64.add`, `i64.sub` and `i64.mul` nodes whose operands are strictly earlier
//! nodes, and one `out` line. Every integer is a canonical decimal. Comments,
//! facts, guards, parameters, shapes, `i64.eq`, F32 operations, repeated
//! outputs and noncanonical spacing are refused before any evaluation.

/// The producer graph schema and the only admitted sub-profile.
pub const GRAPH_HEADER: &str = "ngraph-ir/0 ngraph-i64/0";
/// The producer vector schema.
pub const VECTORS_SCHEMA: &str = "noble-ngraph-vectors/0";
/// The exact vector type line for the two admitted inputs.
pub const VECTORS_TYPES: &str = "types i64 i64";
/// The JSON document schema for consumers that carry inputs as JSON.
pub const INPUTS_JSON_SCHEMA: &str = "lattice.pure-inet-inputs.v1";

const SLOT_LINE: &str = "slot input i64";
const SUBJECT_DOMAIN: &[u8] = b"lattice.pure-inet-program-subject.v1\n";
const SUBJECT_KINDS: [&str; 2] = ["authored", "extracted"];
const INPUT_SLOT_LINES: usize = 2;
const PREAMBLE_LINES: usize = 4;

/// Parse the exact program artifact bytes into the closed acyclic vocabulary.
pub fn parse_program(bytes: &[u8]) -> Result<super::Program, super::Error> {
    let lines = canonical_lines(bytes).ok_or(super::Error::MalformedProgramText)?;
    let [header, subject, first_slot, second_slot, body @ ..] = lines.as_slice() else {
        return Err(super::Error::MalformedProgramText);
    };
    if *header != GRAPH_HEADER || !is_subject(subject) {
        return Err(super::Error::MalformedProgramText);
    }
    if *first_slot != SLOT_LINE || *second_slot != SLOT_LINE {
        return Err(super::Error::UnsupportedProgramLine);
    }
    let [nodes @ .., out] = body else {
        return Err(super::Error::MalformedProgramText);
    };
    if nodes.len() > crate::pure_inet_profile::MAX_PROGRAM_OPERATIONS {
        return Err(super::Error::Core(crate::pure_inet_profile::PureProfileError::ProgramTooLarge));
    }
    let output = out.strip_prefix("out ").and_then(node_reference).ok_or(super::Error::UnsupportedProgramLine)?;
    let operations = (0_u32..)
        .zip(nodes.iter())
        .map(|(index, line)| parse_node(index, line))
        .collect::<Result<Vec<_>, _>>()?;
    debug_assert!(operations.len() <= crate::pure_inet_profile::MAX_PROGRAM_OPERATIONS);
    debug_assert_eq!(lines.len(), operations.len().saturating_add(PREAMBLE_LINES).saturating_add(1));
    super::Program::new(operations, output)
}

/// Print an admitted program in the exact form [`parse_program`] accepts.
/// The `authored` subject identity is derived from the printed nodes.
pub fn render_program(program: &super::Program) -> String {
    let mut body = String::with_capacity(program.nodes().len().saturating_mul(24).saturating_add(64));
    for _ in 0..INPUT_SLOT_LINES {
        body.push_str(SLOT_LINE);
        body.push('\n');
    }
    for (index, operation) in program.nodes().iter().enumerate() {
        body.push_str(&format!("%{index} = {}\n", printed(*operation)));
    }
    body.push_str(&format!("out %{}\n", program.output()));
    let subject = blake3::Hasher::new().update(SUBJECT_DOMAIN).update(body.as_bytes()).finalize().to_hex();
    let rendered = format!("{GRAPH_HEADER}\nsubject authored {subject} closure -\n{body}");
    debug_assert!(rendered.ends_with('\n'));
    rendered
}

/// Parse the exact one-row input artifact bytes.
pub fn parse_inputs(bytes: &[u8]) -> Result<crate::pure_inet_profile::PureInputs, super::Error> {
    let lines = canonical_lines(bytes).ok_or(super::Error::MalformedInputsText)?;
    let [schema, types, row] = lines.as_slice() else {
        return Err(super::Error::MalformedInputsText);
    };
    if *schema != VECTORS_SCHEMA || *types != VECTORS_TYPES {
        return Err(super::Error::MalformedInputsText);
    }
    let mut values = row.strip_prefix("v ").ok_or(super::Error::MalformedInputsText)?.split(' ');
    let (Some(first), Some(second), None) = (values.next(), values.next(), values.next()) else {
        return Err(super::Error::MalformedInputsText);
    };
    Ok(crate::pure_inet_profile::PureInputs::from_canonical(first, second)?)
}

/// Print inputs in the exact form [`parse_inputs`] accepts.
pub fn render_inputs(inputs: crate::pure_inet_profile::PureInputs) -> String {
    format!("{VECTORS_SCHEMA}\n{VECTORS_TYPES}\nv {} {}\n", inputs.first, inputs.second)
}

/// Admit `{"schema": "lattice.pure-inet-inputs.v1", "inputs": ["<i64>", "<i64>"]}`.
/// JSON numbers are refused even when they would fit exactly.
pub fn inputs_from_json(value: &serde_json::Value) -> Result<crate::pure_inet_profile::PureInputs, super::Error> {
    let object = value.as_object().ok_or(super::Error::MalformedInputsText)?;
    let has_exact_fields =
        object.len() == 2 && object.get("schema").and_then(serde_json::Value::as_str) == Some(INPUTS_JSON_SCHEMA);
    let values = object.get("inputs").and_then(serde_json::Value::as_array);
    let Some([first, second]) = values.map(Vec::as_slice).filter(|_| has_exact_fields) else {
        return Err(super::Error::MalformedInputsText);
    };
    Ok(crate::pure_inet_profile::PureInputs {
        first: crate::pure_inet_profile::canonical_i64_from_json(first)?,
        second: crate::pure_inet_profile::canonical_i64_from_json(second)?,
    })
}

/// The JSON document [`inputs_from_json`] admits, with canonical decimal strings.
pub fn inputs_to_json(inputs: crate::pure_inet_profile::PureInputs) -> serde_json::Value {
    serde_json::json!({
        "schema": INPUTS_JSON_SCHEMA,
        "inputs": [crate::pure_inet_profile::format_canonical_i64(inputs.first), crate::pure_inet_profile::format_canonical_i64(inputs.second)],
    })
}

/// Split exact UTF-8 text into lines that each end in one newline and carry
/// single-space-separated tokens with no other whitespace.
fn canonical_lines(bytes: &[u8]) -> Option<Vec<&str>> {
    let text = std::str::from_utf8(bytes).ok()?;
    let body = text.strip_suffix('\n')?;
    let lines = body.split('\n').collect::<Vec<_>>();
    let is_canonical = lines.iter().all(|line| {
        !line.is_empty()
            && line.split(' ').all(|token| !token.is_empty())
            && line.bytes().all(|byte| byte == b' ' || byte.is_ascii_graphic())
    });
    is_canonical.then_some(lines)
}

fn is_subject(line: &str) -> bool {
    let tokens = line.split(' ').collect::<Vec<_>>();
    let [label, kind, identity, closure, closure_ids] = tokens.as_slice() else {
        return false;
    };
    let is_identity_exact =
        identity.len() == 64 && identity.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    *label == "subject"
        && SUBJECT_KINDS.contains(kind)
        && is_identity_exact
        && *closure == "closure"
        && *closure_ids == "-"
}

fn parse_node(index: u32, line: &str) -> Result<super::Node, super::Error> {
    let prefix = format!("%{index} = ");
    let operation = line.strip_prefix(prefix.as_str()).ok_or(super::Error::MalformedProgramText)?;
    let tokens = operation.split(' ').collect::<Vec<_>>();
    debug_assert!(!tokens.is_empty());
    let parsed = match tokens.as_slice() {
        ["input", "0"] => super::Node::Input(0),
        ["input", "1"] => super::Node::Input(1),
        ["const", "i64", value] => super::Node::Const(crate::pure_inet_profile::parse_canonical_i64(value)?),
        [name, left, right] => {
            let (left, right) = (operand(left)?, operand(right)?);
            match *name {
                "i64.add" => super::Node::Add(left, right),
                "i64.sub" => super::Node::Sub(left, right),
                "i64.mul" => super::Node::Mul(left, right),
                _ => return Err(super::Error::UnsupportedProgramLine),
            }
        }
        _ => return Err(super::Error::UnsupportedProgramLine),
    };
    debug_assert!(
        !matches!(parsed, super::Node::Input(slot) if usize::from(slot) >= crate::pure_inet_profile::INPUT_COUNT)
    );
    Ok(parsed)
}

fn operand(token: &str) -> Result<u32, super::Error> {
    node_reference(token).ok_or(super::Error::UnsupportedProgramLine)
}

fn node_reference(token: &str) -> Option<u32> {
    let digits = token.strip_prefix('%')?;
    let is_canonical = !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'));
    is_canonical.then(|| digits.parse().ok()).flatten()
}

fn printed(operation: super::Node) -> String {
    match operation {
        super::Node::Input(slot) => format!("input {slot}"),
        super::Node::Const(value) => format!("const i64 {value}"),
        super::Node::Add(left, right) => format!("i64.add %{left} %{right}"),
        super::Node::Sub(left, right) => format!("i64.sub %{left} %{right}"),
        super::Node::Mul(left, right) => format!("i64.mul %{left} %{right}"),
    }
}
