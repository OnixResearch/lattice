struct RouteScanner<'a> {
    snapshot: &'a super::super::model::ActionRoutingSnapshot,
    action_id: &'a crate::nexus_core::ActionId,
    chain: Vec<super::super::ActionContextId>,
    evaluation_snapshot: Option<super::super::model::ActionRoutingSnapshot>,
    modal_distance: Option<usize>,
    considerations: Vec<super::super::model::ActionRouteConsideration>,
    rejected_registrations: Vec<super::super::ActionHandlerRegistrationId>,
    evidence: super::selection::RejectionEvidence,
}

impl<'a> RouteScanner<'a> {
    fn new(
        snapshot: &'a super::super::model::ActionRoutingSnapshot,
        action_id: &'a crate::nexus_core::ActionId,
        chain: Vec<super::super::ActionContextId>,
    ) -> Self {
        debug_assert!(snapshot.generation > 0);
        debug_assert!(chain.len() <= snapshot.contexts.len());
        let modal_distance = chain
            .iter()
            .position(|context_id| snapshot.contexts.get(context_id).is_some_and(|context| context.modal.is_some()));
        let evaluation_snapshot = (chain != snapshot.active_chain).then(|| {
            let mut supplied = snapshot.clone();
            supplied.active_chain.clone_from(&chain);
            supplied
        });
        Self {
            snapshot,
            action_id,
            chain,
            evaluation_snapshot,
            modal_distance,
            considerations: Vec::with_capacity(snapshot.registrations.len()),
            rejected_registrations: Vec::with_capacity(snapshot.registrations.len()),
            evidence: super::selection::RejectionEvidence::default(),
        }
    }

    fn resolve(mut self) -> super::super::model::ActionRouteProbe {
        debug_assert!(!self.chain.is_empty());
        debug_assert!(self.chain.len() <= self.snapshot.contexts.len());
        for distance in 0..self.chain.len() {
            let context_id = self.chain[distance].clone();
            if let Some(probe) = self.visit_context(distance, &context_id) {
                return probe;
            }
        }
        if let Some(root_context) = self.chain.last().cloned() {
            return self.reject(&root_context);
        }
        let diagnostic = super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
            "request.active_chain",
            "active context chain must not be empty",
        )
        .with_action(self.action_id.clone());
        super::selection::rejected_probe(
            self.snapshot,
            self.action_id.clone(),
            self.chain,
            diagnostic,
            self.considerations,
        )
    }

    fn visit_context(
        &mut self,
        distance: usize,
        context_id: &super::super::ActionContextId,
    ) -> Option<super::super::model::ActionRouteProbe> {
        debug_assert!(distance < self.chain.len());
        debug_assert_eq!(self.chain.get(distance), Some(context_id));
        let context = self.snapshot.contexts.get(context_id)?;
        let mut registrations = self
            .snapshot
            .registrations
            .iter()
            .filter(|registration| {
                &registration.action_id == self.action_id && &registration.owner_context == context_id
            })
            .collect::<Vec<_>>();
        registrations.sort_by(|left, right| right.priority.cmp(&left.priority).then_with(|| left.id.cmp(&right.id)));
        if registrations.is_empty() {
            return None;
        }
        self.evidence.has_registration = true;
        let should_stop = registrations
            .iter()
            .any(|registration| registration.propagation == super::super::model::ActionPropagation::Stop);
        let mut eligible = Vec::with_capacity(registrations.len());
        for registration in registrations {
            let (decision, is_eligible) = self.consider_registration(context, distance, registration);
            if is_eligible {
                eligible.push(registration);
            }
            self.considerations.push(super::super::model::ActionRouteConsideration {
                registration_id: registration.id.clone(),
                owner_context: context_id.clone(),
                context_distance: distance,
                priority: registration.priority,
                propagation: registration.propagation,
                decision,
            });
        }
        if !eligible.is_empty() {
            return Some(super::selection::select_eligible(
                self.snapshot,
                self.action_id,
                self.chain.clone(),
                self.considerations.clone(),
                eligible,
            ));
        }
        should_stop.then(|| self.reject(context_id))
    }

    fn consider_registration(
        &mut self,
        context: &super::super::model::ActionContext,
        distance: usize,
        registration: &'a super::super::model::ActionHandlerRegistration,
    ) -> (super::super::model::ActionRouteDecision, bool) {
        debug_assert_eq!(&registration.action_id, self.action_id);
        debug_assert!(self.snapshot.contexts.contains_key(&registration.owner_context));
        if !context.enabled || !registration.enablement.is_enabled() {
            self.evidence.has_disabled = true;
            self.rejected_registrations.push(registration.id.clone());
            return (super::super::model::ActionRouteDecision::Disabled, false);
        }
        if super::request::modal_blocks(self.snapshot, &self.chain, self.modal_distance, distance, registration) {
            self.evidence.has_modal_block = true;
            self.rejected_registrations.push(registration.id.clone());
            return (super::super::model::ActionRouteDecision::ModalBlocked, false);
        }
        let predicate_snapshot = self.evaluation_snapshot.as_ref().unwrap_or(self.snapshot);
        let evaluation = super::super::predicates::evaluate_action_context_predicate(
            predicate_snapshot,
            &registration.predicate,
            &registration.owner_context,
            &registration.action_id,
        );
        if evaluation.matched {
            return (super::super::model::ActionRouteDecision::Eligible, true);
        }
        self.evidence.has_predicate_mismatch = true;
        self.rejected_registrations.push(registration.id.clone());
        (super::super::model::ActionRouteDecision::PredicateMismatch, false)
    }

    fn reject(&self, context_id: &super::super::ActionContextId) -> super::super::model::ActionRouteProbe {
        let diagnostic = super::selection::rejection_diagnostic(
            self.action_id,
            context_id,
            &self.rejected_registrations,
            self.evidence,
        );
        super::selection::rejected_probe(
            self.snapshot,
            self.action_id.clone(),
            self.chain.clone(),
            diagnostic,
            self.considerations.clone(),
        )
    }
}

pub(super) fn resolve(
    snapshot: &super::super::model::ActionRoutingSnapshot,
    action_id: &crate::nexus_core::ActionId,
    chain: Vec<super::super::ActionContextId>,
) -> super::super::model::ActionRouteProbe {
    RouteScanner::new(snapshot, action_id, chain).resolve()
}
