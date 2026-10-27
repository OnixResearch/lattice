//! Implementation-free observability adapter contracts.
//!
//! Metrics and logging adapters receive only validated safe projections. Concrete
//! sinks, global subscribers, files, clocks, network exporters, and dashboards
//! remain host-owned behavior outside reusable crates.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::ToString;

const METRIC_SCHEMA: &str = "animus.metric-sample.v1";
const LOG_SCHEMA: &str = "animus.log-record.v1";
const SINK_SCHEMA: &str = "animus.observability-sink.v1";
const OBSERVABILITY_OPERATION_LABEL: &str = "observability";
const MIN_LABEL_COUNT: usize = 1;
const MAX_LABEL_COUNT: usize = 16;
const MAX_MESSAGE_CHARS: usize = animus_adapter_safe::MAX_LABEL_CHARS;
const LABEL_JOINER: char = ':';
const LABEL_JOINER_WIDTH: usize = 1;
const SAFE_LABEL_PREFIX_COUNT: usize = 3;

/// Observability contract validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Metric, target, label, or sink name was empty.
    EmptyName,
    /// A label collection was missing or exceeded bounded cardinality.
    InvalidLabelCount,
    /// Safe receipt metadata validation rejected the record.
    UnsafeMetadata(animus_adapter_safe::Error),
    /// Message label exceeded the bounded log-message contract.
    UnboundedMessage,
    /// The sink is known but unavailable.
    SinkUnavailable,
    /// The sink or collector is closed.
    CollectorClosed,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("observability name must not be empty"),
            Self::InvalidLabelCount => {
                formatter.write_str("observability labels are out of bounds")
            }
            Self::UnsafeMetadata(error) => write!(formatter, "{error}"),
            Self::UnboundedMessage => formatter.write_str("observability message is unbounded"),
            Self::SinkUnavailable => formatter.write_str("observability sink is unavailable"),
            Self::CollectorClosed => formatter.write_str("observability collector is closed"),
        }
    }
}

/// Metric sample kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricKind {
    /// Monotonic counter sample.
    Counter,
    /// Gauge sample.
    Gauge,
    /// Duration bucket sample.
    DurationBucket,
}

impl MetricKind {
    /// Returns the stable metric-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::DurationBucket => "duration_bucket",
        }
    }
}

/// Metric unit label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricUnit {
    /// Unitless count.
    Count,
    /// Millisecond duration.
    Milliseconds,
    /// Byte count.
    Bytes,
}

impl MetricUnit {
    /// Returns the stable unit label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Milliseconds => "milliseconds",
            Self::Bytes => "bytes",
        }
    }
}

/// Safe log level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    /// Informational record.
    Info,
    /// Warning record.
    Warning,
    /// Error record.
    Error,
}

impl LogLevel {
    /// Returns the stable log-level label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Sink delivery policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryPolicy {
    /// Sink failures are diagnostics and do not block runtime execution.
    BestEffort,
    /// Required sink delivery failure blocks guarded adapter dispatch.
    AuditRequired,
}

impl DeliveryPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BestEffort => "best_effort",
            Self::AuditRequired => "audit_required",
        }
    }
}

/// Sink failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SinkFailureClass {
    /// Sink was unavailable.
    Unavailable,
    /// Sink collector was closed.
    Closed,
    /// Sink record was malformed.
    MalformedRecord,
    /// Sink record was unsafe or leak-bearing.
    UnsafeRecord,
    /// Audit policy denied guarded dispatch.
    AuditPolicyDenied,
}

impl SinkFailureClass {
    /// Returns the stable sink-failure label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Closed => "closed",
            Self::MalformedRecord => "malformed_record",
            Self::UnsafeRecord => "unsafe_record",
            Self::AuditPolicyDenied => "audit_policy_denied",
        }
    }
}

/// Borrowed metric label before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetricLabelDraft<'a> {
    /// Label key.
    pub key: &'a str,
    /// Label value.
    pub value: &'a str,
}

impl<'a> MetricLabelDraft<'a> {
    /// Builds a metric label draft.
    #[must_use]
    pub const fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}

/// Validated metric label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricLabel {
    key: alloc::string::String,
    value: alloc::string::String,
}

