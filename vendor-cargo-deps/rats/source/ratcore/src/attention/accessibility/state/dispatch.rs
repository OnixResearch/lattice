/// Optional semantic action input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticActionValue {
    /// Integer range value.
    Integer(i64),
    /// Text value.
    Text(String),
}

/// Renderer- or platform-normalized semantic action request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticActionRequest {
    /// Snapshot generation observed by the requester.
    pub generation: super::super::SemanticGeneration,
    /// Target semantic node.
    pub node: super::super::SemanticNodeId,
    /// Requested bounded action.
    pub action: super::super::SemanticAction,
    /// Optional checked action value.
    pub value: Option<SemanticActionValue>,
}

/// Typed application intent produced only for an eligible request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticActionIntent {
    /// Semantic node that accepted the request.
    pub node: super::super::SemanticNodeId,
    /// Accepted semantic action.
    pub action: super::super::SemanticAction,
    /// Application action registry target.
    pub target: crate::nexus_core::ActionId,
    /// Validated optional value.
    pub value: Option<SemanticActionValue>,
}

/// Semantic action routing result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticActionOutcome {
    /// Application intent when the request is eligible.
    pub intent: Option<SemanticActionIntent>,
    /// Stable rejection diagnostics.
    pub issues: Vec<super::super::SemanticIssue>,
}

/// Routes an action through the current accepted semantic projection.
#[must_use]
pub fn route_semantic_action(
    snapshot: &super::super::SemanticSnapshot,
    request: &SemanticActionRequest,
) -> SemanticActionOutcome {
    let target = match validated_target(snapshot, request) {
        Ok(target) => target,
        Err(issue) => return rejected(issue),
    };
    SemanticActionOutcome {
        intent: Some(SemanticActionIntent {
            node: request.node.clone(),
            action: request.action,
            target: target.clone(),
            value: request.value.clone(),
        }),
        issues: Vec::new(),
    }
}

fn validated_target<'a>(
    snapshot: &'a super::super::SemanticSnapshot,
    request: &SemanticActionRequest,
) -> Result<&'a crate::nexus_core::ActionId, super::super::SemanticIssue> {
    let node = current_node(snapshot, request)?;
    validate_availability(node, request)?;
    let target = exposed_target(node, request)?;
    validate_action_value(node, request)?;
    Ok(target)
}

fn exposed_target<'a>(
    node: &'a super::super::SemanticNode,
    request: &SemanticActionRequest,
) -> Result<&'a crate::nexus_core::ActionId, super::super::SemanticIssue> {
    node.actions.get(&request.action).ok_or_else(|| {
        issue(
            super::super::SemanticIssueCode::UnsupportedAction,
            request,
            "semantic node does not expose the requested action",
        )
    })
}

fn current_node<'a>(
    snapshot: &'a super::super::SemanticSnapshot,
    request: &SemanticActionRequest,
) -> Result<&'a super::super::SemanticNode, super::super::SemanticIssue> {
    if request.generation != snapshot.generation {
        return Err(issue(
            super::super::SemanticIssueCode::StaleGeneration,
            request,
            "semantic action observed a stale generation",
        ));
    }
    snapshot.node(&request.node).ok_or_else(|| {
        issue(super::super::SemanticIssueCode::UnknownNode, request, "semantic action target is missing")
    })
}

fn validate_availability(
    node: &super::super::SemanticNode,
    request: &SemanticActionRequest,
) -> Result<(), super::super::SemanticIssue> {
    node_availability_issue(node, request).map_or(Ok(()), Err)
}

fn validate_action_value(
    node: &super::super::SemanticNode,
    request: &SemanticActionRequest,
) -> Result<(), super::super::SemanticIssue> {
    valid_action_value(node.value.as_ref(), request.action, request.value.as_ref())
        .then_some(())
        .ok_or_else(|| {
            issue(
                super::super::SemanticIssueCode::InvalidActionValue,
                request,
                "semantic action value is missing, malformed, or outside accepted bounds",
            )
        })
}

fn node_availability_issue(
    node: &super::super::SemanticNode,
    request: &SemanticActionRequest,
) -> Option<super::super::SemanticIssue> {
    if node.state.is_hidden() {
        return Some(issue(
            super::super::SemanticIssueCode::HiddenNode,
            request,
            "hidden semantic node cannot receive actions",
        ));
    }
    node.state.is_disabled().then(|| {
        issue(
            super::super::SemanticIssueCode::DisabledNode,
            request,
            "disabled semantic node cannot receive actions",
        )
    })
}

fn valid_action_value(
    accepted_value: Option<&super::super::SemanticValue>,
    action: super::super::SemanticAction,
    request_value: Option<&SemanticActionValue>,
) -> bool {
    if action != super::super::SemanticAction::SetValue {
        return request_value.is_none();
    }
    match (accepted_value, request_value) {
        (Some(super::super::SemanticValue::Range(range)), Some(SemanticActionValue::Integer(value))) => {
            *value >= range.minimum && *value <= range.maximum
        }
        (Some(super::super::SemanticValue::Text(_)), Some(SemanticActionValue::Text(value))) => {
            !value.trim().is_empty()
        }
        _ => false,
    }
}

fn issue(
    code: super::super::SemanticIssueCode,
    request: &SemanticActionRequest,
    message: &'static str,
) -> super::super::SemanticIssue {
    super::super::SemanticIssue::new(code, request.node.as_str(), message)
}

fn rejected(issue: super::super::SemanticIssue) -> SemanticActionOutcome {
    SemanticActionOutcome {
        intent: None,
        issues: vec![issue],
    }
}
