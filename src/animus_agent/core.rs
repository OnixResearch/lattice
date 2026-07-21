pub const BUILTIN_PROGRAM: &str = r#"(define (animus-decide)
  (if animus-last-model-message
      (animus-respond animus-last-model-message)
      (animus-model animus-summary)))
"#;
pub const BUILTIN_PROGRAM_IDENTITY: &str = "b3:79d887ab1ffd350d191ad7aea6a10fe3de28f108d34271cc99e4a59e537e9d0e";
pub const TURN_LIMIT: u32 = 2;
pub const ANIMUS_RUNTIME_MAX_TURNS: u32 = 16;

const BLAKE3_IDENTITY_PREFIX: &str = "b3:";
const ANIMUS_ADAPTER_LABEL: &str = "animus";
const ANIMUS_FABRIC_LABEL: &str = "steel";
const OPENAI_PROVIDER_LABEL: &str = "openai_compatible";
const TOOLS_DISABLED_LABEL: &str = "disabled";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub summary: String,
    pub output_mode: crate::agent_node::AgentOutputMode,
    pub program: &'static str,
    pub program_identity: &'static str,
    pub turn_limit: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramIdentityCheck<'a> {
    pub program: &'a str,
    pub expected_identity: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionBudget {
    pub decision_count: u32,
    pub turn_limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeMetadataInput<'a> {
    pub event_count: u64,
    pub model_call_count: u64,
    pub model_label: &'a str,
}

// r[impl lattice.agent_runtime.animus_fabric]
#[must_use]
pub fn program_identity(program: &str) -> String {
    format!("{BLAKE3_IDENTITY_PREFIX}{}", blake3::hash(program.as_bytes()).to_hex())
}

// r[impl lattice.agent_runtime.animus_leaf]
// r[impl lattice.agent_runtime.animus_fabric]
// r[impl lattice.agent_runtime.animus_output]
pub fn plan_execution(
    request: &crate::agent_node::AgentRequest,
) -> Result<ExecutionPlan, crate::agent_node::AgentError> {
    validate_request(request)?;
    validate_program_identity(ProgramIdentityCheck {
        program: BUILTIN_PROGRAM,
        expected_identity: BUILTIN_PROGRAM_IDENTITY,
    })?;
    validate_turn_limit(TURN_LIMIT)?;

    let mut summary_request = request.clone();
    summary_request.model_hint = None;
    let summary = crate::agent_node::render_agent_command_prompt(&summary_request);
    if summary.trim().is_empty() {
        return Err(crate::agent_node::AgentError::MissingPrompt);
    }
    debug_assert_eq!(request.target, crate::agent_node::AgentTarget::Animus);
    debug_assert_eq!(program_identity(BUILTIN_PROGRAM), BUILTIN_PROGRAM_IDENTITY);
    debug_assert!(!summary.trim().is_empty());

    Ok(ExecutionPlan {
        summary,
        output_mode: request.output_mode,
        program: BUILTIN_PROGRAM,
        program_identity: BUILTIN_PROGRAM_IDENTITY,
        turn_limit: TURN_LIMIT,
    })
}

pub fn validate_program_identity(check: ProgramIdentityCheck<'_>) -> Result<(), crate::agent_node::AgentError> {
    if check.program.trim().is_empty() {
        return Err(preflight("Animus Steel program is empty"));
    }
    let observed_identity = program_identity(check.program);
    if observed_identity != check.expected_identity {
        return Err(preflight(format!(
            "Animus Steel program identity mismatch: expected {}, observed {observed_identity}",
            check.expected_identity
        )));
    }
    Ok(())
}

pub fn validate_turn_limit(turn_limit: u32) -> Result<(), crate::agent_node::AgentError> {
    if turn_limit == 0 || turn_limit > ANIMUS_RUNTIME_MAX_TURNS {
        return Err(preflight(format!("Animus turn limit must be between 1 and {ANIMUS_RUNTIME_MAX_TURNS}")));
    }
    Ok(())
}

pub fn validate_decision_count(budget: DecisionBudget) -> Result<(), crate::agent_node::AgentError> {
    validate_turn_limit(budget.turn_limit)?;
    if budget.decision_count >= budget.turn_limit {
        return Err(crate::agent_node::AgentError::Adapter(format!(
            "Animus Steel loop exhausted its {}-turn limit",
            budget.turn_limit
        )));
    }
    Ok(())
}

// r[impl lattice.agent_runtime.animus_output]
pub fn decode_final_response(
    output_mode: crate::agent_node::AgentOutputMode,
    message: &str,
) -> Result<crate::agent_node::AgentOutput, crate::agent_node::AgentError> {
    let Some(message) = non_empty_trimmed(message) else {
        return Err(crate::agent_node::AgentError::Adapter("Animus final response is empty".to_string()));
    };
    match output_mode {
        crate::agent_node::AgentOutputMode::Text => Ok(crate::agent_node::AgentOutput::Text(message.to_string())),
        crate::agent_node::AgentOutputMode::Json => {
            serde_json::from_str(message).map(crate::agent_node::AgentOutput::Json).map_err(|error| {
                crate::agent_node::AgentError::Adapter(format!(
                    "Animus final response is not valid requested JSON: {error}"
                ))
            })
        }
        crate::agent_node::AgentOutputMode::Image | crate::agent_node::AgentOutputMode::Artifact => {
            Err(preflight("Animus runtime supports only text and JSON outputs"))
        }
    }
}

// r[impl lattice.agent_runtime.animus_output]
#[must_use]
pub fn safe_metadata(input: SafeMetadataInput<'_>) -> serde_json::Value {
    serde_json::json!({
        "adapter": ANIMUS_ADAPTER_LABEL,
        "runtime": ANIMUS_ADAPTER_LABEL,
        "fabric": ANIMUS_FABRIC_LABEL,
        "program_identity": BUILTIN_PROGRAM_IDENTITY,
        "turn_limit": TURN_LIMIT,
        "event_count": input.event_count,
        "model_call_count": input.model_call_count,
        "provider": OPENAI_PROVIDER_LABEL,
        "model": input.model_label,
        "tools": TOOLS_DISABLED_LABEL,
    })
}

fn validate_request(request: &crate::agent_node::AgentRequest) -> Result<(), crate::agent_node::AgentError> {
    if request.target != crate::agent_node::AgentTarget::Animus {
        return Err(preflight("Animus execution requires the animus agent target"));
    }
    if request.prompt.trim().is_empty() {
        return Err(crate::agent_node::AgentError::MissingPrompt);
    }
    if !request.image_inputs.is_empty() {
        return Err(preflight("Animus runtime does not support image inputs"));
    }
    if request.session.as_deref().and_then(non_empty_trimmed).is_some() {
        return Err(preflight("Animus runtime does not support sessions"));
    }
    match request.output_mode {
        crate::agent_node::AgentOutputMode::Text | crate::agent_node::AgentOutputMode::Json => {}
        crate::agent_node::AgentOutputMode::Image | crate::agent_node::AgentOutputMode::Artifact => {
            return Err(preflight("Animus runtime supports only text and JSON outputs"));
        }
    }
    debug_assert_eq!(request.target, crate::agent_node::AgentTarget::Animus);
    debug_assert!(!request.prompt.trim().is_empty());
    debug_assert!(request.image_inputs.is_empty());
    debug_assert!(request.session.as_deref().and_then(non_empty_trimmed).is_none());
    Ok(())
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn preflight(message: impl Into<String>) -> crate::agent_node::AgentError {
    crate::agent_node::AgentError::PreflightRejected(message.into())
}
