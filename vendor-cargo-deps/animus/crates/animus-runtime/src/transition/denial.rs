/// Completes a runtime report after middleware blocks an adapter boundary.
#[must_use]
pub fn complete_with_guard_block(
    state: super::TurnState,
    request: super::AdapterRequest,
    hook: String,
    reason: String,
) -> crate::RunReport {
    let boundary = request.boundary();
    let state = state
        .with_event(crate::RuntimeEvent::HookBlocked {
            boundary,
            hook,
            reason: reason.clone(),
        })
        .with_event(denied_event(request, &reason));
    state.complete(crate::RuntimeStatus::Failed {
        failure: crate::RuntimeFailure::Middleware {
            boundary,
            message: reason,
        },
    })
}

/// Completes a runtime report after middleware fails an adapter boundary.
#[must_use]
pub fn complete_with_guard_failure(
    state: super::TurnState,
    request: super::AdapterRequest,
    error: &crate::MiddlewareError,
) -> crate::RunReport {
    let boundary = error.boundary();
    let hook = String::from(error.hook());
    let message = String::from(error.message());
    let state = state
        .with_event(crate::RuntimeEvent::HookFailed {
            boundary,
            hook,
            message: message.clone(),
        })
        .with_event(denied_event(request, &message));
    state.complete(crate::RuntimeStatus::Failed {
        failure: crate::RuntimeFailure::Middleware { boundary, message },
    })
}

fn denied_event(request: super::AdapterRequest, reason: &str) -> crate::RuntimeEvent {
    match request {
        super::AdapterRequest::Model { request } => crate::RuntimeEvent::ModelDenied {
            request,
            reason: String::from(reason),
        },
        super::AdapterRequest::Tool { request } => crate::RuntimeEvent::ToolDenied {
            request,
            reason: String::from(reason),
        },
    }
}
