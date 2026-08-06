const HOST_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("host_id");
const RESOURCE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_resource_id");
const LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("host_label");
const OPERATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_operation");
const KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("host_kind");
const LIFETIME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_selector_lifetime");
const SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_output_source");
const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_output_redaction");
const OUTPUT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_output_body");
const REFERENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_output_reference");
const CORRELATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_correlation_id");
const VERSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_protocol_version");
const FEATURE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_feature");
const AXIS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("layout_axis");
const NODE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("layout_node_id");
const STATUS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("host_status");
const CWD_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_cwd_summary");
const PROVENANCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_provenance_summary");
const COMMAND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("layout_command");
const COMMAND_ARG_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("layout_command_arg");
const EVENT_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_event_kind");
const ERROR_CODE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_error_code");
const ERROR_MESSAGE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_error_message");
const UNKNOWN_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_unknown_field");

const LIFETIME_LIVE: &str = "live";
const LIFETIME_DURABLE: &str = "durable";
const KIND_WORKSPACE: &str = "workspace";
const KIND_TAB: &str = "tab";
const KIND_PANE: &str = "pane";
const KIND_AGENT: &str = "agent";
const OP_FOCUS: &str = "focus";
const OP_RENAME: &str = "rename";
const OP_CLOSE: &str = "close";
const OP_READ: &str = "read";
const OP_SEND: &str = "send";
const OP_WAIT: &str = "wait";
const OP_SUBSCRIBE: &str = "subscribe";
const OP_EXPORT_LAYOUT: &str = "export_layout";
const OP_APPLY_LAYOUT: &str = "apply_layout";
const SOURCE_VISIBLE: &str = "visible";
const SOURCE_RECENT: &str = "recent";
const SOURCE_RECENT_UNWRAPPED: &str = "recent_unwrapped";
const SOURCE_DETECTOR_SAFE: &str = "detector_safe";
const REDACTION_CLEAN: &str = "clean";
const REDACTION_REDACTED: &str = "redacted";
const AXIS_HORIZONTAL: &str = "horizontal";
const AXIS_VERTICAL: &str = "vertical";
const STATUS_IDLE: &str = "idle";
const STATUS_RUNNING: &str = "running";
const STATUS_WAITING: &str = "waiting";
const STATUS_DONE: &str = "done";
const STATUS_FAILED: &str = "failed";
const EVENT_AGENT_STATUS: &str = "agent_status";
const EVENT_OUTPUT_MATCH: &str = "output_match";
const EVENT_PANE_LIFECYCLE: &str = "pane_lifecycle";
const EVENT_CANCELLED: &str = "cancelled";
const EVENT_TIMEOUT: &str = "timeout";
const EVENT_HOST_ERROR: &str = "host_error";
const PROTOCOL_VERSION: &str = "animus-host-control-v1";
const FEATURE_LAYOUT: &str = "layout";
const FEATURE_WAITS: &str = "waits";
const FEATURE_BOUNDED_READS: &str = "bounded_reads";
const OUTPUT_HASH_SCHEMA: &str = "animus.host-control.output.v1";
const HASH_SEPARATOR: &str = "|";
const ABSOLUTE_PATH_PREFIX: &str = "/";
const HOME_PATH_PREFIX: &str = "~";
const URI_SCHEME_MARKER: &str = "://";
const WINDOWS_SEPARATOR: char = '\\';
const SECRET_MARKERS: &[&str] = &["BEGIN PRIVATE KEY", "API_KEY=", "RAW_SECRET:"];
const MAX_HOST_OUTPUT_LINES: u32 = 1_024;
const MIN_HOST_OUTPUT_LINES: u32 = 1;
const NO_TIMEOUT_TICKS: u32 = 0;
#[cfg(test)]
const DEFAULT_TIMEOUT_TICKS: u32 = 10;

