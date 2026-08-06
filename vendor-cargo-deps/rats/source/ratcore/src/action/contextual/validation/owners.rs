pub(super) fn collect_contexts(
    contexts: &[super::super::model::ActionContext],
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    diagnostics: &mut super::DiagnosticBag,
) -> std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext> {
    let mut accepted = std::collections::BTreeMap::new();
    debug_assert!(accepted.is_empty());
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    for context in contexts {
        validate_context(context, definitions, diagnostics);
        if accepted.insert(context.id.clone(), context.clone()).is_some() {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::DuplicateContextId,
                    "context.id",
                    format!("duplicate action context `{}`", context.id),
                )
                .with_context(context.id.clone()),
            );
        }
    }
    accepted
}

pub(super) fn validate_context(
    context: &super::super::model::ActionContext,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(definitions.keys().all(|id| !super::malformed_id(id.as_str())));
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    if super::malformed_id(context.id.as_str()) {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                "context.id",
                "context identity must be non-empty and normalized",
            )
            .with_context(context.id.clone()),
        );
    }
    if context.generation == 0 {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleContextGeneration,
                "context.generation",
                "context generation must be nonzero",
            )
            .with_context(context.id.clone()),
        );
    }
    if !context.enabled && context.disabled_reason.as_deref().is_none_or(|reason| reason.trim().is_empty()) {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                "context.disabled_reason",
                "disabled context requires a reason",
            )
            .with_context(context.id.clone()),
        );
    }
    validate_context_kind_and_modal(context, diagnostics);
    validate_context_facts(context, definitions, diagnostics);
}

pub(super) fn validate_context_kind_and_modal(
    context: &super::super::model::ActionContext,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    if matches!(&context.kind, super::super::model::ActionContextKind::Custom(kind) if super::malformed_id(kind)) {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                "context.kind",
                "custom context kind must be non-empty and normalized",
            )
            .with_context(context.id.clone()),
        );
    }
    let is_modal_kind = context.kind == super::super::model::ActionContextKind::Modal;
    if is_modal_kind != context.modal.is_some() {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::ContradictoryModalRoot,
                "context.modal",
                "modal kind and modal boundary metadata must be declared together",
            )
            .with_context(context.id.clone()),
        );
    }
}

pub(super) fn validate_context_facts(
    context: &super::super::model::ActionContext,
    definitions: &std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(definitions.keys().all(|id| !super::malformed_id(id.as_str())));
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    if context.facts.len() > super::MAX_FACTS_PER_CONTEXT {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedFact,
                "context.facts",
                "context exceeds the bounded fact count",
            )
            .with_context(context.id.clone()),
        );
    }
    for (fact_id, value) in &context.facts {
        if super::malformed_id(fact_id.as_str()) || super::malformed_fact_value(value) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::MalformedFact,
                    "context.fact",
                    format!("context contains malformed fact `{fact_id}`"),
                )
                .with_context(context.id.clone()),
            );
            continue;
        }
        let Some(definition) = definitions.get(fact_id) else {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::UnknownFact,
                    "context.fact",
                    format!("context contains unknown fact `{fact_id}`"),
                )
                .with_context(context.id.clone()),
            );
            continue;
        };
        if value.kind() != definition.value_kind {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::FactTypeMismatch,
                    "context.fact",
                    format!("context fact `{fact_id}` has the wrong value kind"),
                )
                .with_context(context.id.clone()),
            );
        }
    }
}
