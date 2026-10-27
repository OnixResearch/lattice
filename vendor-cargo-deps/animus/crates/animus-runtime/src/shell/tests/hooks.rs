const TOOL_GUARD_LABEL: &str = "tool_guard";
const MODEL_MUTATOR_LABEL: &str = "model_mutator";
const MODEL_REPLACER_LABEL: &str = "model_replacer";
const TOOL_BLOCK_REASON: &str = "capability_denied";
const TOOL_HOOK_FAILURE: &str = "tool hook failed";
const FIRST_MODEL_SUFFIX: &str = " [first]";
const SECOND_MODEL_SUFFIX: &str = " [second]";
const FIRST_MODEL_PROMPT: &str = "intent: build reliable systems [first]";
const FINAL_MODEL_PROMPT: &str = "intent: build reliable systems [first] [second]";
const MODE_NEUTRAL_FORBIDDEN_TUI: &str = "tui";
const MODE_NEUTRAL_FORBIDDEN_TERMINAL: &str = "terminal";
const CANCELLATION_REASON: &str = "host_cancelled";
const COMPACTION_SUMMARY: &str = "safe compaction summary";
const OBSERVER_LABEL: &str = "runtime_observer";
const OBSERVER_FAILURE: &str = "observer failed";
const OBSERVABILITY_SINK_FAILED: &str = "observability_sink_failed";

struct BlockingToolHook;

impl crate::RuntimeMiddleware for BlockingToolHook {
    fn label(&self) -> &str {
        TOOL_GUARD_LABEL
    }

    fn before_tool_dispatch(
        &mut self,
        _request: animus_adapters::ToolRequest,
    ) -> Result<crate::HookDecision<animus_adapters::ToolRequest>, crate::MiddlewareError> {
        Ok(crate::HookDecision::Block {
            reason: String::from(TOOL_BLOCK_REASON),
        })
    }
}

struct FailingToolHook;

impl crate::RuntimeMiddleware for FailingToolHook {
    fn label(&self) -> &str {
        TOOL_GUARD_LABEL
    }

    fn before_tool_dispatch(
        &mut self,
        _request: animus_adapters::ToolRequest,
    ) -> Result<crate::HookDecision<animus_adapters::ToolRequest>, crate::MiddlewareError> {
        Err(crate::MiddlewareError::new(
            crate::HookBoundary::ToolDispatch,
            TOOL_GUARD_LABEL,
            TOOL_HOOK_FAILURE,
        ))
    }
}

struct AppendModelHook;

impl crate::RuntimeMiddleware for AppendModelHook {
    fn label(&self) -> &str {
        MODEL_MUTATOR_LABEL
    }

    fn before_model_dispatch(
        &mut self,
        request: animus_adapters::ModelRequest,
    ) -> Result<crate::HookDecision<animus_adapters::ModelRequest>, crate::MiddlewareError> {
        let prompt = format!("{}{FIRST_MODEL_SUFFIX}", request.prompt());
        Ok(crate::HookDecision::Continue {
            value: animus_adapters::ModelRequest::new(&prompt)
                .expect("mutated model prompt should be valid"),
        })
    }
}

#[derive(Default)]
struct ObserveAndReplaceModelHook {
    observed: Vec<String>,
}

#[derive(Default)]
struct EventObserverHook {
    observed: Vec<crate::RuntimeEventKind>,
}

impl crate::RuntimeMiddleware for EventObserverHook {
    fn label(&self) -> &str {
        OBSERVER_LABEL
    }

    fn observe(&mut self, event: &crate::RuntimeEvent) -> Result<(), crate::MiddlewareError> {
        self.observed.push(event.kind());
        Ok(())
    }
}

struct FailingObserverHook;

impl crate::RuntimeMiddleware for FailingObserverHook {
    fn label(&self) -> &str {
        OBSERVER_LABEL
    }

    fn observe(&mut self, _event: &crate::RuntimeEvent) -> Result<(), crate::MiddlewareError> {
        Err(crate::MiddlewareError::new(
            crate::HookBoundary::Observe,
            OBSERVER_LABEL,
            OBSERVER_FAILURE,
        ))
    }
}