/// Host-control validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostControlError {
    /// Generic adapter text validation failed.
    Contract(crate::AdapterContractError),
    /// A host-control field value is unsupported.
    UnsupportedValue {
        /// Field that failed validation.
        field: &'static str,
        /// Unsupported normalized value.
        value: String,
    },
    /// A command did not include a required selector.
    MissingSelector,
    /// A selector was not in the host topology snapshot.
    UnknownSelector,
    /// A live selector revision did not match the current topology revision.
    StaleSelector,
    /// A selector belonged to a different host.
    CrossHostSelector,
    /// A rename label collided with an existing label.
    DuplicateLabel,
    /// A command operation is not supported by the host snapshot.
    UnsupportedOperation,
    /// A read request was unbounded or exceeded the host-control line budget.
    UnboundedRead,
    /// Output or a safe reference contained secret-like or unsafe material.
    UnsafeOutput,
    /// A declarative layout tree was malformed.
    InvalidLayout,
    /// A wait request did not declare a finite timeout.
    InvalidTimeout,
    /// A protocol envelope used an unsupported version.
    UnsupportedVersion,
}

impl From<crate::AdapterContractError> for HostControlError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Host-controlled resource kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostResourceKind {
    /// Host workspace.
    Workspace,
    /// Host tab.
    Tab,
    /// Host pane.
    Pane,
    /// Host agent.
    Agent,
}

impl HostResourceKind {
    /// Parses a resource kind label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(KIND_FIELD, value, |normalized| match normalized {
            KIND_WORKSPACE => Some(Self::Workspace),
            KIND_TAB => Some(Self::Tab),
            KIND_PANE => Some(Self::Pane),
            KIND_AGENT => Some(Self::Agent),
            _unsupported => None,
        })
    }

    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => KIND_WORKSPACE,
            Self::Tab => KIND_TAB,
            Self::Pane => KIND_PANE,
            Self::Agent => KIND_AGENT,
        }
    }
}

/// Selector lifetime declared by a host.
// r[impl agent_harness_runtime.host_control.selector_lifetime]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorLifetime {
    /// Selector is valid only for a matching topology revision.
    Live,
    /// Selector is host-declared durable across topology revisions.
    Durable,
}

impl SelectorLifetime {
    /// Parses a selector lifetime label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(LIFETIME_FIELD, value, |normalized| match normalized {
            LIFETIME_LIVE => Some(Self::Live),
            LIFETIME_DURABLE => Some(Self::Durable),
            _unsupported => None,
        })
    }

    /// Returns the stable lifetime label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Live => LIFETIME_LIVE,
            Self::Durable => LIFETIME_DURABLE,
        }
    }
}

/// Host-control command operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostOperation {
    /// Focus a selected resource.
    Focus,
    /// Rename a selected resource.
    Rename,
    /// Close a selected resource.
    Close,
    /// Read bounded output.
    Read,
    /// Send host input.
    Send,
    /// Wait for semantic state.
    Wait,
    /// Subscribe to host events.
    Subscribe,
    /// Export a declarative layout.
    ExportLayout,
    /// Apply a declarative layout.
    ApplyLayout,
}

impl HostOperation {
    /// Parses a host operation label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(OPERATION_FIELD, value, |normalized| match normalized {
            OP_FOCUS => Some(Self::Focus),
            OP_RENAME => Some(Self::Rename),
            OP_CLOSE => Some(Self::Close),
            OP_READ => Some(Self::Read),
            OP_SEND => Some(Self::Send),
            OP_WAIT => Some(Self::Wait),
            OP_SUBSCRIBE => Some(Self::Subscribe),
            OP_EXPORT_LAYOUT => Some(Self::ExportLayout),
            OP_APPLY_LAYOUT => Some(Self::ApplyLayout),
            _unsupported => None,
        })
    }

    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focus => OP_FOCUS,
            Self::Rename => OP_RENAME,
            Self::Close => OP_CLOSE,
            Self::Read => OP_READ,
            Self::Send => OP_SEND,
            Self::Wait => OP_WAIT,
            Self::Subscribe => OP_SUBSCRIBE,
            Self::ExportLayout => OP_EXPORT_LAYOUT,
            Self::ApplyLayout => OP_APPLY_LAYOUT,
        }
    }
}
