const MAX_EVALUATION_FRAMES: usize = 513;

struct FactTrace {
    contexts: Vec<super::super::ActionContextId>,
    context_limit: usize,
}

impl FactTrace {
    fn new(context_limit: usize) -> Self {
        Self {
            contexts: Vec::with_capacity(context_limit),
            context_limit,
        }
    }

    fn push(&mut self, context: super::super::ActionContextId) {
        if self.contexts.len() < self.context_limit {
            self.contexts.push(context);
        }
    }

    fn finish(mut self) -> Vec<super::super::ActionContextId> {
        self.contexts.sort();
        self.contexts.dedup();
        self.contexts
    }
}

#[derive(Clone, Copy)]
enum Frame<'a> {
    Enter(&'a super::ActionContextPredicate),
    ExitAll(usize),
    ExitAny(usize),
    ExitNot,
}

struct PredicateStack<'a> {
    frames: Vec<Frame<'a>>,
    values: Vec<bool>,
}

impl<'a> PredicateStack<'a> {
    fn new() -> Self {
        Self {
            frames: Vec::with_capacity(MAX_EVALUATION_FRAMES),
            values: Vec::with_capacity(MAX_EVALUATION_FRAMES),
        }
    }
}

struct PredicateInput<'a> {
    snapshot: &'a super::super::model::ActionRoutingSnapshot,
    owner_distance: usize,
    action_id: &'a crate::nexus_core::ActionId,
}

/// Evaluates a predicate for one owner on the accepted active chain.
///
/// Malformed predicates fail closed with diagnostics. No callback, renderer,
/// clock, or application behavior can be represented by this vocabulary.
#[must_use]
pub fn evaluate_action_context_predicate(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    predicate: &super::ActionContextPredicate,
    owner_context: &super::super::ActionContextId,
    action_id: &crate::nexus_core::ActionId,
) -> super::ActionContextPredicateEvaluation {
    debug_assert!(snapshot.active_chain.len() <= snapshot.contexts.len());
    debug_assert!(snapshot.predicate_limits.valid());
    let mut diagnostics =
        super::inspection::validate_predicate(predicate, &snapshot.fact_definitions, snapshot.predicate_limits);
    let Some(owner_distance) = snapshot.active_chain.iter().position(|context_id| context_id == owner_context) else {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::UnknownOwner,
                "predicate.owner_context",
                "predicate owner is absent from the active chain",
            )
            .with_context(owner_context.clone()),
        );
        return rejected_result(diagnostics);
    };
    if !diagnostics.is_empty() {
        super::inspection::sort_diagnostics(&mut diagnostics);
        return rejected_result(diagnostics);
    }
    let mut trace = FactTrace::new(snapshot.active_chain.len());
    let is_matched = evaluate_valid_predicate(snapshot, predicate, owner_distance, action_id, &mut trace);
    super::ActionContextPredicateEvaluation {
        matched: is_matched,
        diagnostics,
        inspected_fact_contexts: trace.finish(),
    }
}

fn evaluate_valid_predicate(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    predicate: &super::ActionContextPredicate,
    owner_distance: usize,
    action_id: &crate::nexus_core::ActionId,
    trace: &mut FactTrace,
) -> bool {
    let input = PredicateInput {
        snapshot,
        owner_distance,
        action_id,
    };
    let mut stack = PredicateStack::new();
    stack.frames.push(Frame::Enter(predicate));
    while let Some(frame) = stack.frames.pop() {
        match frame {
            Frame::Enter(current) => enter_predicate(current, &mut stack, &input, trace),
            Frame::ExitAll(count) => reduce_values(&mut stack, count, true),
            Frame::ExitAny(count) => reduce_values(&mut stack, count, false),
            Frame::ExitNot => {
                let is_negated = stack.values.pop().is_some_and(|value| !value);
                stack.values.push(is_negated);
            }
        }
    }
    debug_assert!(stack.frames.capacity() >= MAX_EVALUATION_FRAMES);
    debug_assert!(stack.values.len() <= 1);
    stack.values.pop().unwrap_or(false)
}

