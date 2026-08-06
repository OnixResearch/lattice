/// Effect lifecycle observation supplied by the executing shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationEffectLifecycle {
    /// Request emitted by pure update.
    Requested,
    /// Shell began execution.
    Started,
    /// Typed terminal result.
    Result(SimulationEffectResult),
}

/// Stable lifecycle-tracker error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationEffectDiagnosticCode {
    /// Request identity was already active or completed.
    DuplicateRequest,
    /// Start or result referenced an unknown request.
    UnknownRequest,
    /// Start was emitted more than once.
    DuplicateStart,
    /// A terminal result was emitted more than once or arrived late.
    DuplicateOrLateResult,
    /// Effect class changed during one request lifecycle.
    EffectClassMismatch,
}

/// Stable lifecycle-tracker diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationEffectDiagnostic {
    /// Machine-comparable code.
    pub code: SimulationEffectDiagnosticCode,
    /// Safe request identity.
    pub request_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EffectStatus {
    class_projection: u32,
    started: bool,
    terminal: bool,
}

/// Deterministic request/start/result lifecycle tracker.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SimulationEffectTracker {
    requests: std::collections::BTreeMap<String, EffectStatus>,
}

impl SimulationEffectTracker {
    /// Applies one lifecycle fact.
    ///
    /// # Errors
    ///
    /// Returns duplicate, late, unknown, or class-mismatch diagnostics.
    pub fn observe(
        &mut self,
        request_id: &str,
        effect_class: &SimulationEffectClassId,
        lifecycle: SimulationEffectLifecycle,
    ) -> Result<(), SimulationEffectDiagnostic> {
        let class_projection = super::nomenclature::project_simulation_id(effect_class.as_str());
        match lifecycle {
            SimulationEffectLifecycle::Requested => {
                if self.requests.contains_key(request_id) {
                    return Err(effect_diagnostic(
                        SimulationEffectDiagnosticCode::DuplicateRequest,
                        request_id,
                    ));
                }
                self.requests.insert(
                    request_id.to_string(),
                    EffectStatus {
                        class_projection,
                        started: false,
                        terminal: false,
                    },
                );
                Ok(())
            }
            SimulationEffectLifecycle::Started => {
                let status = self.requests.get_mut(request_id).ok_or_else(|| {
                    effect_diagnostic(SimulationEffectDiagnosticCode::UnknownRequest, request_id)
                })?;
                require_class(*status, class_projection, request_id)?;
                if status.started {
                    return Err(effect_diagnostic(
                        SimulationEffectDiagnosticCode::DuplicateStart,
                        request_id,
                    ));
                }
                if status.terminal {
                    return Err(effect_diagnostic(
                        SimulationEffectDiagnosticCode::DuplicateOrLateResult,
                        request_id,
                    ));
                }
                status.started = true;
                Ok(())
            }
            SimulationEffectLifecycle::Result(_) => {
                let status = self.requests.get_mut(request_id).ok_or_else(|| {
                    effect_diagnostic(SimulationEffectDiagnosticCode::UnknownRequest, request_id)
                })?;
                require_class(*status, class_projection, request_id)?;
                if status.terminal {
                    return Err(effect_diagnostic(
                        SimulationEffectDiagnosticCode::DuplicateOrLateResult,
                        request_id,
                    ));
                }
                status.terminal = true;
                Ok(())
            }
        }
    }
}

fn require_class(
    status: EffectStatus,
    class_projection: u32,
    request_id: &str,
) -> Result<(), SimulationEffectDiagnostic> {
    if status.class_projection == class_projection {
        Ok(())
    } else {
        Err(effect_diagnostic(
            SimulationEffectDiagnosticCode::EffectClassMismatch,
            request_id,
        ))
    }
}

fn valid_ready_sources(sources: &[String]) -> bool {
    if sources.is_empty() || sources.len() > MAX_READY_SOURCES {
        return false;
    }
    sources
        .windows(2)
        .all(|pair| pair[0] < pair[1] && valid_semantic_id(&pair[0]))
        && sources.last().is_some_and(|last| valid_semantic_id(last))
}

fn valid_semantic_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

const fn control_issue(
    code: SimulationControlDiagnosticCode,
    subject: &'static str,
) -> SimulationControlDiagnostic {
    SimulationControlDiagnostic { code, subject }
}

fn effect_diagnostic(
    code: SimulationEffectDiagnosticCode,
    request_id: &str,
) -> SimulationEffectDiagnostic {
    SimulationEffectDiagnostic {
        code,
        request_id: request_id.chars().take(96).collect(),
    }
}
