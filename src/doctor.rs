//! Operator diagnostics pure core and presenters.

use serde::Deserialize;
use serde::Serialize;

const FATAL_SORT_RANK: u8 = 0;
const WARNING_SORT_RANK: u8 = 1;
const HEALTHY_SORT_RANK: u8 = 2;
const REDACTED_MARKER: &str = "[redacted]";

pub const CHECK_CONFIG: &str = "config";
pub const CHECK_STORE: &str = "store";
pub const CHECK_WORKFLOWS: &str = "workflows";
pub const CHECK_CREDENTIALS: &str = "credentials";
pub const CHECK_WEBHOOK_LISTENER: &str = "webhook_listener";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorSeverity {
    Healthy,
    Warning,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorStatus {
    Healthy,
    Warning,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorFact {
    pub id: String,
    pub severity: DoctorSeverity,
    pub message: String,
    pub remediation: Option<String>,
    pub redactions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorFacts {
    pub checks: Vec<DoctorFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorFinding {
    pub id: String,
    pub severity: DoctorSeverity,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub status: DoctorStatus,
    pub findings: Vec<DoctorFinding>,
}

impl DoctorFact {
    pub fn healthy(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, DoctorSeverity::Healthy, message)
    }

    pub fn warning(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, DoctorSeverity::Warning, message)
    }

    pub fn fatal(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, DoctorSeverity::Fatal, message)
    }

    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    pub fn with_redaction(mut self, redaction: impl Into<String>) -> Self {
        self.redactions.push(redaction.into());
        self
    }

    fn new(id: impl Into<String>, severity: DoctorSeverity, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity,
            message: message.into(),
            remediation: None,
            redactions: Vec::new(),
        }
    }
}

impl DoctorFacts {
    pub fn new(checks: Vec<DoctorFact>) -> Self {
        Self { checks }
    }
}

// r[impl n8nclone.operator_diagnostics.doctor_core]
pub fn evaluate_doctor_facts(facts: &DoctorFacts) -> DoctorReport {
    let mut findings = facts.checks.iter().map(finding_from_fact).collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        severity_sort_rank(left.severity)
            .cmp(&severity_sort_rank(right.severity))
            .then_with(|| left.id.cmp(&right.id))
    });
    DoctorReport {
        status: report_status(&findings),
        findings,
    }
}

// r[impl n8nclone.operator_diagnostics.doctor_output]
pub fn render_doctor_text(report: &DoctorReport) -> String {
    let mut output = format!("doctor status={}\n", status_label(report.status));
    for finding in &report.findings {
        output.push_str(&format!("{} {}: {}\n", severity_label(finding.severity), finding.id, finding.message));
        if let Some(remediation) = &finding.remediation {
            output.push_str(&format!("  hint: {remediation}\n"));
        }
    }
    output
}

// r[impl n8nclone.operator_diagnostics.doctor_output]
pub fn render_doctor_json(report: &DoctorReport) -> String {
    serde_json::to_string_pretty(report).expect("doctor report serializes")
}

pub fn doctor_has_fatal(report: &DoctorReport) -> bool {
    report.status == DoctorStatus::Fatal
}

fn finding_from_fact(fact: &DoctorFact) -> DoctorFinding {
    DoctorFinding {
        id: fact.id.clone(),
        severity: fact.severity,
        message: sanitize_text(&fact.message, &fact.redactions),
        remediation: fact.remediation.as_ref().map(|remediation| sanitize_text(remediation, &fact.redactions)),
    }
}

fn report_status(findings: &[DoctorFinding]) -> DoctorStatus {
    if findings.iter().any(|finding| finding.severity == DoctorSeverity::Fatal) {
        DoctorStatus::Fatal
    } else if findings.iter().any(|finding| finding.severity == DoctorSeverity::Warning) {
        DoctorStatus::Warning
    } else {
        DoctorStatus::Healthy
    }
}

fn sanitize_text(input: &str, redactions: &[String]) -> String {
    let mut output = input.to_string();
    for redaction in redactions {
        if !redaction.is_empty() {
            output = output.replace(redaction, REDACTED_MARKER);
        }
    }
    output
}

fn severity_sort_rank(severity: DoctorSeverity) -> u8 {
    match severity {
        DoctorSeverity::Fatal => FATAL_SORT_RANK,
        DoctorSeverity::Warning => WARNING_SORT_RANK,
        DoctorSeverity::Healthy => HEALTHY_SORT_RANK,
    }
}

fn status_label(status: DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Healthy => "healthy",
        DoctorStatus::Warning => "warning",
        DoctorStatus::Fatal => "fatal",
    }
}

fn severity_label(severity: DoctorSeverity) -> &'static str {
    match severity {
        DoctorSeverity::Healthy => "healthy",
        DoctorSeverity::Warning => "warning",
        DoctorSeverity::Fatal => "fatal",
    }
}
