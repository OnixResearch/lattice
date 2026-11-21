//! Port implementations. None has an authority or promotion path.

type Prepared = lattice_core::agent_revision::ports::Prepared;
type Cleanup = lattice_core::agent_revision::ports::Cleanup;
type Verdict = lattice_core::agent_revision::ports::Verdict;

impl super::PreparationPort for super::UnavailablePreparation {
    fn prepare(&mut self, _ticket: &super::Ticket, _bundle: &super::Bundle) -> Prepared {
        Prepared::Unknown {}
    }

    fn cleanup(&mut self, _sweep: &super::Sweep) -> Cleanup {
        Cleanup::Unknown
    }
}

impl super::QualificationPort for super::UnavailableQualification {
    fn answer(&mut self, _query: &super::Query) -> Verdict {
        Verdict::Unavailable {}
    }
}

impl super::PreparationPort for super::FixturePreparation {
    fn prepare(&mut self, ticket: &super::Ticket, _bundle: &super::Bundle) -> Prepared {
        self.tickets.push(ticket.clone());
        self.prepared.pop_front().unwrap_or(Prepared::Unknown {})
    }

    fn cleanup(&mut self, sweep: &super::Sweep) -> Cleanup {
        self.sweeps.push(sweep.clone());
        self.cleanups.pop_front().unwrap_or(Cleanup::Unknown)
    }
}

impl super::QualificationPort for super::FixtureQualification {
    fn answer(&mut self, query: &super::Query) -> Verdict {
        self.queries.push(query.clone());
        let is_subject = |known: &super::Query| {
            let keys = [
                known.definition_identity == query.definition_identity,
                known.profile_revision == query.profile_revision,
                known.task_family == query.task_family,
                known.required_class == query.required_class,
            ];
            keys.iter().all(|key| *key)
        };
        self.answers.iter().find(|(known, _)| is_subject(known)).map_or(
            Verdict::NotQualified {
                reason: lattice_core::agent_revision::ports::Unqualified::SubjectMismatch,
            },
            |(_, verdict)| verdict.clone(),
        )
    }
}

impl super::FixturePreparation {
    /// Script preparation and cleanup observations, each returned in order;
    /// an exhausted script observes `unknown`.
    #[must_use]
    pub fn new(prepared: Vec<Prepared>, cleanups: Vec<Cleanup>) -> Self {
        Self {
            prepared: prepared.into(),
            cleanups: cleanups.into(),
            tickets: Vec::new(),
            sweeps: Vec::new(),
        }
    }

    /// Every ticket the port was asked to prepare, in order.
    #[must_use]
    pub fn tickets(&self) -> &[super::Ticket] {
        &self.tickets
    }

    /// Every cleanup the port was asked to perform, in order.
    #[must_use]
    pub fn sweeps(&self) -> &[super::Sweep] {
        &self.sweeps
    }
}

impl super::FixtureQualification {
    /// Answer `verdict` for exactly `query`'s subject keys, scope and class.
    #[must_use]
    pub fn new(answers: Vec<(super::Query, Verdict)>) -> Self {
        Self {
            answers,
            queries: Vec::new(),
        }
    }

    /// Every query the port received, in order.
    #[must_use]
    pub fn queries(&self) -> &[super::Query] {
        &self.queries
    }
}
