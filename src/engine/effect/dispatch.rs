//! Imperative shell for authorization and durable attempt recording.

pub trait AuthorityFactsProvider: Send + Sync {
    fn load(&self, binding: &super::authority::IntentBinding) -> Result<super::authority::AuthorizationFacts, Error>;
}

pub trait ScopeSnapshotProvider: Send + Sync {
    fn load_current(
        &self,
        binding: &super::authority::IntentBinding,
    ) -> Result<Option<super::authority::ScopeSnapshotFacts>, Error>;
}

pub trait EffectAttemptStore: Send + Sync {
    fn reserve(&self, binding: super::authority::IntentBinding) -> Result<super::authority::Attempt, Error>;
    fn record_outcome(&self, event: &super::authority::AuthorizationOutcomeEvent) -> Result<(), Error>;
}

pub struct FileTarget<'a> {
    pub root_id: &'a str,
    pub relative_path: &'a str,
}

pub trait ConfiguredRootRegistry: Send + Sync {
    fn contains_root(&self, root_id: &str) -> bool;
    fn with_root(
        &self,
        root_id: &str,
        operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), Error>,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    AuthorityUnavailable,
    AuthorityDenied,
    AttemptPersistence,
    OutcomePersistence,
    DuplicateAttempt,
    UnknownRoot,
    InvalidPath,
    AdapterRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedAttempt {
    pub attempt: super::authority::Attempt,
    pub authorization: super::authority::AuthorizationDecision,
}

pub trait AuthorizedAdapterRequest {
    type Output;

    fn execute(
        &self,
        runtime: &super::runtime::Adapters<'_>,
        authorized: &AuthorizedAttempt,
    ) -> Result<Self::Output, Error>;
}

pub struct Shell {
    authority: Box<dyn AuthorityFactsProvider>,
    scope: Box<dyn ScopeSnapshotProvider>,
    attempts: Box<dyn EffectAttemptStore>,
    roots: Box<dyn ConfiguredRootRegistry>,
}

impl Shell {
    #[must_use]
    pub fn new(
        authority: Box<dyn AuthorityFactsProvider>,
        scope: Box<dyn ScopeSnapshotProvider>,
        attempts: Box<dyn EffectAttemptStore>,
        roots: Box<dyn ConfiguredRootRegistry>,
    ) -> Self {
        Self {
            authority,
            scope,
            attempts,
            roots,
        }
    }

    // r[impl lattice.choregraph_effect_authority.durability]
    // r[impl lattice.choregraph_effect_authority.denial]
    pub fn authorize(&self, binding: super::authority::IntentBinding) -> Result<AuthorizedAttempt, Error> {
        let attempt = self.attempts.reserve(binding.clone())?;
        let mut facts = match self.authority.load(&binding) {
            Ok(facts) => facts,
            Err(_) => return self.persist_unavailable(&attempt),
        };
        facts.scope_snapshot = match self.scope.load_current(&binding) {
            Ok(snapshot) => snapshot,
            Err(_) => return self.persist_unavailable(&attempt),
        };
        let authorization = super::authority::evaluate_for(&binding, &facts);
        if !authorization.allowed {
            return self.persist_decision(&attempt, &authorization, Error::AuthorityDenied);
        }
        Ok(AuthorizedAttempt { attempt, authorization })
    }

    fn persist_unavailable<T>(&self, attempt: &super::authority::Attempt) -> Result<T, Error> {
        let decision = super::authority::AuthorizationDecision {
            allowed: false,
            binding: None,
            denial: Some(super::authority::AuthorizationDenial::AuthorityUnavailable),
            basalt_diagnostic: None,
        };
        self.persist_decision(attempt, &decision, Error::AuthorityUnavailable)
    }

    fn persist_decision<T>(
        &self,
        attempt: &super::authority::Attempt,
        decision: &super::authority::AuthorizationDecision,
        error: Error,
    ) -> Result<T, Error> {
        let outcome = super::authority::outcome_event(attempt, decision);
        self.attempts.record_outcome(&outcome).map_err(|_| Error::AttemptPersistence)?;
        Err(error)
    }

