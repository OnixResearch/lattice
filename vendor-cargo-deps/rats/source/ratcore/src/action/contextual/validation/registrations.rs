pub(super) struct RegistrationInputs<'a> {
    contexts: &'a std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    definitions: &'a std::collections::BTreeMap<
        super::super::ActionContextFactId,
        super::super::model::facts::ActionContextFactDefinition,
    >,
    registry: &'a crate::nexus_core::ActionRegistrySnapshot,
    limits: super::super::predicates::ActionContextPredicateLimits,
}

impl<'a> RegistrationInputs<'a> {
    pub(super) fn new(
        contexts: &'a std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
        definitions: &'a std::collections::BTreeMap<
            super::super::ActionContextFactId,
            super::super::model::facts::ActionContextFactDefinition,
        >,
        registry: &'a crate::nexus_core::ActionRegistrySnapshot,
        limits: super::super::predicates::ActionContextPredicateLimits,
    ) -> Self {
        Self {
            contexts,
            definitions,
            registry,
            limits,
        }
    }
}

pub(super) fn collect_handlers(
    registrations: &[super::super::model::ActionHandlerRegistration],
    inputs: &RegistrationInputs<'_>,
    diagnostics: &mut super::DiagnosticBag,
) -> Vec<super::super::model::ActionHandlerRegistration> {
    debug_assert!(inputs.limits.valid());
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    let mut seen = std::collections::BTreeSet::<super::super::ActionHandlerRegistrationId>::new();
    for registration in registrations {
        validate_registration(registration, inputs, diagnostics);
        if !seen.insert(registration.id.clone()) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::DuplicateRegistrationId,
                    "registration.id",
                    format!("duplicate handler registration `{}`", registration.id),
                )
                .with_action(registration.action_id.clone())
                .with_registration(registration.id.clone()),
            );
        }
    }
    registrations.to_vec()
}

fn validate_registration(
    registration: &super::super::model::ActionHandlerRegistration,
    inputs: &RegistrationInputs<'_>,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(inputs.limits.valid());
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    validate_identity_and_action(registration, inputs.registry, diagnostics);
    let Some(owner) = inputs.contexts.get(&registration.owner_context) else {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::UnknownOwner,
                "registration.owner_context",
                "registration references an unknown context owner",
            )
            .with_action(registration.action_id.clone())
            .with_context(registration.owner_context.clone())
            .with_registration(registration.id.clone()),
        );
        return;
    };
    validate_owner(registration, owner, diagnostics);
    diagnostics.extend(
        super::super::predicates::inspection::validate_predicate(
            &registration.predicate,
            inputs.definitions,
            inputs.limits,
        )
        .into_iter()
        .map(|diagnostic| {
            diagnostic
                .with_action(registration.action_id.clone())
                .with_context(owner.id.clone())
                .with_registration(registration.id.clone())
        }),
    );
}

fn validate_identity_and_action(
    registration: &super::super::model::ActionHandlerRegistration,
    registry: &crate::nexus_core::ActionRegistrySnapshot,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(registry.actions().iter().all(|action| !super::malformed_id(action.id.as_str())));
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    if super::malformed_id(registration.id.as_str()) {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                "registration.id",
                "registration identity must be non-empty and normalized",
            )
            .with_registration(registration.id.clone()),
        );
    }
    if registry.action(&registration.action_id).is_none() {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::UnknownAction,
                "registration.action_id",
                format!("registration references unknown action `{}`", registration.action_id),
            )
            .with_action(registration.action_id.clone())
            .with_registration(registration.id.clone()),
        );
    }
}

fn validate_owner(
    registration: &super::super::model::ActionHandlerRegistration,
    owner: &super::super::model::ActionContext,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert_eq!(owner.id, registration.owner_context);
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    if owner.generation != registration.owner_generation {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleContextGeneration,
                "registration.owner_generation",
                "registration observes a stale owner generation",
            )
            .with_action(registration.action_id.clone())
            .with_context(owner.id.clone())
            .with_registration(registration.id.clone()),
        );
    }
    if registration.propagation == super::super::model::ActionPropagation::RootOnly && owner.parent.is_some() {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::InvalidPropagation,
                "registration.propagation",
                "root-only registration must be owned by the application root",
            )
            .with_action(registration.action_id.clone())
            .with_context(owner.id.clone())
            .with_registration(registration.id.clone()),
        );
    }
    if matches!(&registration.enablement, crate::nexus_core::ActionEnablement::Disabled { reason } if reason.trim().is_empty())
    {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                "registration.disabled_reason",
                "disabled registration requires a reason",
            )
            .with_registration(registration.id.clone()),
        );
    }
}
