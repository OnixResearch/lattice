//! Implementation-free contracts for the optional Pijul CLI adapter.
//!
//! These DTOs carry explicit requests and bounded executor observations. They do
//! not discover repositories or executables, open `.pijul` databases, spawn
//! processes, read configuration or identity files, access networks, or persist
//! results. Callers must pass loaded outputs through the pure runtime validator
//! before they project or retain any observation.

/// Exact reviewed Pijul CLI release.
pub const PIJUL_CLI_RELEASE: &str = "1.0.0-beta.18";
/// Upstream change immediately preceding the reviewed crates.io publication.
pub const PIJUL_SOURCE_CHANGE: &str = "QRYOAJLYA5KO7JAYU6LMYHPU3NSZCTUMYEPCK3HNZIRIDZDWSMBAC";
/// Upstream main-channel state immediately preceding the reviewed publication.
pub const PIJUL_SOURCE_STATE: &str = "RISR562S7FZP3NNYDFVBQUMDEGB6YSSMSH2RGZDRYBIPGVQDSLXQC";
/// crates.io SHA-256 checksum for the reviewed CLI crate.
pub const PIJUL_CRATE_CHECKSUM_SHA256: &str =
    "0500d14ab41adea4f0d71f03b9d1be895a0638e025204f5b6ab90e46c40684ff";
/// Domain identity for the reviewed release, source state, formats, and license.
pub const PIJUL_COHORT_IDENTITY: &str =
    "ed1fcd07852103dd8ea0b2e824aac6cad51406c987e54240077888ce796aee2b";
/// Exact log JSON schema identity reviewed for this adapter.
pub const PIJUL_LOG_OUTPUT_FORMAT: &str = "pijul-log-hash-json-v1-beta18";
/// Exact diff JSON schema identity reviewed for this adapter.
pub const PIJUL_DIFF_OUTPUT_FORMAT: &str = "pijul-diff-json-v1-beta18";
/// Exact channel line schema identity reviewed for this adapter.
pub const PIJUL_CHANNEL_OUTPUT_FORMAT: &str = "pijul-channel-lines-v1-beta18";
/// Reviewed CLI crate license.
pub const PIJUL_CLI_LICENSE: &str = "GPL-2.0-or-later";

/// Product behaviors excluded from this implementation-free boundary.
pub const PROHIBITED_PIJUL_EFFECTS: &[&str] = &[
    "executable discovery",
    "PATH search",
    "repository discovery",
    ".pijul database access",
    "process spawning",
    "ambient environment reads",
    "identity key reads",
    "network access",
    "automatic record",
    "automatic persistence",
];

/// Pijul operation represented at the adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulOperation {
    /// Bounded log query using the reviewed JSON output.
    Log,
    /// Bounded working-copy diff query using the reviewed JSON output.
    Diff,
    /// Bounded channel-list query using the reviewed line output.
    ChannelList,
    /// Change inspection, unsupported because beta.18 has no safe machine form.
    Change,
    /// Protected record mutation.
    Record,
    /// Protected unrecord mutation.
    Unrecord,
    /// Protected local apply mutation.
    Apply,
    /// Protected channel fork mutation.
    Fork,
    /// Protected working-copy reset mutation.
    Reset,
    /// Protected channel create, delete, rename, or switch mutation.
    ChannelMutation,
}

impl PijulOperation {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Diff => "diff",
            Self::ChannelList => "channel_list",
            Self::Change => "change",
            Self::Record => "record",
            Self::Unrecord => "unrecord",
            Self::Apply => "apply",
            Self::Fork => "fork",
            Self::Reset => "reset",
            Self::ChannelMutation => "channel_mutation",
        }
    }

    /// Returns true for operations that can mutate repository or working-copy state.
    #[must_use]
    pub const fn is_protected_mutation(self) -> bool {
        matches!(
            self,
            Self::Record
                | Self::Unrecord
                | Self::Apply
                | Self::Fork
                | Self::Reset
                | Self::ChannelMutation
        )
    }
}

