/// Validates an ordered controlled history without consulting ambient state.
#[must_use]
pub fn validate_control_history(
    registry: &SimulationIdentityRegistry,
    history: &[SimulationControlInput],
) -> Vec<SimulationControlDiagnostic> {
    let mut issues = ControlIssues::default();
    if history.len() > MAX_CONTROL_INPUTS {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::ScriptTooLarge,
            "history",
        ));
    }
    let mut state = ControlValidationState::default();
    for input in history.iter().take(MAX_CONTROL_INPUTS.saturating_add(1)) {
        validate_control_input(registry, input, &mut state, &mut issues);
    }
    issues.into_inner()
}

#[derive(Default)]
struct ControlIssues(Vec<SimulationControlDiagnostic>);

impl ControlIssues {
    fn push(&mut self, issue: SimulationControlDiagnostic) {
        if self.0.len() < MAX_CONTROL_ISSUES {
            self.0.push(issue);
        }
    }

    fn into_inner(self) -> Vec<SimulationControlDiagnostic> {
        self.0
    }
}

#[derive(Default)]
struct ControlValidationState {
    previous_tick: Option<u64>,
    completed_requests: std::collections::BTreeSet<String>,
}

fn validate_control_input(
    registry: &SimulationIdentityRegistry,
    input: &SimulationControlInput,
    state: &mut ControlValidationState,
    issues: &mut ControlIssues,
) {
    match input {
        SimulationControlInput::LogicalTick { source_id, tick } => {
            validate_tick(source_id, *tick, state, issues);
        }
        SimulationControlInput::FiniteChoice {
            choice_point,
            bound,
            selected,
        } => validate_choice(
            registry,
            choice_point,
            FiniteChoiceValue {
                bound: *bound,
                selected: *selected,
            },
            issues,
        ),
        SimulationControlInput::EventSelection {
            ready_sources,
            selected_source,
        } => validate_event_selection(ready_sources, selected_source, issues),
        SimulationControlInput::EffectResult {
            request_id,
            effect_class,
            ..
        } => validate_effect_result(registry, request_id, effect_class, state, issues),
        SimulationControlInput::ForbiddenSource(source) => {
            issues.push(control_issue(
                forbidden_source_code(*source),
                "ambient-source",
            ));
        }
    }
}

fn validate_tick(
    source_id: &str,
    tick: u64,
    state: &mut ControlValidationState,
    issues: &mut ControlIssues,
) {
    if !valid_semantic_id(source_id) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::UnknownSchedulerSource,
            "scheduler",
        ));
    }
    if state.previous_tick.is_some_and(|previous| tick < previous) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::RegressingLogicalTick,
            "logical-tick",
        ));
    }
    state.previous_tick = Some(tick);
}

#[derive(Clone, Copy)]
struct FiniteChoiceValue {
    bound: u32,
    selected: u32,
}

fn validate_choice(
    registry: &SimulationIdentityRegistry,
    choice_point: &SimulationChoicePointId,
    value: FiniteChoiceValue,
    issues: &mut ControlIssues,
) {
    if !registry.contains(SimulationIdentityKind::ChoicePoint, choice_point.as_str()) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::UnknownChoicePoint,
            "choice-point",
        ));
    }
    if value.bound == 0 {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::EmptyChoiceSet,
            "choice-bound",
        ));
    } else if value.selected >= value.bound {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::ChoiceOutOfRange,
            "choice-selection",
        ));
    }
}

fn validate_event_selection(
    ready_sources: &[String],
    selected_source: &str,
    issues: &mut ControlIssues,
) {
    if !valid_ready_sources(ready_sources) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::InvalidReadySources,
            "ready-sources",
        ));
    }
    if !ready_sources.iter().any(|source| source == selected_source) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::SelectedSourceNotReady,
            "selected-source",
        ));
    }
}

fn validate_effect_result(
    registry: &SimulationIdentityRegistry,
    request_id: &str,
    effect_class: &SimulationEffectClassId,
    state: &mut ControlValidationState,
    issues: &mut ControlIssues,
) {
    debug_assert!(state.completed_requests.len() <= MAX_CONTROL_INPUTS);
    debug_assert!(issues.0.len() <= MAX_CONTROL_ISSUES);
    if !valid_semantic_id(request_id) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::InvalidRequestIdentity,
            "request-id",
        ));
    }
    if !registry.contains(SimulationIdentityKind::EffectClass, effect_class.as_str()) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::UnknownEffectClass,
            "effect-class",
        ));
    }
    if state.completed_requests.len() >= MAX_CONTROL_INPUTS {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::ScriptTooLarge,
            "effect-results",
        ));
    } else if !state.completed_requests.insert(request_id.to_string()) {
        issues.push(control_issue(
            SimulationControlDiagnosticCode::DuplicateEffectResult,
            "request-id",
        ));
    }
}

const fn forbidden_source_code(
    source: ForbiddenSimulationSource,
) -> SimulationControlDiagnosticCode {
    match source {
        ForbiddenSimulationSource::HostClock => SimulationControlDiagnosticCode::HostClockForbidden,
        ForbiddenSimulationSource::HostRandom => {
            SimulationControlDiagnosticCode::HostRandomForbidden
        }
        ForbiddenSimulationSource::DirectExternalIo => {
            SimulationControlDiagnosticCode::DirectExternalIoForbidden
        }
    }
}
