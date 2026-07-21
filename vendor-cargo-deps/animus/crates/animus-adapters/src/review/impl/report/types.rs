use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

const VERDICT_PREFIX: &str = "VERDICT:";
const VERDICT_PASS: &str = "PASS";
const VERDICT_WARN: &str = "WARN";
const VERDICT_FAIL: &str = "FAIL";
const SECTION_PREFIX: &str = "## ";
const FINDINGS_SECTION_TITLE: &str = "Findings";
const BULLET_PREFIX: &str = "- ";
const TAG_CLASS: &str = "class";
const TAG_SCOPE: &str = "scope";
const TAG_ROUTE: &str = "route";
const TAG_PROMOTED: &str = "promoted";
const PROMOTED_YES: &str = "yes";
const PROMOTED_NO: &str = "no";
const TAG_SEPARATOR: char = '=';
const TAG_SPLIT_LIMIT: usize = 2;
const KIBIBYTE_BYTES: usize = 1024;
const MAX_REPORT_KIBIBYTES: usize = 32;
const MAX_REPORT_BYTES: usize = MAX_REPORT_KIBIBYTES * KIBIBYTE_BYTES;
const MAX_SAFE_FIELD_CHARS: usize = 160;
const MAX_FINDING_SUMMARY_CHARS: usize = 400;
const MAX_SECTIONS: usize = 32;
const MAX_FINDINGS: usize = 128;
const MIN_PROMOTION_COUNT: usize = 2;
const SINGLE_ITEM_COUNT: usize = 1;
const ZERO_RECORD_COUNT: usize = 0;
const UNKNOWN_TAG_FIELD: &str = "tag";
const SECTION_BODY_JOINER: &str = "\n";
const HASH_SEPARATOR: &str = "|";

const UNSAFE_METADATA_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
    "provider_payload",
    "BEGIN PRIVATE KEY",
    "token=",
    "api_key",
    "password",
    "/home/",
    "/tmp/",
    "C:\\Users\\",
    "unbounded output",
    "raw tool output",
    "raw diff",
    "raw file body",
    "unbounded transcript",
];

/// Borrowed structured review report input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewReportDraft<'a> {
    /// In-memory markdown report text.
    pub markdown: &'a str,
    /// Safe source label supplied by the host.
    pub source: &'a str,
    /// Safe review stage label supplied by the host.
    pub stage: &'a str,
    /// Safe review subject label supplied by the host.
    pub subject: &'a str,
}

/// Borrowed explicit metric-record action input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewMetricRecordDraft<'a> {
    /// Host-supplied monotonic sequence, timestamp label, or event identity.
    pub sequence: &'a str,
}

/// Structured review verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewVerdict {
    /// Review passed without blocking findings.
    Pass,
    /// Review produced warnings that need triage.
    Warn,
    /// Review failed with blocking findings.
    Fail,
}

impl ReviewVerdict {
    /// Returns the stable wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => VERDICT_PASS,
            Self::Warn => VERDICT_WARN,
            Self::Fail => VERDICT_FAIL,
        }
    }
}

/// Typed diagnostic code for review report parsing or recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewDiagnosticCode {
    /// Report did not contain a verdict line.
    MissingVerdict,
    /// Verdict value was not one of the supported labels.
    MalformedVerdict,
    /// Report exceeded the bounded in-memory parser limit.
    ReportTooLarge,
    /// Safe host metadata was missing, too large, or unsafe.
    UnsafeMetadata,
    /// A metric-bearing finding was missing a required tag.
    MissingFindingTag,
    /// A metric-bearing finding repeated a tag.
    DuplicateFindingTag,
    /// A metric-bearing finding used an unknown tag.
    UnknownFindingTag,
    /// A finding used an unsupported promoted value.
    UnknownPromotedValue,
    /// A finding bullet in `## Findings` was malformed.
    MalformedFindingBullet,
    /// A report with parser diagnostics was used for a metrics append action.
    ReportHasDiagnostics,
    /// Too many sections or findings were supplied.
    TooManyItems,
    /// A BLAKE3 hash field was malformed.
    InvalidHash,
    /// A corrective record could not be admitted for prompt influence.
    PromptAdmissionDenied,
    /// A host-owned corrective store was missing.
    MissingStore,
    /// A host-owned corrective store was unavailable.
    UnavailableStore,
    /// A repeated finding used an unsupported corrective-promotion target route.
    UnsupportedPromotionTarget,
}

/// Structured review report error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewReportError {
    code: ReviewDiagnosticCode,
    field: String,
}

impl ReviewReportError {
    fn new(code: ReviewDiagnosticCode, field: &str) -> Self {
        Self {
            code,
            field: field.to_owned(),
        }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> ReviewDiagnosticCode {
        self.code
    }

    /// Returns the safe field label associated with this error.
    #[must_use]
    pub fn field(&self) -> &str {
        self.field.as_str()
    }
}

impl fmt::Display for ReviewReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "review report error {:?} at {}",
            self.code, self.field
        )
    }
}

/// Non-fatal review parser diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewDiagnostic {
    code: ReviewDiagnosticCode,
    field: String,
}

impl ReviewDiagnostic {
    fn new(code: ReviewDiagnosticCode, field: &str) -> Self {
        Self {
            code,
            field: field.to_owned(),
        }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> ReviewDiagnosticCode {
        self.code
    }

    /// Returns the safe diagnostic field label.
    #[must_use]
    pub fn field(&self) -> &str {
        self.field.as_str()
    }
}

/// Parsed markdown section metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewSection {
    title: String,
    line_count: usize,
    body_hash: String,
}

impl ReviewSection {
    /// Returns the section title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the number of body lines in the section.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Returns the BLAKE3 hash of the section body.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }
}

/// Parsed metric-bearing finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewFinding {
    class: String,
    scope: String,
    route: String,
    promoted: bool,
    summary: String,
    summary_hash: String,
}

impl ReviewFinding {
    /// Returns the finding class tag.
    #[must_use]
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    /// Returns the finding scope tag.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns the finding prevention route tag.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Returns whether this finding is already promoted to a durable rail.
    #[must_use]
    pub const fn promoted(&self) -> bool {
        self.promoted
    }

    /// Returns the human-readable finding summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the BLAKE3 hash of the finding summary.
    #[must_use]
    pub fn summary_hash(&self) -> &str {
        self.summary_hash.as_str()
    }
}
