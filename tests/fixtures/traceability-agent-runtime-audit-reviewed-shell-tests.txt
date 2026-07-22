use super::BoundedLoopFabric;

const SUMMARY: &str = "bounded test summary";
const MODEL_RESPONSE: &str = "bounded model response";
const SPINNING_PROGRAM: &str = r"
(define (animus-decide)
  (animus-model animus-summary))
";
const TOOL_PROGRAM: &str = r#"
(define tool-visible-character-budget 64)
(define (animus-decide)
  (animus-tool
    "search"
    "schema.search.v1"
    (list (list "query" "bounded test"))
    "tool-call-1"
    tool-visible-character-budget
    (list "resource:file:README")
    (list "tool.read")
    "pre_approved"))
"#;
const NO_MODEL_CALLS: u32 = 0;
const ONE_MODEL_CALL: u32 = 1;
const EXPECTED_SPINNING_MODEL_CALLS: u32 = crate::animus_agent::TURN_LIMIT;

#[derive(Debug)]
struct StubError;

impl core::fmt::Display for StubError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("stub adapter must not fail")
    }
}

struct ModelStub {
    calls: u32,
}

impl ModelStub {
    const fn new() -> Self {
        Self { calls: NO_MODEL_CALLS }
    }
}

impl animus_adapters::ComponentAdapter<animus_adapters::ModelRequest, animus_adapters::ModelResponse> for ModelStub {
    type Error = StubError;

    fn call(
        &mut self,
        _request: &animus_adapters::ModelRequest,
    ) -> Result<animus_adapters::ModelResponse, Self::Error> {
        self.calls = self.calls.checked_add(ONE_MODEL_CALL).ok_or(StubError)?;
        animus_adapters::ModelResponse::new(MODEL_RESPONSE).map_err(|_error| StubError)
    }
}

struct NeverTool;

impl animus_adapters::ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse> for NeverTool {
    type Error = StubError;

    fn call(&mut self, _request: &animus_adapters::ToolRequest) -> Result<animus_adapters::ToolResponse, Self::Error> {
        Err(StubError)
    }
}

// r[verify lattice.agent_runtime.animus_fabric]
#[test]
fn bounded_fabric_prevents_a_model_dispatch_after_turn_exhaustion() {
    let runtime_input = animus_runtime::RuntimeInput::new(SUMMARY).expect("runtime input is valid");
    let steel_fabric = animus_steel::SteelLoopFabric::new(SPINNING_PROGRAM).expect("Steel program is valid");
    let mut bounded_fabric =
        BoundedLoopFabric::new(steel_fabric, crate::animus_agent::TURN_LIMIT).expect("turn limit is valid");
    let mut model = ModelStub::new();
    let adapters = animus_runtime::RuntimeAdapters::<_, NeverTool>::new_optional(Some(&mut model), None);

    let run_report = animus_runtime::run_agent_loop(
        &runtime_input,
        animus_runtime::RuntimeConfig::new(animus_runtime::DEFAULT_MAX_TURNS),
        &mut bounded_fabric,
        adapters,
    );

    assert!(matches!(
        run_report.status(),
        animus_runtime::RuntimeStatus::Failed {
            failure: animus_runtime::RuntimeFailure::Fabric { message }
        } if message.contains("exhausted")
    ));
    assert_eq!(model.calls, EXPECTED_SPINNING_MODEL_CALLS);
}

// r[verify lattice.agent_runtime.animus_effects]
#[test]
fn absent_tool_service_fails_before_a_product_tool_effect() {
    let runtime_input = animus_runtime::RuntimeInput::new(SUMMARY).expect("runtime input is valid");
    let steel_fabric = animus_steel::SteelLoopFabric::new(TOOL_PROGRAM).expect("Steel program is valid");
    let mut bounded_fabric =
        BoundedLoopFabric::new(steel_fabric, crate::animus_agent::TURN_LIMIT).expect("turn limit is valid");
    let mut model = ModelStub::new();
    let adapters = animus_runtime::RuntimeAdapters::<_, NeverTool>::new_optional(Some(&mut model), None);

    let run_report = animus_runtime::run_agent_loop(
        &runtime_input,
        animus_runtime::RuntimeConfig::new(animus_runtime::DEFAULT_MAX_TURNS),
        &mut bounded_fabric,
        adapters,
    );

    assert!(matches!(run_report.status(), animus_runtime::RuntimeStatus::Failed {
        failure: animus_runtime::RuntimeFailure::ToolAdapter { .. }
    }));
    assert_eq!(model.calls, NO_MODEL_CALLS);
}
