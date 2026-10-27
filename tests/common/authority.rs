use std::collections::BTreeMap;
use std::path::Path;

use lattice::engine::effect::authority;
use lattice::engine::effect::dispatch;
use lattice::engine::effect::execution;
use lattice::engine::effect::runtime;
use lattice::engine::executor::RunId;
use lattice::store::Store;

const TEST_GRANT_REF: &str = "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const TEST_RECEIPT_REF: &str = "blake3:4444444444444444444444444444444444444444444444444444444444444444";
const TEST_AUTHORITY_REF: &str = "test-current-authority";
const PROCESS_CONTRACT: &str = "lattice-process";
const NETWORK_CONTRACT: &str = "lattice-network";
const FILE_CONTRACT: &str = "lattice-file";
const PROCESS_OPERATION: &str = "lattice/shell";
const NETWORK_OPERATION: &str = "lattice/http";
const FILE_OPERATION: &str = "lattice/file";

#[derive(Debug, Clone, Default)]
pub struct TestRunContextProvider {
    roots: BTreeMap<String, std::path::PathBuf>,
}

impl TestRunContextProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root(mut self, root_id: impl Into<String>, path: impl AsRef<Path>) -> Self {
        let prior = self.roots.insert(root_id.into(), path.as_ref().to_path_buf());
        assert!(prior.is_none(), "test root identifiers must be unique");
        self
    }
}

impl execution::RunContextProvider for TestRunContextProvider {
    fn load(&self, run_id: RunId, store: Option<Store>) -> Result<execution::RunContext, execution::Error> {
        let attempts: Box<dyn dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(execution::StoreAttempts::new(store)),
            None => Box::new(execution::UnavailableAttempts),
        };
        let mut roots = BTreeMap::new();
        for (root_id, path) in &self.roots {
            let directory = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())
                .expect("configured test root opens");
            let prior = roots.insert(root_id.clone(), directory);
            assert!(prior.is_none(), "test root identifiers must remain unique");
        }
        let shell = dispatch::Shell::new(
            Box::new(CurrentAuthorityFacts),
            Box::new(NoScope),
            attempts,
            Box::new(runtime::ConfiguredRoots::new(roots)),
        );
        Ok(execution::RunContext::new(
            authority::RunIdentity::new(format!("test-run/{}", run_id.0))?,
            authority::AuthorityIdentity::new(TEST_AUTHORITY_REF)?,
            authority::CollaborationScopeBinding::Unscoped,
            std::sync::Arc::new(shell),
        ))
    }
}

struct CurrentAuthorityFacts;

impl dispatch::AuthorityFactsProvider for CurrentAuthorityFacts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        let contract_id = match binding.operation.as_str() {
            PROCESS_OPERATION => PROCESS_CONTRACT,
            NETWORK_OPERATION => NETWORK_CONTRACT,
            FILE_OPERATION => FILE_CONTRACT,
            "lattice/agent" => "lattice-agent",
            _ => return Err(dispatch::Error::AuthorityDenied),
        };
        let resource = binding.resource.as_str().to_owned();
        let ability = binding.ability.as_str().to_owned();
        let request_ref = binding.request.as_str().to_owned();
        Ok(authority::AuthorizationFacts {
            binding: binding.clone(),
            evidence: authority::Evidence::VerifiedUcan(basalt::authority_core::AuthorityInput {
                schema: basalt::authority_core::AUTHORITY_INPUT_SCHEMA.to_owned(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: contract_id.to_owned(),
                    resource: resource.clone(),
                    ability: ability.clone(),
                    request_ref: request_ref.clone(),
                },
                verified_grants: vec![basalt::authority_core::VerifiedGrantRef {
                    grant_ref: TEST_GRANT_REF.to_owned(),
                    verification_receipt_ref: TEST_RECEIPT_REF.to_owned(),
                    resource,
                    ability,
                    kind: basalt::authority_core::VerifiedGrantKind::Direct,
                }],
                verification_receipts: vec![basalt::authority_core::UcanVerificationReceiptRef {
                    receipt_ref: TEST_RECEIPT_REF.to_owned(),
                    request_ref,
                    verified_grant_refs: vec![TEST_GRANT_REF.to_owned()],
                    disposition: basalt::authority_core::VerificationRefDisposition::Current,
                }],
                policy_refs: vec![dispatch::trusted_policy_ref(contract_id)],
                caveat: basalt::authority_core::AuthorityCaveatDisposition::Satisfied,
                replay: basalt::authority_core::AuthorityReplayDisposition::Admitted,
                claims: vec![basalt::authority_core::AuthorityClaim::ExactRequest],
                non_claims: basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
            }),
            trusted_policy_ref: dispatch::trusted_policy_ref(contract_id),
            scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: authority::RevocationDisposition::Current,
        })
    }
}

struct NoScope;

impl dispatch::ScopeSnapshotProvider for NoScope {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
        Ok(None)
    }
}