/// Explicit host confirmation state for protected operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulConfirmation {
    /// The operation is read-only and needs no mutation confirmation.
    NotRequired,
    /// The host confirmed the exact protected operation.
    Confirmed,
    /// The host denied the operation.
    Denied,
    /// Confirmation was required but absent.
    Missing,
}

/// Signature observation class reported without authenticity claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulSignatureStatus {
    /// The selected machine output does not report signature state.
    NotReported,
    /// A signature marker was present but was not verified by Animus.
    PresentUnverified,
    /// The CLI explicitly reported no signature.
    Absent,
    /// The selected cohort does not support the requested signature observation.
    Unsupported,
}

impl PijulSignatureStatus {
    /// Returns the stable signature-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotReported => "not-reported",
            Self::PresentUnverified => "present-unverified",
            Self::Absent => "absent",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Executor completion class supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulExecutionStatus {
    /// The child exited before the deadline.
    Exited,
    /// The deadline elapsed.
    DeadlineExceeded,
    /// Cancellation was observed.
    Cancelled,
    /// The injected executor was unavailable.
    Unavailable,
}

/// Explicit bounded request for one Pijul adapter operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulRequest {
    /// Exact operation.
    pub operation: PijulOperation,
    /// Explicit repository path. Receipts must not retain this value.
    pub repository_path: alloc::string::String,
    /// Stable safe repository identity supplied by the host.
    pub repository_identity: alloc::string::String,
    /// Optional exact channel name.
    pub channel: Option<alloc::string::String>,
    /// Optional full change hash or unambiguous prefix.
    pub change_selector: Option<alloc::string::String>,
    /// Explicit confirmation state.
    pub confirmation: PijulConfirmation,
}

/// Explicit executable binding for the injected CLI service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulExecutable {
    /// Absolute executable path selected by the host or Nix helper.
    pub path: alloc::string::String,
    /// Exact safe executable identity expected by policy.
    pub identity: alloc::string::String,
    /// Exact reported release expected before invocation.
    pub release: alloc::string::String,
}

/// Named finite executor bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PijulExecutionBounds {
    /// Maximum standard-input bytes.
    pub max_stdin_bytes: u64,
    /// Maximum standard-output bytes.
    pub max_stdout_bytes: u64,
    /// Maximum standard-error bytes.
    pub max_stderr_bytes: u64,
    /// Maximum resident memory bytes admitted by the executor profile.
    pub max_memory_bytes: u64,
    /// Maximum CPU milliseconds admitted by the executor profile.
    pub max_cpu_milliseconds: u64,
    /// Deadline in milliseconds.
    pub deadline_milliseconds: u64,
}

/// Purely planned child-process request for a caller-injected bounded executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulExecutionRequest {
    /// Exact executable binding.
    pub executable: PijulExecutable,
    /// Explicit argv items, excluding argv zero.
    pub arguments: alloc::vec::Vec<alloc::string::String>,
    /// Explicit repository working directory.
    pub working_directory: alloc::string::String,
    /// Explicit bounded standard input.
    pub standard_input: alloc::vec::Vec<u8>,
    /// Explicit environment entries. The reviewed plan uses an empty set.
    pub environment: alloc::vec::Vec<(alloc::string::String, alloc::string::String)>,
    /// Whether ambient environment inheritance is allowed. The reviewed plan sets false.
    pub inherit_environment: bool,
    /// Named finite bounds.
    pub bounds: PijulExecutionBounds,
}

/// Bounded bytes and status returned by an injected executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulExecutionOutcome {
    /// Executor completion class.
    pub status: PijulExecutionStatus,
    /// Child exit code when the child exited.
    pub exit_code: Option<i32>,
    /// Retained bounded standard output.
    pub standard_output: alloc::vec::Vec<u8>,
    /// Retained bounded standard error.
    pub standard_error: alloc::vec::Vec<u8>,
    /// True when standard output was truncated by the executor.
    pub stdout_truncated: bool,
    /// True when standard error was truncated by the executor.
    pub stderr_truncated: bool,
}