impl MetricLabel {
    /// Builds a label after safe metadata validation.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when key or value is empty or unsafe.
    pub fn new(draft: MetricLabelDraft<'_>) -> Result<Self, Error> {
        ensure_name(draft.key)?;
        ensure_name(draft.value)?;
        validate_safe_labels(&[draft.key, draft.value])?;
        Ok(Self {
            key: draft.key.to_string(),
            value: draft.value.to_string(),
        })
    }

    /// Returns the label key.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the label value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    fn joined(&self) -> alloc::string::String {
        let capacity_bytes = self
            .key
            .len()
            .saturating_add(self.value.len())
            .saturating_add(LABEL_JOINER_WIDTH);
        let mut joined = alloc::string::String::with_capacity(capacity_bytes);
        joined.push_str(self.key.as_str());
        joined.push(LABEL_JOINER);
        joined.push_str(self.value.as_str());
        joined
    }
}

/// Borrowed safe metric sample before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetricSampleDraft<'a> {
    /// Metric name.
    pub name: &'a str,
    /// Metric kind.
    pub kind: MetricKind,
    /// Metric unit.
    pub unit: MetricUnit,
    /// Non-negative metric value.
    pub value: u64,
    /// Safe metric labels.
    pub labels: &'a [MetricLabelDraft<'a>],
}

/// Validated safe metric sample.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricSample {
    name: alloc::string::String,
    kind: MetricKind,
    unit: MetricUnit,
    value: u64,
    labels: alloc::vec::Vec<MetricLabel>,
}

impl MetricSample {
    /// Builds a metric sample after validating labels and safe metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] for invalid names, labels, or unsafe metadata.
    pub fn new(draft: MetricSampleDraft<'_>) -> Result<Self, Error> {
        ensure_name(draft.name)?;
        ensure_label_count(draft.labels.len())?;
        let labels = draft
            .labels
            .iter()
            .copied()
            .map(MetricLabel::new)
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        let label_storage = label_strings(labels.as_slice());
        let label_refs = label_refs(label_storage.as_slice());
        validate_safe_label_envelope(&SafeLabelEnvelope {
            schema: METRIC_SCHEMA,
            name: draft.name,
            labels: &label_refs,
        })?;
        Ok(Self {
            name: draft.name.to_string(),
            kind: draft.kind,
            unit: draft.unit,
            value: draft.value,
            labels,
        })
    }

    /// Returns the metric name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns metric kind.
    #[must_use]
    pub const fn kind(&self) -> MetricKind {
        self.kind
    }

    /// Returns metric unit.
    #[must_use]
    pub const fn unit(&self) -> MetricUnit {
        self.unit
    }

    /// Returns metric value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }

    /// Returns safe labels.
    #[must_use]
    pub fn labels(&self) -> &[MetricLabel] {
        self.labels.as_slice()
    }
}

/// Borrowed safe log record before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogRecordDraft<'a> {
    /// Log target label.
    pub target: &'a str,
    /// Log level.
    pub level: LogLevel,
    /// Bounded safe message label, not raw runtime payload text.
    pub message: &'a str,
    /// Safe labels.
    pub labels: &'a [MetricLabelDraft<'a>],
}

impl<'a> LogRecordDraft<'a> {
    /// Builds a log record draft.
    #[must_use]
    pub const fn new(
        target: &'a str,
        level: LogLevel,
        message: &'a str,
        labels: &'a [MetricLabelDraft<'a>],
    ) -> Self {
        Self {
            target,
            level,
            message,
            labels,
        }
    }
}

/// Validated safe log record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogRecord {
    target: alloc::string::String,
    level: LogLevel,
    message: alloc::string::String,
    labels: alloc::vec::Vec<MetricLabel>,
}

impl LogRecord {
    /// Builds a safe log record after rejecting unsafe metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] for invalid names, labels, message bounds, or leaks.
    pub fn new(draft: LogRecordDraft<'_>) -> Result<Self, Error> {
        ensure_name(draft.target)?;
        ensure_name(draft.message)?;
        if draft.message.chars().count() > MAX_MESSAGE_CHARS {
            return Err(Error::UnboundedMessage);
        }
        ensure_label_count(draft.labels.len())?;
        let labels = draft
            .labels
            .iter()
            .copied()
            .map(MetricLabel::new)
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        let label_storage = label_strings(labels.as_slice());
        let label_refs = label_refs(label_storage.as_slice());
        validate_safe_label_envelope(&SafeLabelEnvelope {
            schema: LOG_SCHEMA,
            name: draft.target,
            labels: &label_refs,
        })?;
        validate_safe_labels(&[draft.message])?;
        Ok(Self {
            target: draft.target.to_string(),
            level: draft.level,
            message: draft.message.to_string(),
            labels,
        })
    }

