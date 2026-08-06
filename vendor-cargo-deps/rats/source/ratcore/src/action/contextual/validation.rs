const MAX_FACTS_PER_CONTEXT: usize = 32;
const MAX_FACT_TEXT_BYTES: usize = 256;
const MAX_ROUTING_DIAGNOSTICS: usize = 4_096;

struct DiagnosticBag {
    items: Vec<super::model::RoutingDiagnostic>,
}

impl DiagnosticBag {
    fn new() -> Self {
        Self {
            items: Vec::with_capacity(MAX_ROUTING_DIAGNOSTICS),
        }
    }

    fn push(&mut self, diagnostic: super::model::RoutingDiagnostic) {
        debug_assert!(self.items.len() <= MAX_ROUTING_DIAGNOSTICS);
        debug_assert!(self.items.capacity() >= MAX_ROUTING_DIAGNOSTICS);
        if self.items.len() < MAX_ROUTING_DIAGNOSTICS {
            self.items.push(diagnostic);
        }
    }

    fn extend(&mut self, diagnostics: impl IntoIterator<Item = super::model::RoutingDiagnostic>) {
        for diagnostic in diagnostics.into_iter().take(MAX_ROUTING_DIAGNOSTICS.saturating_sub(self.items.len())) {
            self.items.push(diagnostic);
        }
        debug_assert!(self.items.len() <= MAX_ROUTING_DIAGNOSTICS);
        debug_assert!(self.items.capacity() >= self.items.len());
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn sort(&mut self) {
        super::predicates::inspection::sort_diagnostics(&mut self.items);
    }

    fn into_vec(self) -> Vec<super::model::RoutingDiagnostic> {
        self.items
    }
}

/// Atomically validates and accepts a contextual routing candidate.
///
/// Invalid candidates preserve a clone of `previous`; they never expose a
/// partially accepted context graph or registration catalog.
#[must_use]
pub fn accept_action_snapshot(
    candidate: super::model::ActionRoutingCandidate,
    registry: &crate::nexus_core::ActionRegistrySnapshot,
    focus_graph: Option<&crate::focus::FocusGraph>,
    previous: Option<&super::model::ActionRoutingSnapshot>,
) -> super::model::ActionRoutingBuild {
    let mut diagnostics = DiagnosticBag::new();
    debug_assert!(diagnostics.items.is_empty());
    debug_assert!(diagnostics.items.capacity() >= MAX_ROUTING_DIAGNOSTICS);
    validate_generation(&candidate, previous, &mut diagnostics);
    let definitions = collect_definitions(&candidate.fact_definitions, &mut diagnostics);
    let contexts = owners::collect_contexts(&candidate.contexts, &definitions, &mut diagnostics);
    graph::validate_hierarchy(&contexts, &candidate, &mut diagnostics);
    let active_chain = graph::derive_active_chain(&contexts, &candidate.focused_context);
    graph::validate_active_chain(&contexts, &active_chain, &candidate, &mut diagnostics);
    focus::validate_links(&contexts, &active_chain, focus_graph, &mut diagnostics);
    let registration_inputs =
        registrations::RegistrationInputs::new(&contexts, &definitions, registry, candidate.predicate_limits);
    let mut registrations =
        registrations::collect_handlers(&candidate.registrations, &registration_inputs, &mut diagnostics);
    diagnostics.sort();
    if !diagnostics.is_empty() {
        return super::model::ActionRoutingBuild {
            snapshot: previous.cloned(),
            diagnostics: diagnostics.into_vec(),
            candidate_accepted: false,
        };
    }
    registrations.sort_by(|left, right| {
        left.action_id
            .cmp(&right.action_id)
            .then_with(|| left.owner_context.cmp(&right.owner_context))
            .then_with(|| right.priority.cmp(&left.priority))
            .then_with(|| left.id.cmp(&right.id))
    });
    super::model::ActionRoutingBuild {
        snapshot: Some(super::model::ActionRoutingSnapshot {
            generation: candidate.generation,
            contexts,
            registrations,
            fact_definitions: definitions,
            active_chain,
            registry: registry.clone(),
            predicate_limits: candidate.predicate_limits,
        }),
        diagnostics: diagnostics.into_vec(),
        candidate_accepted: true,
    }
}

fn validate_generation(
    candidate: &super::model::ActionRoutingCandidate,
    previous: Option<&super::model::ActionRoutingSnapshot>,
    diagnostics: &mut DiagnosticBag,
) {
    if candidate.generation == 0 || previous.is_some_and(|snapshot| candidate.generation <= snapshot.generation) {
        diagnostics.push(super::model::RoutingDiagnostic::new(
            super::model::RoutingDiagnosticCode::NonMonotonicGeneration,
            "routing.generation",
            "routing generation must be nonzero and newer than the accepted snapshot",
        ));
    }
    if !candidate.predicate_limits.valid() {
        diagnostics.push(super::model::RoutingDiagnostic::new(
            super::model::RoutingDiagnosticCode::MalformedPredicate,
            "predicate.limits",
            "predicate limits must be nonzero and within hard safety bounds",
        ));
    }
}

fn collect_definitions(
    definitions: &[super::model::facts::ActionContextFactDefinition],
    diagnostics: &mut DiagnosticBag,
) -> std::collections::BTreeMap<super::ActionContextFactId, super::model::facts::ActionContextFactDefinition> {
    let mut accepted = std::collections::BTreeMap::new();
    debug_assert!(accepted.is_empty());
    debug_assert!(diagnostics.items.len() <= MAX_ROUTING_DIAGNOSTICS);
    for definition in definitions {
        if malformed_id(definition.id.as_str()) {
            diagnostics.push(super::model::RoutingDiagnostic::new(
                super::model::RoutingDiagnosticCode::MalformedIdentity,
                "fact_definition.id",
                "fact definition identity must be non-empty and normalized",
            ));
            continue;
        }
        if accepted.insert(definition.id.clone(), definition.clone()).is_some() {
            diagnostics.push(super::model::RoutingDiagnostic::new(
                super::model::RoutingDiagnosticCode::DuplicateFactDefinition,
                "fact_definition.id",
                format!("duplicate fact definition `{}`", definition.id),
            ));
        }
    }
    accepted
}

#[path = "validation/focus.rs"]
mod focus;
#[path = "validation/graph.rs"]
mod graph;
#[path = "validation/owners.rs"]
mod owners;
#[path = "validation/registrations.rs"]
mod registrations;

pub(super) fn malformed_id(value: &str) -> bool {
    value.trim().is_empty() || value.trim() != value
}

pub(super) fn malformed_fact_value(value: &super::model::facts::ActionContextFactValue) -> bool {
    matches!(value, super::model::facts::ActionContextFactValue::Text(text) if text.len() > MAX_FACT_TEXT_BYTES)
}
