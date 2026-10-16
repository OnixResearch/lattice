//! Host-owned profile evaluation, currentness, clock, roots, and durable attempts.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

const RUN_IDENTITY_DOMAIN: &[u8] = b"lattice.standalone-effect-authority.run.v1";

#[derive(Debug, Clone)]
pub struct OperatorRunContextSource {
    profile_path: PathBuf,
}

impl OperatorRunContextSource {
    #[must_use]
    pub fn new(profile_path: PathBuf) -> Self {
        Self { profile_path }
    }
}

// r[impl lattice.standalone_effect_authority.currentness]
// r[impl lattice.standalone_effect_authority.roots]
impl crate::engine::effect::execution::RunContextProvider for OperatorRunContextSource {
    fn load(
        &self,
        run_id: crate::engine::executor::RunId,
        store: Option<crate::store::Store>,
    ) -> Result<crate::engine::effect::execution::RunContext, crate::engine::effect::execution::Error> {
        let (admitted, _) =
            load_profile(&self.profile_path).map_err(|_| crate::engine::effect::execution::Error::AuthoritySource)?;
        let mut roots = BTreeMap::new();
        for (id, path) in admitted.profile.roots {
            let root = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())
                .map_err(|_| crate::engine::effect::execution::Error::AuthoritySource)?;
            roots.insert(id, root);
        }
        let run_number = run_id.0.to_string();
        let run_reference = crate::engine::effect::authority::blake3_reference(RUN_IDENTITY_DOMAIN, &[
            run_number.as_str(),
            admitted.identity.as_str(),
        ]);
        let attempts: Box<dyn crate::engine::effect::dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(crate::engine::effect::execution::StoreAttempts::new(store)),
            None => Box::new(crate::engine::effect::execution::UnavailableAttempts),
        };
        let shell = crate::engine::effect::dispatch::Shell::new(
            Box::new(CurrentOperatorFacts {
                profile_path: self.profile_path.clone(),
                profile_identity: admitted.identity.clone(),
                run_reference: run_reference.clone(),
            }),
            Box::new(UnscopedSnapshot),
            attempts,
            Box::new(crate::engine::effect::runtime::ConfiguredRoots::new(roots)),
        );
        Ok(crate::engine::effect::execution::RunContext::new(
            crate::engine::effect::authority::RunIdentity::new(run_reference)?,
            crate::engine::effect::authority::AuthorityIdentity::new(admitted.identity)?,
            crate::engine::effect::authority::CollaborationScopeBinding::Unscoped,
            Arc::new(shell),
        ))
    }
}

struct CurrentOperatorFacts {
    profile_path: PathBuf,
    profile_identity: String,
    run_reference: String,
}

// r[impl lattice.standalone_effect_authority.binding]
impl crate::engine::effect::dispatch::AuthorityFactsProvider for CurrentOperatorFacts {
    fn load(
        &self,
        binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::AuthorizationFacts, crate::engine::effect::dispatch::Error> {
        let (current, now) = load_profile(&self.profile_path)?;
        if current.identity != self.profile_identity {
            return Err(crate::engine::effect::dispatch::Error::AuthorityDenied);
        }
        let grant = super::core::check_binding(&current, binding, &self.run_reference, now)
            .map_err(|_| crate::engine::effect::dispatch::Error::AuthorityDenied)?;
        let contract = grant.contract_id().ok_or(crate::engine::effect::dispatch::Error::AuthorityDenied)?;
        Ok(crate::engine::effect::authority::AuthorizationFacts {
            binding: binding.clone(),
            evidence: crate::engine::effect::authority::Evidence::LocalOperator(
                crate::engine::effect::authority::LocalOperatorGrant {
                    authority_ref: self.profile_identity.clone(),
                    operation: grant.operation.clone(),
                    request: basalt::authority_core::AuthorityRequest {
                        contract_id: contract.to_owned(),
                        resource: grant.resource.clone(),
                        ability: grant.ability.clone(),
                        request_ref: binding.request.as_str().to_owned(),
                    },
                },
            ),
            trusted_policy_ref: crate::engine::effect::dispatch::trusted_policy_ref(contract),
            scope_requirement: crate::engine::effect::authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: crate::engine::effect::authority::RevocationDisposition::Current,
        })
    }
}

struct UnscopedSnapshot;

impl crate::engine::effect::dispatch::ScopeSnapshotProvider for UnscopedSnapshot {
    fn load_current(
        &self,
        binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<Option<crate::engine::effect::authority::ScopeSnapshotFacts>, crate::engine::effect::dispatch::Error>
    {
        match binding.scope {
            crate::engine::effect::authority::CollaborationScopeBinding::Unscoped => Ok(None),
            crate::engine::effect::authority::CollaborationScopeBinding::Protected { .. } => {
                Err(crate::engine::effect::dispatch::Error::AuthorityDenied)
            }
        }
    }
}

fn load_profile(path: &Path) -> Result<(super::core::AdmittedProfile, u64), crate::engine::effect::dispatch::Error> {
    if !path.is_absolute() {
        return Err(crate::engine::effect::dispatch::Error::AuthorityDenied);
    }
    let value =
        crate::nickel::evaluate_file(path).map_err(|_| crate::engine::effect::dispatch::Error::AuthorityUnavailable)?;
    let profile = serde_json::from_value(value).map_err(|_| crate::engine::effect::dispatch::Error::AuthorityDenied)?;
    let now = observe_unix_millis()?;
    let admitted =
        super::core::admit(profile, now).map_err(|_| crate::engine::effect::dispatch::Error::AuthorityDenied)?;
    Ok((admitted, now))
}

#[allow(
    unknown_lints,
    ambient_clock,
    reason = "the authority shell observes time explicitly; pure profile admission receives the observed value"
)]
fn observe_unix_millis() -> Result<u64, crate::engine::effect::dispatch::Error> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| crate::engine::effect::dispatch::Error::AuthorityUnavailable)?;
    u64::try_from(elapsed.as_millis()).map_err(|_| crate::engine::effect::dispatch::Error::AuthorityUnavailable)
}