    /// Returns the log target.
    #[must_use]
    pub fn target(&self) -> &str {
        self.target.as_str()
    }

    /// Returns the log level.
    #[must_use]
    pub const fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the safe message label.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Returns safe labels.
    #[must_use]
    pub fn labels(&self) -> &[MetricLabel] {
        self.labels.as_slice()
    }
}

/// Sink delivery outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SinkOutcome {
    /// Sink accepted the record.
    Delivered,
    /// Sink failed under best-effort or audit policy.
    Failed {
        /// Stable failure class.
        class: SinkFailureClass,
        /// Safe reason label.
        reason: alloc::string::String,
    },
}

impl SinkOutcome {
    /// Builds a failed outcome after validating the reason label.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the reason leaks unsafe metadata.
    pub fn failed(class: SinkFailureClass, reason: &str) -> Result<Self, Error> {
        ensure_name(reason)?;
        validate_safe_labels(&[SINK_SCHEMA, class.as_str(), reason])?;
        Ok(Self::Failed {
            class,
            reason: reason.to_string(),
        })
    }

    /// Returns true when the sink delivered the record.
    #[must_use]
    pub const fn is_delivered(&self) -> bool {
        matches!(self, Self::Delivered)
    }
}

/// Decision for guarded dispatch after observability delivery is evaluated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardDecision {
    /// Guarded dispatch may continue.
    Continue,
    /// Guarded dispatch must fail closed before adapter execution.
    Blocked {
        /// Stable failure class.
        class: SinkFailureClass,
        /// Safe reason label.
        reason: alloc::string::String,
    },
}

impl GuardDecision {
    /// Returns true when guarded dispatch may continue.
    #[must_use]
    pub const fn allows_dispatch(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

/// Evaluates sink delivery according to best-effort or audit-required policy.
#[must_use]
pub fn evaluate_delivery(policy: DeliveryPolicy, outcome: &SinkOutcome) -> GuardDecision {
    match (policy, outcome) {
        (DeliveryPolicy::BestEffort, _)
        | (DeliveryPolicy::AuditRequired, SinkOutcome::Delivered) => GuardDecision::Continue,
        (DeliveryPolicy::AuditRequired, SinkOutcome::Failed { class, reason }) => {
            GuardDecision::Blocked {
                class: *class,
                reason: reason.clone(),
            }
        }
    }
}

/// Implementation-free metrics sink contract.
pub trait MetricsAdapter {
    /// Emits a validated metric sample.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the sink cannot accept the sample.
    fn emit_metric(&mut self, sample: &MetricSample) -> Result<SinkOutcome, Error>;
}

/// Implementation-free logging sink contract.
pub trait LoggingAdapter {
    /// Emits a validated log record.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the sink cannot accept the record.
    fn emit_log(&mut self, record: &LogRecord) -> Result<SinkOutcome, Error>;
}

fn ensure_name(value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::EmptyName);
    }
    Ok(())
}

fn ensure_label_count(count: usize) -> Result<(), Error> {
    if !(MIN_LABEL_COUNT..=MAX_LABEL_COUNT).contains(&count) {
        return Err(Error::InvalidLabelCount);
    }
    Ok(())
}

fn validate_safe_labels(labels: &[&str]) -> Result<(), Error> {
    animus_adapter_safe::Record::new(
        animus_adapter_safe::Draft::new(
            animus_adapter_safe::Status::Completed,
            animus_adapter_safe::IssueCode::ProvenanceObservation,
            animus_adapter_safe::Operation::ProvenanceObservation,
        )
        .with_labels(labels)
        .with_redactions(&[animus_adapter_safe::Redaction::PayloadHashOnly]),
    )
    .map(|_record| ())
    .map_err(Error::UnsafeMetadata)
}

struct SafeLabelEnvelope<'a> {
    schema: &'static str,
    name: &'a str,
    labels: &'a [&'a str],
}

