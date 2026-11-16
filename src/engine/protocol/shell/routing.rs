//! Material-free adapter routing and exact placement/profile matching.

use super::super::model;

impl super::Adapters {
    pub(super) fn select(
        &self,
        session: &model::SessionRecord,
        envelope: &model::MessageEnvelope,
    ) -> Result<&dyn super::ProtocolAdapter, model::Issue> {
        let placement = session
            .plan
            .roles
            .iter()
            .find(|role| role.role == envelope.destination_role)
            .ok_or(model::Issue::MissingRole)?;
        let adapter = lookup(self, &placement.placement)?;
        if adapter.profile() != &expected_adapter_profile(&placement.placement) {
            return Err(model::Issue::AdapterFailed);
        }
        Ok(adapter)
    }
}

fn lookup<'a>(
    adapters: &'a super::Adapters,
    placement: &model::RolePlacement,
) -> Result<&'a dyn super::ProtocolAdapter, model::Issue> {
    match &placement.kind {
        model::PlacementKind::Native { runtime_route } => {
            adapters.native.get(runtime_route.as_str()).map(Box::as_ref).ok_or(model::Issue::AdapterFailed)
        }
        model::PlacementKind::External { contract, .. } => {
            adapters.external.get(contract.as_str()).map(Box::as_ref).ok_or(model::Issue::AdapterFailed)
        }
    }
}

fn expected_adapter_profile(placement: &model::RolePlacement) -> super::AdapterProfile {
    match &placement.kind {
        model::PlacementKind::Native { runtime_route } => super::AdapterProfile::Native {
            cohort: placement.adapter_cohort.clone(),
            runtime_route: runtime_route.clone(),
        },
        model::PlacementKind::External {
            contract,
            destination_policy,
            credential_reference,
            ..
        } => super::AdapterProfile::External {
            cohort: placement.adapter_cohort.clone(),
            contract: contract.clone(),
            destination_policy: destination_policy.clone(),
            credential_reference: credential_reference.clone(),
        },
    }
}
