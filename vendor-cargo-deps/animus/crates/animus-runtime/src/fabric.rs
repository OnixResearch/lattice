/// Swappable loop-fabric boundary used by the runtime shell.
pub trait LoopFabric {
    /// Error returned by the concrete loop fabric.
    type Error: core::fmt::Display;

    /// Produces one validated loop decision for the supplied turn input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the fabric cannot produce a valid decision.
    fn decide(&mut self, turn: &crate::RuntimeLoopTurn) -> Result<crate::RuntimeLoopDecision, Self::Error>;
}

#[cfg(feature = "steel")]
impl LoopFabric for animus_steel::SteelLoopFabric {
    type Error = animus_steel::LoopError;

    fn decide(&mut self, turn: &crate::RuntimeLoopTurn) -> Result<crate::RuntimeLoopDecision, Self::Error> {
        let steel_turn = animus_steel::LoopTurn::try_from(turn)?;
        animus_steel::SteelLoopFabric::decide(self, &steel_turn).map(crate::RuntimeLoopDecision::from)
    }
}

#[cfg(test)]
mod tests {
    use super::LoopFabric;

    const SUMMARY: &str = "intent: build reliable systems";
    const MODEL_MESSAGE: &str = "model answer";
    const TOOL_NAME: &str = "search";
    const TOOL_INPUT: &str = "query";
    const TOOL_SCHEMA: &str = "schema.search.v1";
    const TOOL_ARGUMENT_VALUE: &str = "adapter contracts";
    const TOOL_CALL_ID: &str = "tool-call-1";
    const TOOL_VISIBLE_BUDGET: u32 = 64;
    const TOOL_MUTATION_TARGET: &str = "resource:file:README";
    const TOOL_CAPABILITY: &str = "tool.read";
    const TOOL_RESULT_SHAPE: &str = "text-summary";
    const TOOL_PROMPT_SNIPPET: &str = "Use search for indexed project facts.";
    const TOOL_PROMPT_GUIDELINE: &str = "Do not infer missing search results.";
    const TOOL_SOURCE_IDENTITY: &str = "host.registry";
    const TOOL_PROVENANCE: &str = "builtin";
    const TOOL_OUTPUT: &str = "tool result";
    const FABRIC_FAILURE: &str = "malformed decision";
    const NO_ADAPTER_CALLS: usize = 0;
    const ONE_ADAPTER_CALL: usize = 1;
    const TOOL_ARGUMENT_COUNT: usize = ONE_ADAPTER_CALL;
    const FOLLOWUP_TURN_BUDGET: usize = 2;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct DecisionFailure;

