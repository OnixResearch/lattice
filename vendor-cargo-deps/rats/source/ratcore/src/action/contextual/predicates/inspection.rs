struct PredicateScan {
    diagnostics: Vec<super::super::model::RoutingDiagnostic>,
    node_count: u32,
    diagnostic_limit: u32,
}

impl PredicateScan {
    fn new(diagnostic_limit: u32) -> Self {
        let count = usize::try_from(diagnostic_limit).map_or(0, |value| value);
        Self {
            diagnostics: Vec::with_capacity(count),
            node_count: 0,
            diagnostic_limit,
        }
    }

    fn push(&mut self, diagnostic: super::super::model::RoutingDiagnostic) {
        let count = usize::try_from(self.diagnostic_limit).map_or(0, |value| value);
        if self.diagnostics.len() < count {
            self.diagnostics.push(diagnostic);
        }
    }
}

#[derive(Clone, Copy)]
struct InspectFrame<'a> {
    predicate: &'a super::ActionContextPredicate,
    depth: u32,
}

struct InspectStack<'a> {
    items: Vec<InspectFrame<'a>>,
}

impl<'a> InspectStack<'a> {
    fn new(count: usize) -> Self {
        Self {
            items: Vec::with_capacity(count),
        }
    }

    fn push(&mut self, frame: InspectFrame<'a>) -> bool {
        if self.items.len() >= self.items.capacity() {
            return false;
        }
        self.items.push(frame);
        true
    }

    fn pop(&mut self) -> Option<InspectFrame<'a>> {
        self.items.pop()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn capacity(&self) -> usize {
        self.items.capacity()
    }
}

pub(in crate::nexus_core::contextual) fn validate_predicate(
    predicate: &super::ActionContextPredicate,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    limits: super::ActionContextPredicateLimits,
) -> Vec<super::super::model::RoutingDiagnostic> {
    if !limits.valid() {
        return vec![super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::MalformedPredicate,
            "predicate.limits",
            "predicate limits must be nonzero and within hard safety bounds",
        )];
    }
    debug_assert!(limits.max_depth > 0);
    debug_assert!(limits.max_nodes > 0);
    let mut inspection = PredicateScan::new(limits.max_nodes);
    inspect_predicate(predicate, definitions, limits, &mut inspection);
    sort_diagnostics(&mut inspection.diagnostics);
    inspection.diagnostics
}

fn inspect_predicate(
    predicate: &super::ActionContextPredicate,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    limits: super::ActionContextPredicateLimits,
    inspection: &mut PredicateScan,
) {
    let count = usize::try_from(limits.max_nodes).map_or(0, |value| value);
    let mut frames = InspectStack::new(count);
    let has_root_frame = frames.push(InspectFrame { predicate, depth: 1 });
    debug_assert!(has_root_frame);
    while let Some(frame) = frames.pop() {
        inspection.node_count = inspection.node_count.saturating_add(1);
        if frame.depth > limits.max_depth {
            inspection.push(depth_diagnostic());
            continue;
        }
        if inspection.node_count > limits.max_nodes {
            inspection.push(node_limit_diagnostic());
            break;
        }
        inspect_node(frame, definitions, limits, &mut frames, inspection);
    }
    debug_assert!(inspection.node_count <= limits.max_nodes.saturating_add(1));
    debug_assert!(frames.capacity() >= count);
}

fn inspect_node<'a>(
    frame: InspectFrame<'a>,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    limits: super::ActionContextPredicateLimits,
    frames: &mut InspectStack<'a>,
    inspection: &mut PredicateScan,
) {
    match frame.predicate {
        super::ActionContextPredicate::All(children) | super::ActionContextPredicate::Any(children) => {
            if children.is_empty() {
                inspection.push(malformed_children_diagnostic());
            }
            for child in children.iter().rev() {
                if !frames.push(InspectFrame {
                    predicate: child,
                    depth: frame.depth.saturating_add(1),
                }) {
                    inspection.push(node_limit_diagnostic());
                    break;
                }
            }
        }
        super::ActionContextPredicate::Not(child) => {
            if !frames.push(InspectFrame {
                predicate: child,
                depth: frame.depth.saturating_add(1),
            }) {
                inspection.push(node_limit_diagnostic());
            }
        }
        super::ActionContextPredicate::FactPresent(fact_id) => {
            validate_fact_reference(fact_id, None, definitions, inspection);
        }
        super::ActionContextPredicate::FactEquals { fact_id, value } => {
            validate_fact_reference(fact_id, Some(value), definitions, inspection);
        }
        super::ActionContextPredicate::ContextKind(super::super::model::ActionContextKind::Custom(kind))
            if kind.trim().is_empty() || kind.trim() != kind =>
        {
            inspection.push(super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedPredicate,
                "predicate.context_kind",
                "custom context kind must be non-empty and normalized",
            ));
        }
        super::ActionContextPredicate::Always
        | super::ActionContextPredicate::ContextKind(_)
        | super::ActionContextPredicate::Focused
        | super::ActionContextPredicate::Ancestor
        | super::ActionContextPredicate::ActionState(_) => {}
    }
    debug_assert!(frames.len() <= frames.capacity());
    debug_assert!(limits.valid());
}

fn validate_fact_reference(
    fact_id: &super::super::ActionContextFactId,
    expected: Option<&super::super::model::facts::ActionContextFactValue>,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    inspection: &mut PredicateScan,
) {
    let Some(definition) = definitions.get(fact_id) else {
        inspection.push(super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::UnknownFact,
            "predicate.fact_id",
            format!("predicate references unknown fact `{fact_id}`"),
        ));
        return;
    };
    if expected.is_some_and(|value| value.kind() != definition.value_kind) {
        inspection.push(super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::FactTypeMismatch,
            "predicate.fact_value",
            format!("predicate value kind does not match fact `{fact_id}`"),
        ));
    }
}

fn depth_diagnostic() -> super::super::model::RoutingDiagnostic {
    super::super::model::RoutingDiagnostic::new(
        super::super::model::RoutingDiagnosticCode::PredicateDepthExceeded,
        "predicate.depth",
        "context predicate exceeds the accepted depth",
    )
}

fn node_limit_diagnostic() -> super::super::model::RoutingDiagnostic {
    super::super::model::RoutingDiagnostic::new(
        super::super::model::RoutingDiagnosticCode::PredicateNodeLimitExceeded,
        "predicate.nodes",
        "context predicate exceeds the accepted node count",
    )
}

fn malformed_children_diagnostic() -> super::super::model::RoutingDiagnostic {
    super::super::model::RoutingDiagnostic::new(
        super::super::model::RoutingDiagnosticCode::MalformedPredicate,
        "predicate.children",
        "logical context predicates require at least one child",
    )
}

pub(in crate::nexus_core::contextual) fn sort_diagnostics(diagnostics: &mut [super::super::model::RoutingDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.action_id.cmp(&right.action_id))
            .then_with(|| left.context_ids.cmp(&right.context_ids))
            .then_with(|| left.registration_ids.cmp(&right.registration_ids))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.message.cmp(&right.message))
    });
}