/// Host-injected bounded child-process executor.
pub trait PijulBoundedExecutor {
    /// Executes exactly one planned request.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue when the executor refuses or cannot run the request.
    fn execute(
        &mut self,
        request: &PijulExecutionRequest,
    ) -> Result<PijulExecutionOutcome, PijulServiceIssue>;
}

/// One loaded log entry before pure cohort validation and redaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulLoadedLogEntry {
    /// Full change hash.
    pub hash: alloc::string::String,
    /// Optional channel state identity.
    pub state: Option<alloc::string::String>,
    /// Descriptive author strings loaded from CLI output.
    pub authors: alloc::vec::Vec<alloc::string::String>,
    /// Optional descriptive timestamp.
    pub timestamp: Option<alloc::string::String>,
    /// Optional change message, omitted from all validated projections.
    pub message: Option<alloc::string::String>,
    /// Optional full description, denied by the reviewed profile.
    pub description: Option<alloc::string::String>,
    /// Optional repository-relative paths, omitted from default projections.
    pub paths: alloc::vec::Vec<alloc::string::String>,
}

/// One loaded diff status before pure cohort validation and path redaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulLoadedDiffEntry {
    /// Repository-relative path loaded from the CLI.
    pub path: alloc::string::String,
    /// Exact beta.18 operation label.
    pub operation: alloc::string::String,
    /// Optional one-based descriptive line position.
    pub line: Option<u64>,
}

/// One loaded channel-list entry before pure policy validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulLoadedChannel {
    /// Channel name.
    pub name: alloc::string::String,
    /// Whether the CLI marked this channel as current.
    pub is_current: bool,
}

/// Loaded protocol output that still requires pure validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PijulLoadedOutput {
    /// Reviewed beta.18 log JSON values.
    Log(alloc::vec::Vec<PijulLoadedLogEntry>),
    /// Reviewed beta.18 diff JSON values.
    Diff(alloc::vec::Vec<PijulLoadedDiffEntry>),
    /// Reviewed beta.18 channel line values.
    Channels(alloc::vec::Vec<PijulLoadedChannel>),
}

/// Untrusted backend response from the host service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulBackendResponse {
    /// Requested operation.
    pub operation: PijulOperation,
    /// Executor outcome used to classify exit, deadline, and truncation facts.
    pub execution: PijulExecutionOutcome,
    /// Parsed loaded output. This value is not validated evidence.
    pub loaded: Option<PijulLoadedOutput>,
}

/// Implementation-free Pijul backend service port.
pub trait PijulBackendService {
    /// Handles one explicit operation through the host implementation.
    ///
    /// # Errors
    ///
    /// Returns a typed issue before projection when the service is unavailable,
    /// the cohort mismatches, parsing fails, or the operation is denied.
    fn execute(
        &mut self,
        request: &PijulRequest,
    ) -> Result<PijulBackendResponse, PijulServiceIssue>;
}

/// Fail-closed service issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PijulServiceIssueKind {
    /// No compatible service or executor was injected.
    Unavailable,
    /// Executable release or identity mismatched the profile.
    CohortMismatch,
    /// The operation is not admitted by the exact cohort policy.
    UnsupportedOperation,
    /// A protected operation lacked exact host confirmation.
    ConfirmationRequired,
    /// Input metadata was malformed or unsafe.
    InvalidRequest,
    /// The executor refused or failed the request.
    ExecutorFailure,
    /// Output bytes did not match the reviewed protocol.
    ProtocolFailure,
    /// Output exceeded a configured bound or was truncated.
    BoundExceeded,
}

/// Safe typed service issue without raw output, private paths, or credentials.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulServiceIssue {
    kind: PijulServiceIssueKind,
    subject: alloc::string::String,
}

impl PijulServiceIssue {
    /// Creates a safe service issue.
    #[must_use]
    pub fn new(kind: PijulServiceIssueKind, subject: impl Into<alloc::string::String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn kind(&self) -> PijulServiceIssueKind {
        self.kind
    }

    /// Returns the safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }
}