fn validate_safe_label_envelope(envelope: &SafeLabelEnvelope<'_>) -> Result<(), Error> {
    let mut combined = alloc::vec::Vec::with_capacity(
        envelope
            .labels
            .len()
            .saturating_add(SAFE_LABEL_PREFIX_COUNT),
    );
    combined.push(envelope.schema);
    combined.push(OBSERVABILITY_OPERATION_LABEL);
    combined.push(envelope.name);
    combined.extend_from_slice(envelope.labels);
    validate_safe_labels(combined.as_slice())
}

fn label_strings(labels: &[MetricLabel]) -> alloc::vec::Vec<alloc::string::String> {
    labels.iter().map(MetricLabel::joined).collect()
}

fn label_refs(labels: &[alloc::string::String]) -> alloc::vec::Vec<&str> {
    labels.iter().map(alloc::string::String::as_str).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const METRIC_NAME: &str = "runtime.events";
    const LOG_TARGET: &str = "runtime.event";
    const EVENT_KIND_KEY: &str = "event_kind";
    const EVENT_KIND_VALUE: &str = "runtime_started";
    const STATUS_KEY: &str = "status";
    const STATUS_VALUE: &str = "completed";
    const METRIC_VALUE: u64 = 1;
    const MESSAGE: &str = "runtime_started";
    const EMPTY_NAME: &str = "";
    const UNSAFE_VALUE: &str = "token=secret";
    const PRIVATE_PATH: &str = "/home/example/private.txt";

    #[test]
    fn dtos_accept_safe_metrics_logs_and_sink_outcomes() -> Result<(), Error> {
        let labels = [
            MetricLabelDraft::new(EVENT_KIND_KEY, EVENT_KIND_VALUE),
            MetricLabelDraft::new(STATUS_KEY, STATUS_VALUE),
        ];
        let metric = MetricSample::new(MetricSampleDraft {
            name: METRIC_NAME,
            kind: MetricKind::Counter,
            unit: MetricUnit::Count,
            value: METRIC_VALUE,
            labels: &labels,
        })?;
        assert_eq!(metric.name(), METRIC_NAME);
        assert_eq!(metric.kind(), MetricKind::Counter);
        assert_eq!(metric.labels().len(), labels.len());

        let log = LogRecord::new(LogRecordDraft::new(
            LOG_TARGET,
            LogLevel::Info,
            MESSAGE,
            &labels,
        ))?;
        assert_eq!(log.target(), LOG_TARGET);
        assert_eq!(log.level(), LogLevel::Info);
        assert_eq!(log.message(), MESSAGE);

        assert!(SinkOutcome::Delivered.is_delivered());
        let failed = SinkOutcome::failed(SinkFailureClass::Unavailable, "sink_unavailable")?;
        assert!(!failed.is_delivered());
        assert!(evaluate_delivery(DeliveryPolicy::BestEffort, &failed).allows_dispatch());
        assert!(!evaluate_delivery(DeliveryPolicy::AuditRequired, &failed).allows_dispatch());
        Ok(())
    }

    #[test]
    fn dtos_reject_empty_names_unsafe_labels_and_unbounded_messages() {
        let labels = [MetricLabelDraft::new(EVENT_KIND_KEY, EVENT_KIND_VALUE)];
        assert_eq!(
            MetricSample::new(MetricSampleDraft {
                name: EMPTY_NAME,
                kind: MetricKind::Counter,
                unit: MetricUnit::Count,
                value: METRIC_VALUE,
                labels: &labels,
            }),
            Err(Error::EmptyName)
        );
        assert!(matches!(
            MetricSample::new(MetricSampleDraft {
                name: METRIC_NAME,
                kind: MetricKind::Gauge,
                unit: MetricUnit::Count,
                value: METRIC_VALUE,
                labels: &[MetricLabelDraft::new(EVENT_KIND_KEY, UNSAFE_VALUE)],
            }),
            Err(Error::UnsafeMetadata(_))
        ));
        assert!(matches!(
            LogRecord::new(LogRecordDraft::new(
                LOG_TARGET,
                LogLevel::Error,
                PRIVATE_PATH,
                &labels,
            )),
            Err(Error::UnsafeMetadata(_))
        ));
        assert_eq!(
            LogRecord::new(LogRecordDraft::new(
                LOG_TARGET,
                LogLevel::Warning,
                MESSAGE,
                &[]
            )),
            Err(Error::InvalidLabelCount)
        );
    }
}
