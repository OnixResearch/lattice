const FABRIC_PRELUDE: &str = "\
(define animus-summary \"\")\n\
(define animus-last-model-message #f)\n\
(define animus-last-tool-output #f)\n\
(define (animus-model prompt) (list 'model prompt))\n\
(define (animus-tool name schema arguments correlation-id visible-budget mutation-targets capabilities approval)\n\
  (list 'tool name schema arguments correlation-id visible-budget mutation-targets capabilities approval))\n\
(define (animus-respond message) (list 'respond message))\n\
(define (animus-completion-candidate request) (list 'completion-candidate request))\n\
(define (animus-stop) (list 'stop))\n";
const DECIDE_CALL: &str = "(animus-decide)";

/// Persistent Steel-backed loop fabric.
pub struct Runtime {
    engine: steel::steel_vm::engine::Engine,
}

impl Runtime {
    /// Builds a Steel loop fabric from a user program.
    ///
    /// The program must define `(animus-decide)`. Each call to [`Self::decide`]
    /// binds `animus-summary`, `animus-last-model-message`, and
    /// `animus-last-tool-output` before invoking that function.
    ///
    /// # Errors
    ///
    /// Returns [`crate::LoopError::Load`] when Steel rejects the prelude or user
    /// program.
    pub fn new(program: &str) -> Result<Self, crate::LoopError> {
        let mut engine = steel::steel_vm::engine::Engine::new();
        load_program(&mut engine, FABRIC_PRELUDE)?;
        load_program(&mut engine, program)?;

        Ok(Self { engine })
    }

    /// Evaluates one agent-loop turn and returns a validated loop decision.
    ///
    /// # Errors
    ///
    /// Returns [`crate::LoopError`] when Steel evaluation fails or when the
    /// returned decision violates the Rust boundary contract.
    pub fn decide(&mut self, input: &crate::LoopTurn) -> Result<crate::LoopDecision, crate::LoopError> {
        let bindings = input.bindings_program();
        evaluate_program(&mut self.engine, &bindings)?;
        let values = evaluate_program(&mut self.engine, DECIDE_CALL)?;
        let decision_value = values.into_iter().last().ok_or(crate::LoopError::MissingDecision)?;

        crate::LoopDecision::from_value(decision_value)
    }

    /// Builds a fresh fabric and evaluates one turn.
    ///
    /// # Errors
    ///
    /// Returns [`crate::LoopError`] when loading or evaluating the Steel program
    /// fails or the decision is invalid.
    pub fn decide_once(program: &str, input: &crate::LoopTurn) -> Result<crate::LoopDecision, crate::LoopError> {
        let mut runtime = Self::new(program)?;
        runtime.decide(input)
    }
}

fn load_program(engine: &mut steel::steel_vm::engine::Engine, program: &str) -> Result<(), crate::LoopError> {
    engine
        .run(String::from(program))
        .map(|_values| ())
        .map_err(|source| crate::LoopError::Load { source })
}

fn evaluate_program(
    engine: &mut steel::steel_vm::engine::Engine,
    program: &str,
) -> Result<Vec<steel::SteelVal>, crate::LoopError> {
    engine.run(String::from(program)).map_err(|source| crate::LoopError::Evaluate { source })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "intent: build reliable systems";
    const MODEL_MESSAGE: &str = "model answer";
    const TOOL_OUTPUT: &str = "tool result";
    const MODEL_PROGRAM: &str = r"
(define (animus-decide)
  (animus-model animus-summary))
";
    const TOOL_PROGRAM: &str = r#"
(define (animus-decide)
  (animus-tool
    "search"
    "schema.search.v1"
    (list (list "query" "adapter contracts"))
    "tool-call-1"
    10
    (list "resource:file:README")
    (list "tool.read")
    "pre_approved"))
"#;
    const RESPOND_PROGRAM: &str = r"
(define (animus-decide)
  (if animus-last-model-message
      (animus-respond animus-last-model-message)
      (animus-stop)))
";
    const INVALID_RETURN_PROGRAM: &str = r#"
(define (animus-decide)
  "not a decision")
"#;
    const INVALID_PAYLOAD_PROGRAM: &str = r#"
(define (animus-decide)
  (animus-model "   "))
"#;
    const INVALID_LOAD_PROGRAM: &str = "(define (";

    #[test]
    fn steel_program_selects_model_call() {
        let input = crate::LoopTurn::new(SUMMARY, None, None).expect("valid turn should parse");
        let decision = Runtime::decide_once(MODEL_PROGRAM, &input).expect("decision should parse");

        match decision {
            crate::LoopDecision::CallModel { request } => {
                assert_eq!(request.prompt(), SUMMARY);
            }
            other => panic!("expected model call, got {other:?}"),
        }
    }

    #[test]
    fn steel_program_selects_tool_call() {
        let input = crate::LoopTurn::new(SUMMARY, None, None).expect("valid turn should parse");
        let decision = Runtime::decide_once(TOOL_PROGRAM, &input).expect("decision should parse");

        assert!(matches!(decision, crate::LoopDecision::CallTool { .. }));
    }

    #[test]
    fn steel_program_can_use_previous_outputs() {
        let input =
            crate::LoopTurn::new(SUMMARY, Some(MODEL_MESSAGE), Some(TOOL_OUTPUT)).expect("valid turn should parse");
        let decision = Runtime::decide_once(RESPOND_PROGRAM, &input).expect("decision should parse");

        match decision {
            crate::LoopDecision::Respond { response } => {
                assert_eq!(response.message(), MODEL_MESSAGE);
            }
            other => panic!("expected response, got {other:?}"),
        }
    }

    #[test]
    fn steel_program_can_stop() {
        let input = crate::LoopTurn::new(SUMMARY, None, None).expect("valid turn should parse");
        let decision = Runtime::decide_once(RESPOND_PROGRAM, &input).expect("decision should parse");

        assert_eq!(decision, crate::LoopDecision::Stop);
    }

    #[test]
    fn invalid_return_shape_is_rejected() {
        let input = crate::LoopTurn::new(SUMMARY, None, None).expect("valid turn should parse");
        let error = Runtime::decide_once(INVALID_RETURN_PROGRAM, &input).expect_err("invalid decision should fail");

        assert!(matches!(error, crate::LoopError::InvalidDecision { .. }));
    }

    #[test]
    fn invalid_adapter_payload_is_rejected() {
        let input = crate::LoopTurn::new(SUMMARY, None, None).expect("valid turn should parse");
        let error = Runtime::decide_once(INVALID_PAYLOAD_PROGRAM, &input).expect_err("invalid payload should fail");

        assert!(matches!(error, crate::LoopError::AdapterContract { .. }));
    }

    #[test]
    fn invalid_program_load_is_rejected() {
        let error = match Runtime::new(INVALID_LOAD_PROGRAM) {
            Ok(_runtime) => panic!("invalid program should fail"),
            Err(error) => error,
        };

        assert!(matches!(error, crate::LoopError::Load { .. }));
    }
}
