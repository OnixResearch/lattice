/// Replay errors that fail closed without exposing raw adapter-owned payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Replay received no entries.
    EmptyLedger,
    /// Snapshot contains the same entry identifier more than once.
    DuplicateEntryId {
        /// Repeated entry identifier.
        entry_id: alloc::string::String,
    },
    /// Snapshot references a parent identifier that is absent.
    MissingParent {
        /// Child entry identifier.
        entry_id: alloc::string::String,
        /// Missing parent identifier.
        parent_id: alloc::string::String,
    },
    /// Snapshot parent links contain a cycle.
    ParentCycle {
        /// Entry whose ancestry was being validated.
        entry_id: alloc::string::String,
        /// Repeated ancestor identifier.
        cycle_entry_id: alloc::string::String,
    },
    /// Snapshot active leaf does not identify an entry.
    InvalidActiveLeaf {
        /// Requested active leaf identifier.
        active_leaf: alloc::string::String,
    },
    /// Replay encountered an unsupported entry class.
    UnsupportedEntry {
        /// Safe entry identifier.
        entry_id: alloc::string::String,
        /// Safe unsupported kind label.
        kind_label: alloc::string::String,
        /// Redaction marker proving raw payload was omitted.
        redaction: crate::ReceiptRedaction,
    },
    /// Tool result did not directly follow an assistant/model entry.
    OrphanToolResult {
        /// Tool result entry identifier.
        entry_id: alloc::string::String,
    },
    /// Summary could not replace a prior model-visible span.
    MalformedSummaryBoundary {
        /// Summary entry identifier.
        entry_id: alloc::string::String,
    },
    /// Label target did not identify a ledger entry.
    MissingLabelTarget {
        /// Label target identifier.
        target_id: alloc::string::String,
    },
    /// Resume/replay was requested without an injected ledger service.
    MissingLedgerService {
        /// Safe session label.
        session_label: alloc::string::String,
    },
    /// Requested session was absent from the injected service.
    MissingSession {
        /// Safe session label.
        session_label: alloc::string::String,
    },
    /// Injected ledger service was unavailable.
    UnavailableLedgerService {
        /// Safe session label.
        session_label: alloc::string::String,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyLedger => formatter.write_str("ledger replay requires at least one entry"),
            Self::DuplicateEntryId { entry_id } => {
                write!(formatter, "ledger entry id {entry_id} is duplicated")
            }
            Self::MissingParent { entry_id, parent_id } => {
                write!(formatter, "ledger entry {entry_id} references missing parent {parent_id}")
            }
            Self::ParentCycle {
                entry_id,
                cycle_entry_id,
            } => write!(formatter, "ledger entry {entry_id} ancestry repeats {cycle_entry_id}"),
            Self::InvalidActiveLeaf { active_leaf } => {
                write!(formatter, "ledger active leaf {active_leaf} is not present")
            }
            Self::UnsupportedEntry {
                entry_id,
                kind_label,
                redaction,
            } => write!(
                formatter,
                "ledger entry {entry_id} with kind {kind_label} is unsupported and {}",
                redaction.as_str()
            ),
            Self::OrphanToolResult { entry_id } => {
                write!(formatter, "ledger tool result {entry_id} has no assistant parent")
            }
            Self::MalformedSummaryBoundary { entry_id } => {
                write!(formatter, "ledger summary {entry_id} has no prior model-visible span to replace")
            }
            Self::MissingLabelTarget { target_id } => {
                write!(formatter, "ledger label target {target_id} is not present")
            }
            Self::MissingLedgerService { session_label } => {
                write!(formatter, "session {session_label} has no injected ledger service")
            }
            Self::MissingSession { session_label } => {
                write!(formatter, "session {session_label} is not present in ledger service")
            }
            Self::UnavailableLedgerService { session_label } => {
                write!(formatter, "session {session_label} ledger service is unavailable")
            }
        }
    }
}
