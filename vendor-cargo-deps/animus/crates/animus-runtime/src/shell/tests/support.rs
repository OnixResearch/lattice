pub(super) const SUMMARY: &str = "intent: build reliable systems";
pub(super) const MODEL_MESSAGE: &str = "model answer";
pub(super) const TOOL_NAME: &str = "search";
pub(super) const TOOL_INPUT: &str = "query";
pub(super) const TOOL_SCHEMA: &str = "schema.search.v1";
pub(super) const TOOL_ARGUMENT_VALUE: &str = "adapter contracts";
pub(super) const TOOL_CALL_ID: &str = "tool-call-1";
pub(super) const TOOL_VISIBLE_BUDGET: u32 = 64;
pub(super) const TOOL_CAPABILITY: &str = "tool.read";
pub(super) const TOOL_RESULT_SHAPE: &str = "text-summary";
pub(super) const TOOL_PROMPT_SNIPPET: &str = "Use search for indexed project facts.";
pub(super) const TOOL_PROMPT_GUIDELINE: &str = "Do not infer missing search results.";
pub(super) const TOOL_SOURCE_IDENTITY: &str = "host.registry";
pub(super) const TOOL_PROVENANCE: &str = "builtin";
pub(super) const TOOL_OUTPUT: &str = "tool result";
pub(super) const ADAPTER_FAILURE: &str = "adapter failed";
pub(super) const UNRELATED_MODEL_LABEL: &str = "unrelated-model";
pub(super) const UNRELATED_TOOL_LABEL: &str = "unrelated-tool";
pub(super) const ABSENT_SERVICE: &str = "absent";
pub(super) const DENIED_SERVICE: &str = "denied";
pub(super) const INVALID_CONFIG_TURNS: usize = 0;
pub(super) const NO_ADAPTER_CALLS: usize = 0;
pub(super) const EXPECTED_ADAPTER_CALLS: usize = 1;
pub(super) const CUSTOM_REGISTRY_DESCRIPTOR_COUNT: usize = 2;
pub(super) const INITIAL_TURN_BUDGET: usize = 1;
pub(super) const FOLLOWUP_TURN_BUDGET: usize = 2;
const FOLLOWUP_TURN_BUDGET_U32: u32 = 2;
pub(super) const BLANK_TEXT: &str = "   ";
pub(super) const STEEL_POLICY_PROFILE: &str = "default-loop";
pub(super) const STEEL_POLICY_SCRIPT: &str = "scripts/default-loop.scm";
pub(super) const RESPOND_PROGRAM: &str = r#"
(define (animus-decide)
  (animus-respond "model answer"))
"#;
pub(super) const STOP_PROGRAM: &str = r"
(define (animus-decide)
  (animus-stop))
";
pub(super) const MODEL_THEN_RESPOND_PROGRAM: &str = r"
(define (animus-decide)
  (if animus-last-model-message
      (animus-respond animus-last-model-message)
      (animus-model animus-summary)))
";
pub(super) const TOOL_THEN_RESPOND_PROGRAM: &str = r#"
(define (animus-decide)
  (if animus-last-tool-output
      (animus-respond animus-last-tool-output)
      (animus-tool
        "search"
        "schema.search.v1"
        (list (list "query" "adapter contracts"))
        "tool-call-1"
        64
        (list "resource:file:README")
        (list "tool.read")
        "pre_approved")))
"#;
pub(super) const INVALID_DECISION_PROGRAM: &str = r#"
(define (animus-decide)
  "not a decision")
"#;
pub(super) const INVALID_LOAD_PROGRAM: &str = "(define (";
pub(super) const SPINNING_MODEL_PROGRAM: &str = r"
(define (animus-decide)
  (animus-model animus-summary))
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AdapterFailure;

impl core::fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(ADAPTER_FAILURE)
    }
}

pub(super) struct ModelStub {
    fail: bool,
    pub(super) calls: usize,
}

