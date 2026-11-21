//! Port implementations. None has an authority or promotion path.

// r[impl lattice.agent_revisions.attachments]

type Prepared = lattice_core::agent_revision::ports::Prepared;
type Cleanup = lattice_core::agent_revision::ports::Cleanup;
type Verdict = lattice_core::agent_revision::ports::Verdict;
type Class = lattice_core::agent_revision::ports::EvidenceClass;
type Unqualified = lattice_core::agent_revision::ports::Unqualified;
type Answer = animus_runtime::qualification::QualificationAnswer;
type AnimusClass = animus_runtime::qualification::EvidenceClass;
type Reason = animus_runtime::qualification::RejectionReason;
type Label = animus_runtime::qualification::EvidenceLabel;
type Provenance = lattice_core::agent_revision::ports::Provenance;

/// Largest attachment file, equal to the producer's canonical bound.
const MAX_ATTACHMENT_FILE_BYTES: usize = animus_runtime::qualification::MAX_ATTACHMENT_BYTES;
/// Most attachment files one command may name.
const MAX_ATTACHMENT_FILES: usize = 16;

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

impl super::QualificationPort for super::AttachmentQualification {
    fn answer(&mut self, query: &super::Query) -> Verdict {
        if self.attachments.is_empty() {
            return Verdict::Unavailable {};
        }
        let attachments = self.attachments.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let expected = animus_runtime::qualification::ExpectedSubject {
            lattice_definition_identity: &query.definition_identity,
            lattice_profile_revision: &query.profile_revision,
            task_family: &query.task_family,
            required_class: animus_class(query.required_class),
            now_ms: query.now_ms,
            required_components: &[],
            current_components: &[],
        };
        debug_assert!(attachments.len() <= MAX_ATTACHMENT_FILES);
        verdict(animus_runtime::qualification::qualify(&attachments, &expected), &attachments)
    }
}

impl super::AttachmentQualification {
    /// Read each operator-supplied attachment file: a regular file, not a
    /// symlink, of at most 16 KiB. One trailing newline is dropped, so the
    /// producer's checked files are accepted as written.
    ///
    /// # Errors
    ///
    /// Too many files, or a file that is missing, not regular or oversized.
    pub fn load(paths: &[std::path::PathBuf]) -> Result<Self, String> {
        if paths.len() > MAX_ATTACHMENT_FILES {
            return Err(format!("at most {MAX_ATTACHMENT_FILES} qualification attachments may be named"));
        }
        let attachments = paths.iter().map(|path| read_attachment(path)).collect::<Result<Vec<_>, _>>()?;
        debug_assert_eq!(attachments.len(), paths.len());
        debug_assert!(attachments.iter().all(|bytes| bytes.len() <= MAX_ATTACHMENT_FILE_BYTES));
        Ok(Self { attachments })
    }
}

fn read_attachment(path: &std::path::Path) -> Result<Vec<u8>, String> {
    use std::io::Read as _;
    let shown = path.display();
    let limit_bytes = u64::try_from(MAX_ATTACHMENT_FILE_BYTES).map_err(|_| "attachment bound is invalid")?;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("qualification attachment {shown} is unavailable: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("qualification attachment {shown} is not a regular file"));
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("qualification attachment {shown} is unavailable: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("qualification attachment {shown} is unavailable: {error}"))?;
    if !opened.is_file() {
        return Err(format!("qualification attachment {shown} is not a regular file"));
    }
    if opened.len() > limit_bytes {
        return Err(format!("qualification attachment {shown} exceeds {MAX_ATTACHMENT_FILE_BYTES} bytes"));
    }
    let mut bytes = Vec::with_capacity(MAX_ATTACHMENT_FILE_BYTES);
    file.take(limit_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("qualification attachment {shown} could not be read: {error}"))?;
    if bytes.len() > MAX_ATTACHMENT_FILE_BYTES {
        return Err(format!("qualification attachment {shown} exceeds {MAX_ATTACHMENT_FILE_BYTES} bytes"));
    }
    debug_assert!(opened.is_file());
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    debug_assert!(bytes.len() <= MAX_ATTACHMENT_FILE_BYTES);
    Ok(bytes)
}

const fn animus_class(class: Class) -> AnimusClass {
    match class {
        Class::DeclaredSupport => AnimusClass::DeclaredSupport,
        Class::ContractConformance => AnimusClass::ContractConformance,
        Class::TaskFamilyEvaluation => AnimusClass::TaskFamilyEvaluation,
        Class::DeploymentAdmission => AnimusClass::DeploymentAdmission,
    }
}

const fn lattice_class(class: AnimusClass) -> Class {
    match class {
        AnimusClass::DeclaredSupport => Class::DeclaredSupport,
        AnimusClass::ContractConformance => Class::ContractConformance,
        AnimusClass::TaskFamilyEvaluation => Class::TaskFamilyEvaluation,
        AnimusClass::DeploymentAdmission => Class::DeploymentAdmission,
    }
}

const fn unqualified(reason: Reason) -> Unqualified {
    match reason {
        Reason::Malformed => Unqualified::Malformed,
        Reason::SubjectMismatch => Unqualified::SubjectMismatch,
        Reason::ScopeMismatch => Unqualified::ScopeMismatch,
        Reason::InsufficientClass => Unqualified::InsufficientClass,
        Reason::Unusable => Unqualified::Unusable,
        Reason::Stale => Unqualified::Stale,
        Reason::Conflicting => Unqualified::Conflicting,
    }
}

const fn provenance(label: Label) -> Provenance {
    match label {
        Label::Fixture => Provenance::Fixture,
        Label::Live => Provenance::Live,
    }
}

/// Map the producer's answer. A qualified answer carries the subject and
/// provenance label Animus decoded from the chosen attachment's own bytes.
fn verdict(answer: Answer, attachments: &[&[u8]]) -> Verdict {
    match answer {
        Answer::Qualified(qualification) => {
            let Some(document) = decoded(attachments, &qualification.qualification_ref) else {
                // `qualify` names one of the supplied attachments; anything
                // else is not a well-formed answer.
                return Verdict::NotQualified {
                    reason: Unqualified::Malformed,
                };
            };
            let subject = document.subject;
            Verdict::Qualified(lattice_core::agent_revision::ports::Attachment {
                qualification_ref: qualification.qualification_ref,
                subject: lattice_core::agent_revision::ports::Subject {
                    definition_identity: subject.lattice_definition_identity,
                    profile_revision: subject.lattice_profile_revision,
                    task_family: subject.task_family,
                },
                evidence_class: lattice_class(qualification.evidence_class),
                protocol_identity: qualification.protocol_identity,
                verifier_identity: qualification.verifier_identity,
                observed_at_ms: qualification.observed_at_ms,
                provenance: provenance(document.provenance.label),
            })
        }
        Answer::NotQualified(reason) => Verdict::NotQualified {
            reason: unqualified(reason),
        },
        Answer::Unavailable => Verdict::Unavailable {},
    }
}

/// The decoded document of the supplied attachment with `reference`.
fn decoded(attachments: &[&[u8]], reference: &str) -> Option<animus_runtime::qualification::AttachmentDocument> {
    attachments
        .iter()
        .filter_map(|bytes| animus_runtime::qualification::decode_attachment(bytes).ok())
        .find(|attachment| attachment.identity == reference)
        .map(|attachment| attachment.document)
}