fn enter_predicate<'a>(
    predicate: &'a super::ActionContextPredicate,
    stack: &mut PredicateStack<'a>,
    input: &PredicateInput<'_>,
    trace: &mut FactTrace,
) {
    match predicate {
        super::ActionContextPredicate::All(children) => {
            push_children(stack, children, Frame::ExitAll(child_count(children)))
        }
        super::ActionContextPredicate::Any(children) => {
            push_children(stack, children, Frame::ExitAny(child_count(children)))
        }
        super::ActionContextPredicate::Not(child) => {
            stack.frames.push(Frame::ExitNot);
            stack.frames.push(Frame::Enter(child));
        }
        _ => stack.values.push(evaluate_atom(input, predicate, trace)),
    }
}

fn child_count(children: &[super::ActionContextPredicate]) -> usize {
    children.len()
}

fn push_children<'a>(stack: &mut PredicateStack<'a>, children: &'a [super::ActionContextPredicate], exit: Frame<'a>) {
    stack.frames.push(exit);
    for child in children.iter().rev() {
        if stack.frames.len() < MAX_EVALUATION_FRAMES {
            stack.frames.push(Frame::Enter(child));
        }
    }
}

fn evaluate_atom(input: &PredicateInput<'_>, predicate: &super::ActionContextPredicate, trace: &mut FactTrace) -> bool {
    let snapshot = input.snapshot;
    let owner_distance = input.owner_distance;
    let action_id = input.action_id;
    debug_assert!(owner_distance < snapshot.active_chain.len());
    debug_assert!(snapshot.predicate_limits.valid());
    match predicate {
        super::ActionContextPredicate::Always => true,
        super::ActionContextPredicate::FactPresent(fact_id) => {
            fact_value(snapshot, owner_distance, fact_id, trace).is_some()
        }
        super::ActionContextPredicate::FactEquals { fact_id, value } => {
            fact_value(snapshot, owner_distance, fact_id, trace) == Some(value)
        }
        super::ActionContextPredicate::ContextKind(kind) => snapshot.active_chain[..=owner_distance]
            .iter()
            .filter_map(|context_id| snapshot.contexts.get(context_id))
            .any(|context| &context.kind == kind),
        super::ActionContextPredicate::Focused => owner_distance == 0,
        super::ActionContextPredicate::Ancestor => owner_distance > 0,
        super::ActionContextPredicate::ActionState(state) => {
            snapshot.registry.action(action_id).is_some_and(|action| {
                let is_action_active = snapshot.registry.is_active(action);
                matches!(
                    (state, is_action_active),
                    (super::ActionStatePredicate::Enabled, true) | (super::ActionStatePredicate::Disabled, false)
                )
            })
        }
        super::ActionContextPredicate::All(_)
        | super::ActionContextPredicate::Any(_)
        | super::ActionContextPredicate::Not(_) => false,
    }
}

fn fact_value<'a>(
    snapshot: &'a super::super::model::ActionRoutingSnapshot,
    owner_distance: usize,
    fact_id: &super::super::ActionContextFactId,
    trace: &mut FactTrace,
) -> Option<&'a super::super::model::facts::ActionContextFactValue> {
    snapshot.active_chain[..=owner_distance].iter().find_map(|context_id| {
        let value = snapshot.contexts.get(context_id)?.facts.get(fact_id)?;
        trace.push(context_id.clone());
        Some(value)
    })
}

fn reduce_values(stack: &mut PredicateStack<'_>, count: usize, is_all: bool) {
    let mut is_result = is_all;
    for _ in 0..count {
        let is_value = stack.values.pop().unwrap_or(false);
        is_result = if is_all {
            is_result && is_value
        } else {
            is_result || is_value
        };
    }
    stack.values.push(is_result);
}

fn rejected_result(
    diagnostics: Vec<super::super::model::RoutingDiagnostic>,
) -> super::ActionContextPredicateEvaluation {
    super::ActionContextPredicateEvaluation {
        matched: false,
        diagnostics,
        inspected_fact_contexts: Vec::new(),
    }
}