    pub fn record_authorization(
        &self,
        authorized: &AuthorizedAttempt,
    ) -> Result<super::authority::AuthorizationOutcomeEvent, Error> {
        let outcome = super::authority::outcome_event(&authorized.attempt, &authorized.authorization);
        self.attempts.record_outcome(&outcome).map_err(|_| Error::OutcomePersistence)?;
        Ok(outcome)
    }

    // r[impl lattice.choregraph_effect_authority.files]
    pub fn admit_file_path(
        &self,
        authorized: &AuthorizedAttempt,
        target: FileTarget<'_>,
    ) -> Result<cap_root_core::RootRelativePath, Error> {
        if !authorized.authorization.allowed {
            return Err(Error::AuthorityDenied);
        }
        if !self.roots.contains_root(target.root_id) {
            return Err(Error::UnknownRoot);
        }
        cap_root_core::RootRelativePath::new(target.relative_path, cap_root_core::AdmissionMode::NonEmptyRelative)
            .map_err(|_| Error::InvalidPath)
    }

    pub fn with_file_root(
        &self,
        authorized: &AuthorizedAttempt,
        root_id: &str,
        operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if !authorized.authorization.allowed {
            return Err(Error::AuthorityDenied);
        }
        self.roots.with_root(root_id, operation)
    }

    pub fn record_outcome(
        &self,
        authorized: &AuthorizedAttempt,
        adapter_succeeded: bool,
    ) -> Result<super::authority::AuthorizationOutcomeEvent, Error> {
        let (state, denial) = if adapter_succeeded {
            (super::authority::OutcomeState::ObservedSuccess, None)
        } else {
            (
                super::authority::OutcomeState::ObservedFailure,
                Some(super::authority::AuthorizationDenial::AdapterRejected),
            )
        };
        let outcome = super::authority::observed_outcome_event(&authorized.attempt, state, denial);
        self.attempts.record_outcome(&outcome).map_err(|_| Error::OutcomePersistence)?;
        Ok(outcome)
    }

    pub fn record_unknown_after_dispatch(
        &self,
        authorized: &AuthorizedAttempt,
    ) -> Result<super::authority::AuthorizationOutcomeEvent, Error> {
        let outcome = super::authority::observed_outcome_event(
            &authorized.attempt,
            super::authority::OutcomeState::UnknownAfterDispatch,
            None,
        );
        self.attempts.record_outcome(&outcome).map_err(|_| Error::OutcomePersistence)?;
        Ok(outcome)
    }

    // r[impl lattice.choregraph_effect_authority.adapters]
    pub fn dispatch<R>(
        &self,
        runtime: &super::runtime::Adapters<'_>,
        binding: super::authority::IntentBinding,
        request: &R,
    ) -> Result<R::Output, Error>
    where
        R: AuthorizedAdapterRequest,
    {
        let authorized = self.authorize(binding)?;
        let result = request.execute(runtime, &authorized);
        match result {
            Ok(output) => {
                self.persist_observed_or_unknown(&authorized, true)?;
                Ok(output)
            }
            Err(error) => {
                self.persist_observed_or_unknown(&authorized, false)?;
                Err(error)
            }
        }
    }

    fn persist_observed_or_unknown(
        &self,
        authorized: &AuthorizedAttempt,
        adapter_succeeded: bool,
    ) -> Result<(), Error> {
        if self.record_outcome(authorized, adapter_succeeded).is_ok() {
            return Ok(());
        }
        self.record_unknown_after_dispatch(authorized)?;
        Err(Error::OutcomePersistence)
    }
}

#[must_use]
pub fn trusted_policy_ref(contract_id: &str) -> basalt::authority_core::BasaltPolicyRef {
    basalt::authority_core::BasaltPolicyRef::new(
        super::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF,
        super::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_SCHEMA,
        contract_id,
    )
}
