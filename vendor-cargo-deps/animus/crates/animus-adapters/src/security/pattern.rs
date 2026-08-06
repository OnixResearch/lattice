struct Rule {
    patterns: &'static [&'static str],
    category: super::Category,
    confidence: super::Confidence,
    disposition: super::Disposition,
}

const RULES: &[Rule] = &[
    Rule {
        patterns: super::PROMPT_INJECTION_PATTERNS,
        category: super::Category::PromptInjection,
        confidence: super::Confidence::Medium,
        disposition: super::Disposition::Block,
    },
    Rule {
        patterns: super::DESTRUCTIVE_ACTION_PATTERNS,
        category: super::Category::DestructiveAction,
        confidence: super::Confidence::High,
        disposition: super::Disposition::Block,
    },
    Rule {
        patterns: super::EXFILTRATION_PATTERNS,
        category: super::Category::Exfiltration,
        confidence: super::Confidence::High,
        disposition: super::Disposition::Block,
    },
    Rule {
        patterns: super::PRIVILEGE_ESCALATION_PATTERNS,
        category: super::Category::PrivilegeEscalation,
        confidence: super::Confidence::High,
        disposition: super::Disposition::Block,
    },
    Rule {
        patterns: super::OBFUSCATED_REMOTE_EXECUTION_PATTERNS,
        category: super::Category::ObfuscatedRemoteExecution,
        confidence: super::Confidence::High,
        disposition: super::Disposition::Block,
    },
    Rule {
        patterns: super::OUT_OF_SCOPE_MUTATION_PATTERNS,
        category: super::Category::OutOfScopeMutation,
        confidence: super::Confidence::High,
        disposition: super::Disposition::Block,
    },
];

/// Runs deterministic security scanning over safe labels and bounded summaries.
#[must_use]
pub fn scan_review_input(input: &super::Review) -> super::Report {
    let searchable = searchable_text(input);
    let evidence_hash = input.primary_evidence_hash();
    let mut findings = super::Collection::new();
    for rule in RULES {
        push_if_matches(&mut findings, searchable.as_str(), rule, evidence_hash);
    }
    super::Report::complete(findings)
}

fn searchable_text(input: &super::Review) -> super::Text {
    let mut text = super::Text::new();
    push_text(&mut text, input.action_class().as_str());
    push_text(&mut text, input.risk_class().as_str());
    push_text(&mut text, input.tool().tool_name());
    push_text(&mut text, input.tool().target_label());
    for label in input.task_labels() {
        push_text(&mut text, label.as_str());
    }
    for summary in input.recent_context_summaries() {
        push_text(&mut text, summary.as_str());
    }
    for label in input.tool().safe_argument_labels() {
        push_text(&mut text, label.as_str());
    }
    for label in input.active_policy_labels() {
        push_text(&mut text, label.as_str());
    }
    debug_assert!(!input.tool().tool_name().is_empty());
    debug_assert!(!text.is_empty());
    text.to_ascii_lowercase()
}

fn push_text(text: &mut super::Text, value: &str) {
    if !text.is_empty() {
        text.push(super::SEARCH_SEPARATOR);
    }
    text.push_str(value);
}

fn push_if_matches(
    findings: &mut super::Collection<super::Finding>,
    searchable: &str,
    rule: &Rule,
    evidence_hash: &str,
) {
    if rule.patterns.iter().any(|pattern| searchable.contains(pattern)) {
        findings.push(super::Finding::new(rule.category, rule.confidence, rule.disposition, evidence_hash));
    }
}