impl ModelStub {
    pub(super) fn ok() -> Self {
        Self {
            fail: false,
            calls: NO_ADAPTER_CALLS,
        }
    }

    pub(super) fn failing() -> Self {
        Self {
            fail: true,
            calls: NO_ADAPTER_CALLS,
        }
    }
}

impl animus_adapters::ComponentAdapter<animus_adapters::ModelRequest, animus_adapters::ModelResponse> for ModelStub {
    type Error = AdapterFailure;

    fn call(&mut self, request: &animus_adapters::ModelRequest) -> Result<animus_adapters::ModelResponse, Self::Error> {
        assert_eq!(request.prompt(), SUMMARY);
        self.calls = self.calls.saturating_add(EXPECTED_ADAPTER_CALLS);
        if self.fail {
            return Err(AdapterFailure);
        }

        animus_adapters::ModelResponse::new(MODEL_MESSAGE).map_err(|_error| AdapterFailure)
    }
}

pub(super) struct ToolStub {
    fail: bool,
    pub(super) calls: usize,
}

impl ToolStub {
    pub(super) fn ok() -> Self {
        Self {
            fail: false,
            calls: NO_ADAPTER_CALLS,
        }
    }

    pub(super) fn failing() -> Self {
        Self {
            fail: true,
            calls: NO_ADAPTER_CALLS,
        }
    }
}

impl animus_adapters::ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse> for ToolStub {
    type Error = AdapterFailure;

    fn call(&mut self, request: &animus_adapters::ToolRequest) -> Result<animus_adapters::ToolResponse, Self::Error> {
        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(request.schema_identity(), TOOL_SCHEMA);
        assert_eq!(request.arguments()[NO_ADAPTER_CALLS].name(), TOOL_INPUT);
        assert_eq!(request.arguments()[NO_ADAPTER_CALLS].value(), TOOL_ARGUMENT_VALUE);
        assert_eq!(request.correlation_id(), TOOL_CALL_ID);
        assert_eq!(request.output_policy().visible_char_budget, TOOL_VISIBLE_BUDGET);
        self.calls = self.calls.saturating_add(EXPECTED_ADAPTER_CALLS);
        if self.fail {
            return Err(AdapterFailure);
        }

        Ok(tool_response())
    }
}

pub(super) fn run_program_with_initial_turn_budget(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_initial_turn_budget(),
        runtime_adapters(Some(model), Some(tool)),
    )
}

pub(super) fn run_program_with_followup_turn_budget(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), Some(tool)),
    )
}

pub(super) fn run_policy_program_with_followup_turn_budget(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    let profile = policy_profile_for_program(program, FOLLOWUP_TURN_BUDGET_U32);
    let script = policy_script_for_program(program);
    let policy = super::super::SteelPolicyRuntime::new(&profile, script);
    super::super::run_with_steel_policy(
        &policy,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), Some(tool)),
    )
}

pub(super) fn run_policy_program_with_bad_profile(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    let profile = policy_profile_for_program(program, FOLLOWUP_TURN_BUDGET_U32);
    let script = animus_steel::SteelPolicyScript::new("other-loop", STEEL_POLICY_SCRIPT, program);
    let policy = super::super::SteelPolicyRuntime::new(&profile, script);
    super::super::run_with_steel_policy(
        &policy,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), Some(tool)),
    )
}

pub(super) fn run_program_with_missing_model_service(program: &str, tool: &mut ToolStub) -> crate::RunReport {
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(None, Some(tool)),
    )
}

pub(super) fn run_program_with_missing_tool_service(program: &str, model: &mut ModelStub) -> crate::RunReport {
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), None),
    )
}

