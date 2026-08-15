const FATAL_SORT_RANK: u8 = 0;
const WARNING_SORT_RANK: u8 = 1;
const HEALTHY_SORT_RANK: u8 = 2;
const REDACTED_MARKER: &str = "[redacted]";

pub const CHECK_CONFIG: &str = "config";
pub const CHECK_STORE: &str = "store";
pub const CHECK_WORKFLOWS: &str = "workflows";
pub const CHECK_CREDENTIALS: &str = "credentials";
pub const CHECK_WEBHOOK_LISTENER: &str = "webhook_listener";
pub const CHECK_OPERATION_CONTRACTS: &str = "operation_contracts";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Healthy,
    Warning,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Healthy,
    Warning,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub id: String,
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
    pub redactions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub checks: Vec<Fact>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub status: Status,
    pub findings: Vec<Finding>,
}

impl Fact {
    pub fn healthy(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, Severity::Healthy, message)
    }

    #[cfg(test)]
    pub fn warning(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, Severity::Warning, message)
    }

    pub fn fatal(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, Severity::Fatal, message)
    }

    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    #[cfg(test)]
    pub fn with_redactions<I, S>(mut self, redactions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.redactions = redactions.into_iter().map(Into::into).collect();
        self
    }

    fn new(id: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity,
            message: message.into(),
            remediation: None,
            redactions: Vec::new(),
        }
    }
}

impl Facts {
    pub fn new(checks: Vec<Fact>) -> Self {
        Self { checks }
    }
}

pub fn evaluate(facts: &Facts) -> Report {
    let mut findings = facts.checks.iter().map(finding_from_fact).collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        severity_sort_rank(left.severity)
            .cmp(&severity_sort_rank(right.severity))
            .then_with(|| left.id.cmp(&right.id))
    });
    Report {
        status: report_status(&findings),
        findings,
    }
}

pub fn render_text(report: &Report) -> String {
    let mut output = format!("doctor status={}\n", status_label(report.status));
    for finding in &report.findings {
        output.push_str(&format!("{} {}: {}\n", severity_label(finding.severity), finding.id, finding.message));
        if let Some(remediation) = &finding.remediation {
            output.push_str(&format!("  hint: {remediation}\n"));
        }
    }
    output
}

pub fn render_json(report: &Report) -> String {
    match serde_json::to_string_pretty(report) {
        Ok(json) => json,
        Err(error) => format!(
            "{{\"status\":\"fatal\",\"findings\":[{{\"id\":\"doctor_output\",\"severity\":\"fatal\",\"message\":\"json render failed: {error}\",\"remediation\":null}}]}}"
        ),
    }
}

pub fn has_fatal(report: &Report) -> bool {
    report.status == Status::Fatal
}

fn finding_from_fact(fact: &Fact) -> Finding {
    Finding {
        id: sanitize_text(&fact.id, &fact.redactions),
        severity: fact.severity,
        message: sanitize_text(&fact.message, &fact.redactions),
        remediation: fact.remediation.as_ref().map(|remediation| sanitize_text(remediation, &fact.redactions)),
    }
}

fn report_status(findings: &[Finding]) -> Status {
    if findings.iter().any(|finding| finding.severity == Severity::Fatal) {
        Status::Fatal
    } else if findings.iter().any(|finding| finding.severity == Severity::Warning) {
        Status::Warning
    } else {
        Status::Healthy
    }
}

fn sanitize_text(input: &str, redactions: &[String]) -> String {
    let mut normalized =
        redactions.iter().map(String::as_str).filter(|redaction| !redaction.is_empty()).collect::<Vec<_>>();
    normalized.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    normalized.dedup();

    let mut output = input.to_string();
    for redaction in normalized {
        output = output.replace(redaction, REDACTED_MARKER);
    }
    output
}

fn severity_sort_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Fatal => FATAL_SORT_RANK,
        Severity::Warning => WARNING_SORT_RANK,
        Severity::Healthy => HEALTHY_SORT_RANK,
    }
}

