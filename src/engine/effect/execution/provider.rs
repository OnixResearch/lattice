//! Product authority context providers and durable attempt adapters.

const RUN_IDENTITY_DOMAIN: &[u8] = b"lattice.choregraph-effect-run.v1";
const UNAVAILABLE_AUTHORITY_DOMAIN: &[u8] = b"lattice.choregraph-effect-authority-unavailable.v1";
const UNAVAILABLE_AUTHORITY_LABEL: &str = "current-authority-source-unavailable";

pub trait RunContextProvider: Send + Sync {
    fn load(
        &self,
        run_id: crate::engine::executor::RunId,
        store: Option<crate::store::Store>,
    ) -> Result<super::RunContext, super::Error>;
}

#[derive(Debug, Default)]
pub struct UnavailableRunContextSource;

impl RunContextProvider for UnavailableRunContextSource {
    fn load(
        &self,
        run_id: crate::engine::executor::RunId,
        store: Option<crate::store::Store>,
    ) -> Result<super::RunContext, super::Error> {
        super::RunContext::unavailable_for_store(run_id, store)
    }
}

impl super::RunContext {
    pub fn unavailable(
        run_id: crate::engine::executor::RunId,
        attempts: Box<dyn crate::engine::effect::dispatch::EffectAttemptStore>,
    ) -> Result<Self, super::Error> {
        let run_number = run_id.0.to_string();
        let run_reference =
            crate::engine::effect::authority::blake3_reference(RUN_IDENTITY_DOMAIN, &[run_number.as_str()]);
        let authority_reference = crate::engine::effect::authority::blake3_reference(UNAVAILABLE_AUTHORITY_DOMAIN, &[
            UNAVAILABLE_AUTHORITY_LABEL,
        ]);
        let dispatch = crate::engine::effect::dispatch::Shell::new(
            Box::new(UnavailableAuthorityFacts),
            Box::new(UnavailableScopeSnapshot),
            attempts,
            Box::new(NoConfiguredRoots),
        );
        Ok(Self::new(
            crate::engine::effect::authority::RunIdentity::new(run_reference)?,
            crate::engine::effect::authority::AuthorityIdentity::new(authority_reference)?,
            crate::engine::effect::authority::CollaborationScopeBinding::Unscoped,
            std::sync::Arc::new(dispatch),
        ))
    }

    pub fn unavailable_for_store(
        run_id: crate::engine::executor::RunId,
        store: Option<crate::store::Store>,
    ) -> Result<Self, super::Error> {
        let attempts: Box<dyn crate::engine::effect::dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(StoreAttempts::new(store)),
            None => Box::new(UnavailableAttempts),
        };
        Self::unavailable(run_id, attempts)
    }
}

pub struct StoreAttempts {
    store: crate::store::Store,
}

impl StoreAttempts {
    #[must_use]
    pub fn new(store: crate::store::Store) -> Self {
        Self { store }
    }
}

impl crate::engine::effect::dispatch::EffectAttemptStore for StoreAttempts {
    fn reserve(
        &self,
        binding: crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::Attempt, crate::engine::effect::dispatch::Error> {
        crate::store::EffectAttemptRepository::reserve_effect_attempt(&self.store, binding).map_err(|error| match error
        {
            crate::store::StoreError::Conflict(_) => crate::engine::effect::dispatch::Error::DuplicateAttempt,
            _ => crate::engine::effect::dispatch::Error::AttemptPersistence,
        })
    }

    fn record_outcome(
        &self,
        event: &crate::engine::effect::authority::AuthorizationOutcomeEvent,
    ) -> Result<(), crate::engine::effect::dispatch::Error> {
        crate::store::EffectAttemptRepository::save_effect_outcome(&self.store, event)
            .map_err(|_| crate::engine::effect::dispatch::Error::OutcomePersistence)
    }
}

pub struct UnavailableAttempts;

impl crate::engine::effect::dispatch::EffectAttemptStore for UnavailableAttempts {
    fn reserve(
        &self,
        _binding: crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::Attempt, crate::engine::effect::dispatch::Error> {
        Err(crate::engine::effect::dispatch::Error::AttemptPersistence)
    }

    fn record_outcome(
        &self,
        _event: &crate::engine::effect::authority::AuthorizationOutcomeEvent,
    ) -> Result<(), crate::engine::effect::dispatch::Error> {
        Err(crate::engine::effect::dispatch::Error::OutcomePersistence)
    }
}

struct UnavailableAuthorityFacts;

impl crate::engine::effect::dispatch::AuthorityFactsProvider for UnavailableAuthorityFacts {
    fn load(
        &self,
        _binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::AuthorizationFacts, crate::engine::effect::dispatch::Error> {
        Err(crate::engine::effect::dispatch::Error::AuthorityUnavailable)
    }
}

struct UnavailableScopeSnapshot;

impl crate::engine::effect::dispatch::ScopeSnapshotProvider for UnavailableScopeSnapshot {
    fn load_current(
        &self,
        _binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<Option<crate::engine::effect::authority::ScopeSnapshotFacts>, crate::engine::effect::dispatch::Error>
    {
        Err(crate::engine::effect::dispatch::Error::AuthorityUnavailable)
    }
}

struct NoConfiguredRoots;

impl crate::engine::effect::dispatch::ConfiguredRootRegistry for NoConfiguredRoots {
    fn contains_root(&self, _root_id: &str) -> bool {
        false
    }

    fn with_root(
        &self,
        _root_id: &str,
        _operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), crate::engine::effect::dispatch::Error>,
    ) -> Result<(), crate::engine::effect::dispatch::Error> {
        Err(crate::engine::effect::dispatch::Error::UnknownRoot)
    }
}