    impl core::fmt::Display for DecisionFailure {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str(FABRIC_FAILURE)
        }
    }

    struct FakeLoop {
        decisions: Vec<Result<crate::RuntimeLoopDecision, DecisionFailure>>,
        index: usize,
    }

    impl FakeLoop {
        fn new(decisions: Vec<Result<crate::RuntimeLoopDecision, DecisionFailure>>) -> Self {
            Self { decisions, index: 0 }
        }
    }

    impl LoopFabric for FakeLoop {
        type Error = DecisionFailure;

        fn decide(&mut self, _turn: &crate::RuntimeLoopTurn) -> Result<crate::RuntimeLoopDecision, Self::Error> {
            let decision = self.decisions.get(self.index).cloned().unwrap_or(Err(DecisionFailure));
            self.index = self.index.saturating_add(ONE_ADAPTER_CALL);
            decision
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct AdapterFailure;

    impl core::fmt::Display for AdapterFailure {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str("adapter failed")
        }
    }

    struct ModelStub {
        calls: usize,
    }

    impl ModelStub {
        const fn new() -> Self {
            Self {
                calls: NO_ADAPTER_CALLS,
            }
        }
    }

    impl animus_adapters::ComponentAdapter<animus_adapters::ModelRequest, animus_adapters::ModelResponse> for ModelStub {
        type Error = AdapterFailure;

        fn call(
            &mut self,
            request: &animus_adapters::ModelRequest,
        ) -> Result<animus_adapters::ModelResponse, Self::Error> {
            assert_eq!(request.prompt(), SUMMARY);
            self.calls = self.calls.saturating_add(ONE_ADAPTER_CALL);
            animus_adapters::ModelResponse::new(MODEL_MESSAGE).map_err(|_error| AdapterFailure)
        }
    }

    struct ToolStub {
        calls: usize,
    }

    impl ToolStub {
        const fn new() -> Self {
            Self {
                calls: NO_ADAPTER_CALLS,
            }
        }
    }

    impl animus_adapters::ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse> for ToolStub {
        type Error = AdapterFailure;

        fn call(
            &mut self,
            request: &animus_adapters::ToolRequest,
        ) -> Result<animus_adapters::ToolResponse, Self::Error> {
            assert_eq!(request.name(), TOOL_NAME);
            assert_eq!(request.schema_identity(), TOOL_SCHEMA);
            assert_eq!(request.arguments()[NO_ADAPTER_CALLS].name(), TOOL_INPUT);
            assert_eq!(request.arguments()[NO_ADAPTER_CALLS].value(), TOOL_ARGUMENT_VALUE);
            self.calls = self.calls.saturating_add(ONE_ADAPTER_CALL);
            Ok(tool_response())
        }
    }

    #[test]
    fn non_steel_loop_returns_immediate_final_response() {
        let mut model = ModelStub::new();
        let mut tool = ToolStub::new();
        let mut fabric = FakeLoop::new(vec![Ok(response_decision(MODEL_MESSAGE))]);
        let result = run_fake(&mut fabric, &mut model, &mut tool);

        assert!(matches!(
            result.status(),
            crate::RuntimeStatus::FinalResponse { response } if response.message() == MODEL_MESSAGE
        ));
        assert_eq!(model.calls, NO_ADAPTER_CALLS);
        assert_eq!(tool.calls, NO_ADAPTER_CALLS);
    }

    #[test]
    fn non_steel_loop_dispatches_model_and_tool_paths() {
        let mut model = ModelStub::new();
        let mut tool = ToolStub::new();
        let mut model_fabric = FakeLoop::new(vec![Ok(model_decision()), Ok(response_decision(MODEL_MESSAGE))]);
        let model_result = run_fake(&mut model_fabric, &mut model, &mut tool);
        let mut tool_fabric = FakeLoop::new(vec![Ok(tool_decision()), Ok(response_decision(TOOL_OUTPUT))]);
        let tool_result = run_fake(&mut tool_fabric, &mut model, &mut tool);

        assert!(matches!(
            model_result.status(),
            crate::RuntimeStatus::FinalResponse { response } if response.message() == MODEL_MESSAGE
        ));
        assert!(matches!(
            tool_result.status(),
            crate::RuntimeStatus::FinalResponse { response } if response.message() == TOOL_OUTPUT
        ));
        assert_eq!(model.calls, ONE_ADAPTER_CALL);
        assert_eq!(tool.calls, ONE_ADAPTER_CALL);
    }

    #[test]
    fn non_steel_loop_fails_closed_for_malformed_decision_and_missing_service() {
        let mut model = ModelStub::new();
        let mut tool = ToolStub::new();
        let mut bad_fabric = FakeLoop::new(vec![Err(DecisionFailure)]);
        let bad_result = run_fake(&mut bad_fabric, &mut model, &mut tool);
        let mut missing_model_fabric = FakeLoop::new(vec![Ok(model_decision())]);
        let missing_model_result = crate::run_agent_loop(
            &runtime_input(),
            runtime_config(),
            &mut missing_model_fabric,
            runtime_adapters(None, Some(&mut tool)),
        );

        assert!(matches!(
            bad_result.status(),
            crate::RuntimeStatus::Failed {
                failure: crate::RuntimeFailure::Fabric { message }
            } if message == FABRIC_FAILURE
        ));
        assert!(matches!(
            missing_model_result.status(),
            crate::RuntimeStatus::Failed {
                failure: crate::RuntimeFailure::ModelAdapter { message }
            } if message == animus_adapters::OutcomeKind::Absent.as_str()
        ));
        assert_eq!(model.calls, NO_ADAPTER_CALLS);
        assert_eq!(tool.calls, NO_ADAPTER_CALLS);
    }

    fn run_fake(fabric: &mut FakeLoop, model: &mut ModelStub, tool: &mut ToolStub) -> crate::RunReport {
        crate::run_agent_loop(&runtime_input(), runtime_config(), fabric, runtime_adapters(Some(model), Some(tool)))
    }

    fn runtime_input() -> crate::RuntimeInput {
        crate::RuntimeInput::new(SUMMARY).expect("summary should validate")
    }

    fn runtime_config() -> crate::RuntimeConfig {
        crate::RuntimeConfig::from_max_turn_count(FOLLOWUP_TURN_BUDGET).expect("turn budget should validate")
    }

    fn model_decision() -> crate::RuntimeLoopDecision {
        crate::RuntimeLoopDecision::from_draft(crate::RuntimeLoopDecisionDraft::model(SUMMARY))
            .expect("model decision should validate")
    }

    fn tool_decision() -> crate::RuntimeLoopDecision {
        crate::RuntimeLoopDecision::CallTool {
            request: tool_request(),
        }
    }

    fn tool_request() -> animus_adapters::ToolRequest {
        let fields = [animus_adapters::tool::ArgumentFieldDraft {
            name: Some(TOOL_INPUT),
            value: Some(TOOL_ARGUMENT_VALUE),
        }];
        let arguments =
            animus_adapters::tool::PreparedArguments::prepare(&animus_adapters::tool::ArgumentPreparationRequest {
                current_schema_identity: TOOL_SCHEMA,
                allowed_field_names: &[TOOL_INPUT],
                compatibility_rules: &[],
                fields: &fields,
            })
            .expect("arguments should validate");
        assert_eq!(arguments.schema_identity(), TOOL_SCHEMA);
        assert_eq!(arguments.fields().len(), TOOL_ARGUMENT_COUNT);
        let request = animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
            name: Some(TOOL_NAME),
            arguments: Some(&arguments),
            correlation_id: Some(TOOL_CALL_ID),
            output_policy: animus_adapters::tool::OutputPolicy {
                visible_char_budget: TOOL_VISIBLE_BUDGET,
            },
            mutation_targets: &[TOOL_MUTATION_TARGET],
            required_capability_labels: &[TOOL_CAPABILITY],
            approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved.as_str(),
        })
        .expect("tool request should validate");
        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(request.correlation_id(), TOOL_CALL_ID);
        request
    }

    fn tool_response() -> animus_adapters::ToolResponse {
        let bounded = animus_adapters::tool::BoundedOutput::new(
            &animus_adapters::tool::OutputDraft {
                body: TOOL_OUTPUT,
                full_output_reference: None,
                hash_requirement: animus_adapters::tool::HashRequirement::Required,
            },
            animus_adapters::tool::OutputPolicy {
                visible_char_budget: TOOL_VISIBLE_BUDGET,
            },
        )
        .expect("output should bound");
        animus_adapters::ToolResponse::from_bounded(bounded, animus_adapters::tool::OutputRedactionClass::Safe)
    }

    fn runtime_adapters<'a>(
        model: Option<&'a mut ModelStub>,
        tool: Option<&'a mut ToolStub>,
    ) -> crate::RuntimeAdapters<'a, ModelStub, ToolStub> {
        let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
            crate::RuntimeCapabilityDescriptorDraft::default_model(),
            crate::RuntimeCapabilityDescriptorDraft {
                label: TOOL_CAPABILITY,
                service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
                scope: None,
            },
        ])
        .expect("registry should validate");
        crate::RuntimeAdapters::<ModelStub, ToolStub>::new_optional(model, tool)
            .with_capability_registry(registry)
            .with_tool_definitions(vec![tool_definition()])
    }

    fn tool_definition() -> animus_adapters::tool::Definition {
        animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
            name: Some(TOOL_NAME),
            label: Some(TOOL_NAME),
            description: Some(TOOL_PROMPT_SNIPPET),
            parameter_schema_identity: Some(TOOL_SCHEMA),
            prompt_snippet: Some(TOOL_PROMPT_SNIPPET),
            prompt_guidelines: &[TOOL_PROMPT_GUIDELINE],
            source_identity: Some(TOOL_SOURCE_IDENTITY),
            provenance: Some(TOOL_PROVENANCE),
            capability_labels: &[TOOL_CAPABILITY],
            result_metadata_shape: Some(TOOL_RESULT_SHAPE),
            status: animus_adapters::tool::DefinitionStatus::Active,
        })
        .expect("definition should validate")
    }

    fn response_decision(message: &str) -> crate::RuntimeLoopDecision {
        crate::RuntimeLoopDecision::from_draft(crate::RuntimeLoopDecisionDraft::respond(message))
            .expect("response decision should validate")
    }
}
