//! Exact action binding to the accepted product authority boundary.
//!
//! r[impl lattice.role_protocol.authorization]
//! r[impl lattice.role_protocol.external_roles]
//! r[impl lattice.role_protocol.boundary]
#![allow(
    unknown_lints,
    path_segment_repetition,
    reason = "the exported authority adapter name states the external effect-authority boundary"
)]

const INTENT_DOMAIN: &[u8] = b"lattice.role-protocol.authority-intent.v1";
const TRANSITION: &str = "protocol-prepared->dispatch";
const OPERATION: &str = "lattice/protocol";
const TRANSPORT_ABILITY: &str = "protocol/transport";
const EXTERNAL_ROLE_ABILITY: &str = "protocol/external-role";
const RESOURCE_PREFIX: &str = "lattice://role-protocol/";

pub struct EffectAuthority<'a> {
    dispatch: &'a crate::engine::effect::dispatch::Shell,
    authority: crate::engine::effect::authority::AuthorityIdentity,
    scope: crate::engine::effect::authority::CollaborationScopeBinding,
}

impl<'a> EffectAuthority<'a> {
    #[must_use]
    pub const fn new(
        dispatch: &'a crate::engine::effect::dispatch::Shell,
        authority: crate::engine::effect::authority::AuthorityIdentity,
        scope: crate::engine::effect::authority::CollaborationScopeBinding,
    ) -> Self {
        Self {
            dispatch,
            authority,
            scope,
        }
    }

    pub fn binding(
        &self,
        session: &super::super::model::SessionRecord,
        envelope: &super::super::model::MessageEnvelope,
    ) -> Result<crate::engine::effect::authority::IntentBinding, super::super::model::Error> {
        let destination = session
            .plan
            .roles
            .iter()
            .find(|role| role.role == envelope.destination_role)
            .ok_or(super::super::model::Issue::MissingRole)?;
        let ability = match destination.placement.kind {
            super::super::model::PlacementKind::Native { .. } => TRANSPORT_ABILITY,
            super::super::model::PlacementKind::External { .. } => EXTERNAL_ROLE_ABILITY,
        };
        let resource = format!("{RESOURCE_PREFIX}{}", envelope.destination_instance.as_str());
        let identity_fields = [
            session.plan.cohort.global_artifact.as_str(),
            session.plan.cohort.complete_local_set.as_str(),
            session.plan.session.as_str(),
            envelope.step.as_str(),
            TRANSITION,
            OPERATION,
            resource.as_str(),
            ability,
            envelope.authority_reference.as_str(),
            envelope.identity.as_str(),
            crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF,
            self.authority.as_str(),
        ];
        let intent = crate::engine::effect::authority::blake3_reference(INTENT_DOMAIN, &identity_fields);
        Ok(crate::engine::effect::authority::IntentBinding {
            schema: crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA.to_owned(),
            machine: crate::engine::effect::authority::MachineIdentity::new(
                session.plan.cohort.global_artifact.as_str(),
            )
            .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            workflow_revision: crate::engine::effect::authority::WorkflowRevisionIdentity::new(
                session.plan.cohort.complete_local_set.as_str(),
            )
            .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            run: crate::engine::effect::authority::RunIdentity::new(session.plan.session.as_str())
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            node: crate::engine::effect::authority::NodeIdentity::new(envelope.step.as_str())
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            transition: crate::engine::effect::authority::TransitionIdentity::new(TRANSITION)
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            operation: crate::engine::effect::authority::OperationIdentity::new(OPERATION)
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            resource: crate::engine::effect::authority::ResourceIdentity::new(resource)
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            ability: crate::engine::effect::authority::AbilityIdentity::new(ability)
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            request: crate::engine::effect::authority::RequestIdentity::new(envelope.authority_reference.as_str())
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            payload: crate::engine::effect::authority::PayloadIdentity::new(envelope.identity.as_str())
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            policy: crate::engine::effect::authority::PolicyIdentity::new(
                crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF,
            )
            .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            authority: self.authority.clone(),
            intent: crate::engine::effect::authority::IntentIdentity::new(intent)
                .map_err(|_| super::super::model::Issue::IdentityMismatch)?,
            scope: self.scope.clone(),
        })
    }
}

impl super::ProtocolAuthority for EffectAuthority<'_> {
    fn authorize(
        &self,
        session: &super::super::model::SessionRecord,
        envelope: &super::super::model::MessageEnvelope,
    ) -> Result<super::super::model::Identity, super::super::model::Issue> {
        super::super::core::validate_dispatch_envelope(session, envelope)
            .map_err(|_| super::super::model::Issue::IdentityMismatch)?;
        let binding = self.binding(session, envelope).map_err(|_| super::super::model::Issue::IdentityMismatch)?;
        let authorized = self.dispatch.authorize(binding).map_err(|_| super::super::model::Issue::AuthorityDenied)?;
        self.dispatch
            .record_authorization(&authorized)
            .map_err(|_| super::super::model::Issue::PersistenceFailed)?;
        super::super::model::Identity::new(authorized.attempt.identity.as_str())
            .map_err(|_| super::super::model::Issue::IdentityMismatch)
    }
}