fn status_label(status: Status) -> &'static str {
    match status {
        Status::Healthy => "healthy",
        Status::Warning => "warning",
        Status::Fatal => "fatal",
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Healthy => "healthy",
        Severity::Warning => "warning",
        Severity::Fatal => "fatal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEALTHY_CONFIG_MESSAGE: &str = "config loaded";
    const HEALTHY_STORE_MESSAGE: &str = "store opened";
    const FATAL_CONFIG_MESSAGE: &str = "config file not found";
    const FATAL_NICKEL_MESSAGE: &str = "Nickel evaluation failed";
    const FATAL_STORE_MESSAGE: &str = "store open failed";
    const FATAL_CREDENTIAL_MESSAGE: &str = "credential provider failed";
    const FATAL_LISTENER_MESSAGE: &str = "listener readiness failed";
    const FIX_CONFIG_HINT: &str = "fix config";
    const FIX_NICKEL_HINT: &str = "fix Nickel config";
    const FIX_STORE_HINT: &str = "repair store";
    const FIX_CREDENTIAL_HINT: &str = "configure provider";
    const FIX_LISTENER_HINT: &str = "check listener bind";
    const MIXED_FATAL_MESSAGE: &str = "config failed";
    const MIXED_WARNING_MESSAGE: &str = "listener degraded";
    const MIXED_HEALTHY_MESSAGE: &str = "workflows valid";
    const PLAINTEXT_SECRET: &str = "doctor-secret-token";
    const ENVIRONMENT_VALUE: &str = "doctor-environment-value";
    const AUTHORIZATION_HEADER: &str = "Bearer doctor-authorization-value";
    const RAW_PAYLOAD: &str = r#"{"private":"doctor-payload-value"}"#;
    const PROVIDER_OUTPUT: &str = "doctor-provider-output";
    const OVERLAPPING_SHORT_VALUE: &str = "doctor-overlap";
    const OVERLAPPING_LONG_VALUE: &str = "doctor-overlap-sensitive-suffix";

    fn mixed_report() -> Report {
        evaluate(&Facts::new(vec![
            Fact::healthy(CHECK_WORKFLOWS, MIXED_HEALTHY_MESSAGE),
            Fact::warning(CHECK_WEBHOOK_LISTENER, MIXED_WARNING_MESSAGE).with_remediation(FIX_LISTENER_HINT),
            Fact::fatal(CHECK_CONFIG, MIXED_FATAL_MESSAGE).with_remediation(FIX_CONFIG_HINT),
        ]))
    }

    fn fatal_case(fact: Fact, expected: Finding) -> (Fact, Finding) {
        (fact, expected)
    }

    // r[verify n8nclone.operator_diagnostics.doctor_core]
    // r[verify n8nclone.operator_diagnostics.doctor_core.healthy]
    #[test]
    fn healthy_facts_produce_a_deterministic_healthy_report() {
        let facts = Facts::new(vec![
            Fact::healthy(CHECK_STORE, HEALTHY_STORE_MESSAGE),
            Fact::healthy(CHECK_CONFIG, HEALTHY_CONFIG_MESSAGE),
        ]);
        let expected = Report {
            status: Status::Healthy,
            findings: vec![
                Finding {
                    id: CHECK_CONFIG.to_string(),
                    severity: Severity::Healthy,
                    message: HEALTHY_CONFIG_MESSAGE.to_string(),
                    remediation: None,
                },
                Finding {
                    id: CHECK_STORE.to_string(),
                    severity: Severity::Healthy,
                    message: HEALTHY_STORE_MESSAGE.to_string(),
                    remediation: None,
                },
            ],
        };

        let report = evaluate(&facts);

        assert_eq!(report, expected);
        assert_eq!(evaluate(&facts), report);
        assert!(!has_fatal(&report));
    }

    // r[verify n8nclone.operator_diagnostics.doctor_core.fatal]
    #[test]
    fn every_enumerated_failing_fact_produces_a_stable_safe_finding() {
        let cases = vec![
            fatal_case(Fact::fatal(CHECK_CONFIG, FATAL_CONFIG_MESSAGE).with_remediation(FIX_CONFIG_HINT), Finding {
                id: CHECK_CONFIG.to_string(),
                severity: Severity::Fatal,
                message: FATAL_CONFIG_MESSAGE.to_string(),
                remediation: Some(FIX_CONFIG_HINT.to_string()),
            }),
            fatal_case(Fact::fatal(CHECK_CONFIG, FATAL_NICKEL_MESSAGE).with_remediation(FIX_NICKEL_HINT), Finding {
                id: CHECK_CONFIG.to_string(),
                severity: Severity::Fatal,
                message: FATAL_NICKEL_MESSAGE.to_string(),
                remediation: Some(FIX_NICKEL_HINT.to_string()),
            }),
            fatal_case(Fact::fatal(CHECK_STORE, FATAL_STORE_MESSAGE).with_remediation(FIX_STORE_HINT), Finding {
                id: CHECK_STORE.to_string(),
                severity: Severity::Fatal,
                message: FATAL_STORE_MESSAGE.to_string(),
                remediation: Some(FIX_STORE_HINT.to_string()),
            }),
            fatal_case(
                Fact::fatal(CHECK_CREDENTIALS, FATAL_CREDENTIAL_MESSAGE).with_remediation(FIX_CREDENTIAL_HINT),
                Finding {
                    id: CHECK_CREDENTIALS.to_string(),
                    severity: Severity::Fatal,
                    message: FATAL_CREDENTIAL_MESSAGE.to_string(),
                    remediation: Some(FIX_CREDENTIAL_HINT.to_string()),
                },
            ),
            fatal_case(
                Fact::fatal(CHECK_WEBHOOK_LISTENER, FATAL_LISTENER_MESSAGE).with_remediation(FIX_LISTENER_HINT),
                Finding {
                    id: CHECK_WEBHOOK_LISTENER.to_string(),
                    severity: Severity::Fatal,
                    message: FATAL_LISTENER_MESSAGE.to_string(),
                    remediation: Some(FIX_LISTENER_HINT.to_string()),
                },
            ),
        ];

        for (fact, expected) in cases {
            let facts = Facts::new(vec![fact]);
            let report = evaluate(&facts);
            assert_eq!(report.status, Status::Fatal);
            assert_eq!(report.findings, vec![expected]);
            assert!(has_fatal(&report));
            assert_eq!(evaluate(&facts), report);
        }
    }

    // r[verify n8nclone.operator_diagnostics.doctor_output]
    // r[verify n8nclone.operator_diagnostics.doctor_output.text]
    #[test]
    fn mixed_findings_render_as_exact_deterministic_actionable_text() {
        let report = mixed_report();
        let expected = concat!(
            "doctor status=fatal\n",
            "fatal config: config failed\n",
            "  hint: fix config\n",
            "warning webhook_listener: listener degraded\n",
            "  hint: check listener bind\n",
            "healthy workflows: workflows valid\n",
        );

        let rendered = render_text(&report);

        assert_eq!(rendered, expected);
        assert_eq!(render_text(&report), rendered);
    }

    // r[verify n8nclone.operator_diagnostics.doctor_output.json]
    #[test]
    fn mixed_findings_render_as_exact_deterministic_automation_safe_json() {
        let report = mixed_report();
        let expected = serde_json::json!({
            "status": "fatal",
            "findings": [
                {
                    "id": "config",
                    "severity": "fatal",
                    "message": "config failed",
                    "remediation": "fix config"
                },
                {
                    "id": "webhook_listener",
                    "severity": "warning",
                    "message": "listener degraded",
                    "remediation": "check listener bind"
                },
                {
                    "id": "workflows",
                    "severity": "healthy",
                    "message": "workflows valid",
                    "remediation": null
                }
            ]
        });

        let rendered = render_json(&report);
        let decoded: serde_json::Value = serde_json::from_str(&rendered).expect("doctor JSON must decode");

        assert_eq!(decoded, expected);
        assert_eq!(render_json(&report), rendered);
    }

    // r[verify n8nclone.operator_diagnostics.doctor_redaction]
    // r[verify n8nclone.operator_diagnostics.doctor_redaction.text]
    // r[verify n8nclone.operator_diagnostics.doctor_redaction.json]
    #[test]
    fn sensitive_fact_values_are_absent_from_text_and_json() {
        let sensitive_values = [
            PLAINTEXT_SECRET,
            ENVIRONMENT_VALUE,
            AUTHORIZATION_HEADER,
            RAW_PAYLOAD,
            PROVIDER_OUTPUT,
            OVERLAPPING_SHORT_VALUE,
            OVERLAPPING_LONG_VALUE,
        ];
        let message = sensitive_values.join(" | ");
        let remediation = format!("remove {OVERLAPPING_LONG_VALUE} and {PLAINTEXT_SECRET}");
        let sensitive_id = format!("{CHECK_CREDENTIALS}:{PLAINTEXT_SECRET}");
        let facts = Facts::new(vec![
            Fact::warning(sensitive_id, message).with_remediation(remediation).with_redactions(sensitive_values),
        ]);

        let report = evaluate(&facts);
        let text = render_text(&report);
        let json = render_json(&report);

        assert_eq!(report.status, Status::Warning);
        assert_eq!(report.findings.len(), 1);
        assert!(text.contains(REDACTED_MARKER));
        assert!(json.contains(REDACTED_MARKER));
        assert!(!json.contains("redactions"));
        for sensitive_fragment in [
            PLAINTEXT_SECRET,
            ENVIRONMENT_VALUE,
            "doctor-authorization-value",
            "doctor-payload-value",
            PROVIDER_OUTPUT,
            "sensitive-suffix",
        ] {
            assert!(!text.contains(sensitive_fragment));
            assert!(!json.contains(sensitive_fragment));
        }
    }
}