impl crate::RuntimeMiddleware for ObserveAndReplaceModelHook {
    fn label(&self) -> &str {
        MODEL_REPLACER_LABEL
    }

    fn before_model_dispatch(
        &mut self,
        request: animus_adapters::ModelRequest,
    ) -> Result<crate::HookDecision<animus_adapters::ModelRequest>, crate::MiddlewareError> {
        self.observed.push(String::from(request.prompt()));
        let prompt = format!("{}{SECOND_MODEL_SUFFIX}", request.prompt());
        Ok(crate::HookDecision::Replace {
            value: animus_adapters::ModelRequest::new(&prompt)
                .expect("replacement model prompt should be valid"),
        })
    }
}

fn run_with_tool_hook(
    hook: &mut dyn crate::RuntimeMiddleware,
) -> (
    crate::RunReport,
    super::support::ModelStub,
    super::support::ToolStub,
) {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let hooks: Vec<&mut dyn crate::RuntimeMiddleware> = vec![hook];
    let result = super::super::run_with_steel_program(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &super::support::runtime_input(),
        super::support::runtime_config_with_followup_turn_budget(),
        super::super::RuntimeAdapters::new(&mut model, &mut tool)
            .with_middleware(crate::RuntimeMiddlewareStack::from_hooks(hooks)),
    );
    (result, model, tool)
}

fn event_kinds(report: &crate::RunReport) -> Vec<crate::RuntimeEventKind> {
    report
        .events()
        .iter()
        .map(crate::RuntimeEvent::kind)
        .collect()
}

#[test]
fn model_run_events_preserve_turn_order() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert_eq!(
        event_kinds(&result),
        vec![
            crate::RuntimeEventKind::RuntimeStarted,
            crate::RuntimeEventKind::ContextReplayed,
            crate::RuntimeEventKind::LoopStarted,
            crate::RuntimeEventKind::TurnStarted,
            crate::RuntimeEventKind::LoopDecision,
            crate::RuntimeEventKind::ModelRequested,
            crate::RuntimeEventKind::ModelResponded,
            crate::RuntimeEventKind::TurnStarted,
            crate::RuntimeEventKind::LoopDecision,
            crate::RuntimeEventKind::FinalResponse,
            crate::RuntimeEventKind::Shutdown,
        ]
    );
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn headless_subscriber_receives_mode_neutral_lifecycle_facts() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );
    let labels: Vec<&str> = event_kinds(&result)
        .iter()
        .map(|kind| kind.as_str())
        .collect();

    assert!(labels.contains(&crate::RuntimeEventKind::RuntimeStarted.as_str()));
    assert!(labels.contains(&crate::RuntimeEventKind::ToolRequested.as_str()));
    assert!(labels.contains(&crate::RuntimeEventKind::ToolResponded.as_str()));
    assert!(labels.contains(&crate::RuntimeEventKind::Shutdown.as_str()));
    assert!(
        labels
            .iter()
            .all(|label| !label.contains(MODE_NEUTRAL_FORBIDDEN_TUI))
    );
    assert!(
        labels
            .iter()
            .all(|label| !label.contains(MODE_NEUTRAL_FORBIDDEN_TERMINAL))
    );
}

#[test]
fn tool_guard_denies_before_adapter_dispatch() {
    let mut hook = BlockingToolHook;
    let (result, model, tool) = run_with_tool_hook(&mut hook);

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::Middleware { boundary, message }
        } if *boundary == crate::HookBoundary::ToolDispatch && message == TOOL_BLOCK_REASON
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::HookBlocked { boundary, hook, reason }
            if *boundary == crate::HookBoundary::ToolDispatch
                && hook == TOOL_GUARD_LABEL
                && reason == TOOL_BLOCK_REASON
    )));
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::ToolDenied { reason, .. } if reason == TOOL_BLOCK_REASON
    )));
    assert!(
        result
            .events()
            .iter()
            .all(|event| { !matches!(event, crate::RuntimeEvent::ToolResponded { .. }) })
    );
}