pub(super) fn run_program_with_unrelated_model_capability(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
        crate::RuntimeCapabilityDescriptorDraft {
            label: UNRELATED_MODEL_LABEL,
            service_kind: crate::RuntimeCapabilityServiceKind::Model.as_str(),
            scope: None,
        },
        crate::RuntimeCapabilityDescriptorDraft {
            label: TOOL_CAPABILITY,
            service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
            scope: None,
        },
    ])
    .expect("registry should validate");
    assert_eq!(registry.descriptors().len(), CUSTOM_REGISTRY_DESCRIPTOR_COUNT);
    assert!(registry.contains_service(crate::RuntimeCapabilityServiceKind::Model));
    assert!(registry.contains_service(crate::RuntimeCapabilityServiceKind::Tool));
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), Some(tool)).with_capability_registry(registry),
    )
}

pub(super) fn run_program_with_unrelated_tool_capability(
    program: &str,
    model: &mut ModelStub,
    tool: &mut ToolStub,
) -> crate::RunReport {
    let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
        crate::RuntimeCapabilityDescriptorDraft::default_model(),
        crate::RuntimeCapabilityDescriptorDraft {
            label: UNRELATED_TOOL_LABEL,
            service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
            scope: None,
        },
    ])
    .expect("registry should validate");
    assert_eq!(registry.descriptors().len(), CUSTOM_REGISTRY_DESCRIPTOR_COUNT);
    assert!(registry.contains_service(crate::RuntimeCapabilityServiceKind::Model));
    assert!(registry.contains_service(crate::RuntimeCapabilityServiceKind::Tool));
    super::super::run_with_steel_program(
        program,
        &runtime_input(),
        runtime_config_with_followup_turn_budget(),
        runtime_adapters(Some(model), Some(tool)).with_capability_registry(registry),
    )
}

pub(super) fn runtime_input() -> crate::RuntimeInput {
    crate::RuntimeInput::new(SUMMARY).expect("summary should be valid")
}

pub(super) fn tool_response() -> animus_adapters::ToolResponse {
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
    .expect("tool output should bound");
    animus_adapters::ToolResponse::from_bounded(bounded, animus_adapters::tool::OutputRedactionClass::Safe)
}

fn runtime_adapters<'a>(
    model: Option<&'a mut ModelStub>,
    tool: Option<&'a mut ToolStub>,
) -> super::super::RuntimeAdapters<'a, ModelStub, ToolStub> {
    let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
        crate::RuntimeCapabilityDescriptorDraft::default_model(),
        crate::RuntimeCapabilityDescriptorDraft {
            label: TOOL_CAPABILITY,
            service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
            scope: None,
        },
    ])
    .expect("registry should validate");
    super::super::RuntimeAdapters::<ModelStub, ToolStub>::new_optional(model, tool)
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
    .expect("tool definition should validate")
}

fn policy_profile_for_program(program: &str, max_turns: u32) -> animus_steel::SteelPolicyProfile {
    let script_hash = animus_steel::steel_script_hash(program);
    animus_steel::SteelPolicyProfile::from_draft(&animus_steel::SteelPolicyProfileDraft {
        name: STEEL_POLICY_PROFILE,
        enabled: true,
        is_default: true,
        script_label: STEEL_POLICY_SCRIPT,
        script_hash: script_hash.as_str(),
        max_turns,
        allowed_host_actions: &["model", "tool", "respond", "completion-candidate", "stop"],
        fallback_mode: "block",
        receipt_redaction: "hash-only",
    })
    .expect("policy fixture should be valid")
}

fn policy_script_for_program(program: &str) -> animus_steel::SteelPolicyScript<'_> {
    animus_steel::SteelPolicyScript::new(STEEL_POLICY_PROFILE, STEEL_POLICY_SCRIPT, program)
}

fn runtime_config_with_initial_turn_budget() -> crate::RuntimeConfig {
    crate::RuntimeConfig::from_max_turn_count(INITIAL_TURN_BUDGET).expect("one max turn should be valid")
}

pub(super) fn runtime_config_with_followup_turn_budget() -> crate::RuntimeConfig {
    crate::RuntimeConfig::from_max_turn_count(FOLLOWUP_TURN_BUDGET).expect("two max turns should be valid")
}
