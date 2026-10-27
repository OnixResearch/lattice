#![allow(
    unknown_lints,
    compound_condition,
    nested_conditionals,
    non_trait_imports,
    path_segment_repetition,
    reason = "the closed route authority adapter checks every exact binding field and maps reviewed ability families"
)]

use crate::engine::effect::authority;
use crate::engine::effect::dispatch;
use crate::engine::effect::execution;
use crate::engine::executor::RunId;

const RUN_DOMAIN: &[u8] = b"lattice.workflow-exchange-run-authority.v1";
const AUTHORITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-route-authority.v1";
const FILE_CONTRACT: &str = "lattice-file";
const NETWORK_CONTRACT: &str = "lattice-network";
const PROCESS_CONTRACT: &str = "lattice-process";
const PROTOCOL_CONTRACT: &str = "lattice-role-protocol";
const AGENT_CONTRACT: &str = "lattice-agent";

#[derive(Debug, Clone)]
pub struct RouteExactAuthorityProvider {
    workflow_revision: String,
    run_id: RunId,
    run_reference: String,
    authority_reference: String,
}

impl RouteExactAuthorityProvider {
    pub fn new(
        handler_profile_identity: &lattice_workflow_contract::Identity,
        request_identity: &lattice_workflow_contract::Identity,
        workflow_revision: &lattice_workflow_contract::Identity,
        run_id: RunId,
    ) -> Self {
        let run_number = run_id.0.to_string();
        let run_reference = authority::blake3_reference(RUN_DOMAIN, &[
            handler_profile_identity.as_str(),
            request_identity.as_str(),
            workflow_revision.as_str(),
            run_number.as_str(),
        ]);
        let authority_reference = authority::blake3_reference(AUTHORITY_DOMAIN, &[
            handler_profile_identity.as_str(),
            request_identity.as_str(),
            workflow_revision.as_str(),
        ]);
        Self {
            workflow_revision: workflow_revision.as_str().to_string(),
            run_id,
            run_reference,
            authority_reference,
        }
    }
}

impl execution::RunContextProvider for RouteExactAuthorityProvider {
    fn load(
        &self,
        run_id: RunId,
        store: Option<crate::store::Store>,
    ) -> Result<execution::RunContext, execution::Error> {
        if run_id != self.run_id {
            return Err(execution::Error::MachineIdentity);
        }
        let attempts: Box<dyn dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(execution::StoreAttempts::new(store)),
            None => Box::new(execution::UnavailableAttempts),
        };
        let shell = dispatch::Shell::new(
            Box::new(RouteExactAuthorityFacts {
                workflow_revision: self.workflow_revision.clone(),
                run_reference: self.run_reference.clone(),
                authority_reference: self.authority_reference.clone(),
            }),
            Box::new(NoScopeSnapshot),
            attempts,
            Box::new(NoConfiguredRoots),
        );
        Ok(execution::RunContext::new(
            authority::RunIdentity::new(self.run_reference.clone())?,
            authority::AuthorityIdentity::new(self.authority_reference.clone())?,
            authority::CollaborationScopeBinding::Unscoped,
            std::sync::Arc::new(shell),
        ))
    }
}

struct RouteExactAuthorityFacts {
    workflow_revision: String,
    run_reference: String,
    authority_reference: String,
}

impl dispatch::AuthorityFactsProvider for RouteExactAuthorityFacts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        let expected_revision = self
            .workflow_revision
            .strip_prefix(lattice_workflow_contract::BLAKE3_PREFIX)
            .map(|digest| format!("blake3:{digest}"))
            .ok_or(dispatch::Error::AuthorityDenied)?;
        let is_revision_exact = binding.workflow_revision.as_str() == expected_revision;
        let is_run_exact = binding.run.as_str() == self.run_reference;
        let is_authority_exact = binding.authority.as_str() == self.authority_reference;
        let is_scope_unscoped = matches!(binding.scope, authority::CollaborationScopeBinding::Unscoped);
        if !is_revision_exact || !is_run_exact || !is_authority_exact || !is_scope_unscoped {
            return Err(dispatch::Error::AuthorityDenied);
        }
        let contract_id = contract_for_ability(binding.ability.as_str()).ok_or(dispatch::Error::AuthorityDenied)?;
        Ok(authority::AuthorizationFacts {
            binding: binding.clone(),
            evidence: authority::Evidence::LocalOperator(authority::LocalOperatorGrant {
                authority_ref: self.authority_reference.clone(),
                operation: binding.operation.as_str().to_string(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: contract_id.to_string(),
                    resource: binding.resource.as_str().to_string(),
                    ability: binding.ability.as_str().to_string(),
                    request_ref: binding.request.as_str().to_string(),
                },
            }),
            trusted_policy_ref: dispatch::trusted_policy_ref(contract_id),
            scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: authority::RevocationDisposition::Current,
        })
    }
}

fn contract_for_ability(ability: &str) -> Option<&'static str> {
    if ability.starts_with("file/") {
        Some(FILE_CONTRACT)
    } else if ability.starts_with("network/") {
        Some(NETWORK_CONTRACT)
    } else if ability.starts_with("process/") {
        Some(PROCESS_CONTRACT)
    } else if ability.starts_with("protocol/") {
        Some(PROTOCOL_CONTRACT)
    } else if ability == "agent/execute" {
        Some(AGENT_CONTRACT)
    } else {
        None
    }
}

struct NoScopeSnapshot;

impl dispatch::ScopeSnapshotProvider for NoScopeSnapshot {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
        Ok(None)
    }
}

struct NoConfiguredRoots;

impl dispatch::ConfiguredRootRegistry for NoConfiguredRoots {
    fn contains_root(&self, _root_id: &str) -> bool {
        false
    }

    fn with_root(
        &self,
        _root_id: &str,
        _operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), dispatch::Error>,
    ) -> Result<(), dispatch::Error> {
        Err(dispatch::Error::UnknownRoot)
    }
}
