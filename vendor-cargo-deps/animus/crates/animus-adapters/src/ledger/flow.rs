mod fault;

pub use fault::Error;

const SESSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ledger_session_label");
const EMPTY_ENTRY_COUNT: usize = 0;
const NO_CONTEXT_ITEMS: usize = 0;

/// Successful replay result for neutral entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    context: alloc::vec::Vec<super::ContextItem>,
    counts: super::ReplayCounts,
    labels: alloc::vec::Vec<super::Label>,
}

impl Report {
    pub(super) fn new(
        context: alloc::vec::Vec<super::ContextItem>,
        counts: super::ReplayCounts,
        labels: alloc::vec::Vec<super::Label>,
    ) -> Self {
        Self {
            context,
            counts,
            labels,
        }
    }

    /// Returns model-visible context in replay order.
    #[must_use]
    pub fn context(&self) -> &[super::ContextItem] {
        self.context.as_slice()
    }

    /// Returns safe metadata counts.
    #[must_use]
    pub const fn counts(&self) -> super::ReplayCounts {
        self.counts
    }

    /// Returns operator-facing labels preserved as replay-neutral metadata.
    #[must_use]
    pub fn labels(&self) -> &[super::Label] {
        self.labels.as_slice()
    }
}

/// Replay request sent to an injected session-ledger service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    session_label: alloc::string::String,
}

impl Request {
    /// Builds a replay request with a safe session label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the session label is empty,
    /// overlong, or contains a NUL character.
    pub fn new(session_label: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            session_label: crate::validation::normalized_adapter_field(
                SESSION_FIELD,
                session_label,
            )?,
        })
    }

    /// Returns the safe session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        self.session_label.as_str()
    }
}

/// Session-ledger adapter contract.
pub trait SessionLedgerService:
    crate::ComponentAdapter<Request, crate::ServiceOutcome<Report>>
{
}

impl<T> SessionLedgerService for T where
    T: crate::ComponentAdapter<Request, crate::ServiceOutcome<Report>>
{
}

/// Replays neutral entries into model-visible context plus safe counts.
///
/// # Errors
///
/// Returns [`Error`] when the ledger is empty or contains an unsupported or
/// adapter-owned entry marker.
pub fn replay(entries: &[super::Entry]) -> Result<Report, Error> {
    if entries.len() == EMPTY_ENTRY_COUNT {
        return Err(Error::EmptyLedger);
    }

    let mut context = alloc::vec::Vec::with_capacity(entries.len());
    let mut counts = super::ReplayCounts::default();
    for entry in entries {
        counts.record_entry();
        if let Some(item) = replay_entry(entry, &mut counts)? {
            context.push(item);
        }
    }

    Ok(Report::new(context, counts, alloc::vec::Vec::new()))
}

/// Builds the fail-closed outcome for an absent session-ledger service.
#[must_use]
pub fn missing_service(_request: &Request) -> crate::ServiceOutcome<Report> {
    let receipt = crate::Receipt::new(crate::ReceiptDraft::new(
        crate::ServiceKind::SessionLedger,
        crate::CapabilityClass::SessionPersistence,
        crate::OutcomeKind::Absent,
        crate::Reason::NotInjected,
    ));
    crate::ServiceOutcome::absent(receipt)
}

/// Builds the typed error for a missing ledger service before execution.
#[must_use]
pub fn missing_service_error(request: &Request) -> Error {
    Error::MissingLedgerService {
        session_label: alloc::string::String::from(request.session_label()),
    }
}

/// Builds the typed error for a missing session before execution.
#[must_use]
pub fn missing_session_error(request: &Request) -> Error {
    Error::MissingSession {
        session_label: alloc::string::String::from(request.session_label()),
    }
}

/// Builds the typed error for an unavailable ledger service before execution.
#[must_use]
pub fn unavailable_service_error(request: &Request) -> Error {
    Error::UnavailableLedgerService {
        session_label: alloc::string::String::from(request.session_label()),
    }
}

pub(super) fn replay_entry(
    entry: &super::Entry,
    counts: &mut super::ReplayCounts,
) -> Result<Option<super::ContextItem>, Error> {
    match entry {
        super::Entry::UserMessage { id, content } => Ok(Some(model_item(
            counts,
            super::ContextRole::User,
            id,
            content,
        ))),
        super::Entry::AssistantMessage { id, content } => Ok(Some(model_item(
            counts,
            super::ContextRole::Assistant,
            id,
            content,
        ))),
        super::Entry::ToolResult { id, content, .. } => Ok(Some(model_item(
            counts,
            super::ContextRole::Tool,
            id,
            content,
        ))),
        super::Entry::Summary { id, content, .. } => {
            counts.record_summary();
            Ok(Some(model_item(
                counts,
                super::ContextRole::System,
                id,
                content,
            )))
        }
        super::Entry::UsageObservation { .. } => {
            counts.record_usage();
            Ok(None)
        }
        super::Entry::SafeReceipt { .. } => {
            counts.record_receipt();
            Ok(None)
        }
        super::Entry::Unsupported { id, kind_label }
        | super::Entry::AdapterOwned { id, kind_label } => Err(unsupported_entry(id, kind_label)),
    }
}

pub(super) fn model_item(
    counts: &mut super::ReplayCounts,
    role: super::ContextRole,
    source_id: &super::EntryId,
    content: &str,
) -> super::ContextItem {
    counts.record_model_visible();
    super::ContextItem::new(
        role,
        alloc::string::String::from(content),
        source_id.clone(),
    )
}

pub(super) fn unsupported_entry(id: &super::EntryId, kind_label: &str) -> Error {
    Error::UnsupportedEntry {
        entry_id: id_string(id),
        kind_label: alloc::string::String::from(kind_label),
        redaction: crate::ReceiptRedaction::OutputBounded,
    }
}

pub(super) fn id_string(id: &super::EntryId) -> alloc::string::String {
    alloc::string::String::from(id.as_str())
}

const _: () = assert!(NO_CONTEXT_ITEMS == EMPTY_ENTRY_COUNT);