#[test]
fn hook_error_fails_safe_before_tool_adapter_dispatch() {
    let mut hook = FailingToolHook;
    let (result, model, tool) = run_with_tool_hook(&mut hook);

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::Middleware { boundary, message }
        } if *boundary == crate::HookBoundary::ToolDispatch && message == TOOL_HOOK_FAILURE
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::HookFailed { boundary, hook, message }
            if *boundary == crate::HookBoundary::ToolDispatch
                && hook == TOOL_GUARD_LABEL
                && message == TOOL_HOOK_FAILURE
    )));
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::ToolDenied { reason, .. } if reason == TOOL_HOOK_FAILURE
    )));
}

#[test]
fn observe_only_middleware_receives_runtime_events_in_report_order() {
    let mut observer = EventObserverHook::default();
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = {
        let hooks: Vec<&mut dyn crate::RuntimeMiddleware> = vec![&mut observer];
        super::super::run_with_steel_program(
            super::support::MODEL_THEN_RESPOND_PROGRAM,
            &super::support::runtime_input(),
            super::support::runtime_config_with_followup_turn_budget(),
            super::super::RuntimeAdapters::new(&mut model, &mut tool)
                .with_middleware(crate::RuntimeMiddlewareStack::from_hooks(hooks)),
        )
    };

    assert_eq!(observer.observed, event_kinds(&result));
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn observe_only_middleware_failure_is_best_effort_and_bounded() {
    let mut observer = FailingObserverHook;
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = {
        let hooks: Vec<&mut dyn crate::RuntimeMiddleware> = vec![&mut observer];
        super::super::run_with_steel_program(
            super::support::MODEL_THEN_RESPOND_PROGRAM,
            &super::support::runtime_input(),
            super::support::runtime_config_with_followup_turn_budget(),
            super::super::RuntimeAdapters::new(&mut model, &mut tool)
                .with_middleware(crate::RuntimeMiddlewareStack::from_hooks(hooks)),
        )
    };

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::FinalResponse { .. }
    ));
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::HookFailed { boundary, hook, message }
            if *boundary == crate::HookBoundary::Observe
                && hook == OBSERVER_LABEL
                && message == OBSERVABILITY_SINK_FAILED
    )));
    assert!(result.events().iter().all(|event| {
        !matches!(
            event,
            crate::RuntimeEvent::HookFailed { message, .. } if message == OBSERVER_FAILURE
        )
    }));
}

#[test]
fn hook_chain_applies_mutate_then_replace_in_order() {
    let mut first = AppendModelHook;
    let mut second = ObserveAndReplaceModelHook::default();
    let hooks: Vec<&mut dyn crate::RuntimeMiddleware> = vec![&mut first, &mut second];
    let mut stack = crate::RuntimeMiddlewareStack::from_hooks(hooks);
    let request = animus_adapters::ModelRequest::new(super::support::SUMMARY)
        .expect("base model request should be valid");
    let outcome = stack.prepare_model_dispatch(request);

    let crate::hooks::chain::MiddlewareOutcome::Accepted(request) = outcome else {
        panic!("model hook chain should accept the final request");
    };
    assert_eq!(request.prompt(), FINAL_MODEL_PROMPT);
    assert_eq!(
        second.observed.as_slice(),
        &[String::from(FIRST_MODEL_PROMPT)]
    );
}

#[test]
fn guarded_boundaries_fail_safe_for_sensitive_surfaces() {
    let guarded = [
        crate::HookBoundary::ModelDispatch,
        crate::HookBoundary::ToolDispatch,
        crate::HookBoundary::Compaction,
        crate::HookBoundary::Replay,
        crate::HookBoundary::SessionReplacement,
    ];

    for boundary in guarded {
        assert!(boundary.fails_safe());
        assert!(!boundary.as_str().is_empty());
    }
    assert!(!crate::HookBoundary::Observe.fails_safe());
}

#[test]
fn lifecycle_dtos_cover_cancellation_and_compaction() {
    let cancelled = crate::RuntimeEvent::Cancelled {
        reason: String::from(CANCELLATION_REASON),
    };
    let compaction = crate::RuntimeEvent::CompactionRequested {
        summary: String::from(COMPACTION_SUMMARY),
    };

    assert_eq!(cancelled.kind(), crate::RuntimeEventKind::Cancelled);
    assert_eq!(
        compaction.kind(),
        crate::RuntimeEventKind::CompactionRequested
    );
}
