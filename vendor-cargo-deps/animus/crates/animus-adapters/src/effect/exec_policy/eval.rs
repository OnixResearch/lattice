/// Borrowed exec-policy evaluation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    /// Command intent selected by the runtime or process adapter.
    pub command: &'a super::CommandIntent,
    /// Active sandbox context for the escalation request.
    pub sandbox: &'a super::SandboxContext,
    /// Validated policy rules selected by the host.
    pub rules: &'a [super::Rule],
}

/// Redacted evidence for a policy rule matched during evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleEvidence {
    rule_id: super::Text,
    decision: super::Decision,
    rationale: super::Text,
    command_fingerprint: super::Text,
}

impl RuleEvidence {
    fn new(rule: &super::Rule, intent: &super::CommandIntent) -> Self {
        Self {
            rule_id: super::Text::from(rule.id()),
            decision: rule.decision(),
            rationale: super::Text::from(rule.rationale()),
            command_fingerprint: super::Text::from(intent.fingerprint()),
        }
    }

    /// Returns the matched rule identity.
    #[must_use]
    pub fn rule_id(&self) -> &str {
        self.rule_id.as_str()
    }

    /// Returns the matched rule decision.
    #[must_use]
    pub const fn decision(&self) -> super::Decision {
        self.decision
    }

    /// Returns the safe rule rationale.
    #[must_use]
    pub fn rationale(&self) -> &str {
        self.rationale.as_str()
    }

    /// Returns the BLAKE3 fingerprint of the command that matched the rule.
    #[must_use]
    pub fn command_fingerprint(&self) -> &str {
        self.command_fingerprint.as_str()
    }
}

/// Exec-policy evaluation result with redacted command evidence only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    decision: super::Decision,
    matched_rules: super::Collection<RuleEvidence>,
    command_fingerprints: super::Collection<super::Text>,
    sandbox: super::SandboxContext,
    shell_analysis: super::AnalysisKind,
    redaction_marker: &'static str,
}

impl Evaluation {
    /// Returns the final most-restrictive policy decision.
    #[must_use]
    pub const fn decision(&self) -> super::Decision {
        self.decision
    }

    /// Returns matched-rule evidence.
    #[must_use]
    pub fn matched_rules(&self) -> &[RuleEvidence] {
        self.matched_rules.as_slice()
    }

    /// Returns command fingerprints evaluated by policy.
    #[must_use]
    pub fn command_fingerprints(&self) -> &[super::Text] {
        self.command_fingerprints.as_slice()
    }

    /// Returns the sandbox escalation context.
    #[must_use]
    pub const fn sandbox(&self) -> &super::SandboxContext {
        &self.sandbox
    }

    /// Returns the shell-wrapper analysis classification.
    #[must_use]
    pub const fn shell_analysis(&self) -> super::AnalysisKind {
        self.shell_analysis
    }

    /// Returns the redaction marker for command evidence.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }
}

/// Evaluates exec policy as a pure function over command intent and sandbox context.
#[must_use]
pub fn evaluate(request: Request<'_>) -> Evaluation {
    let analysis = super::analyze_wrapper(request.command);
    let intent_count = analysis.intents().len();
    let matched_rule_slots = intent_count.saturating_mul(request.rules.len());
    assert!(intent_count > 0, "evaluation must inspect at least one intent");
    assert!(intent_count <= super::MAX_COMMAND_WORDS, "evaluated intent count must fit command bound");
    let mut final_decision = super::Decision::Allow;
    let mut matched_rules = super::Collection::with_capacity(matched_rule_slots);
    let mut command_fingerprints = super::Collection::with_capacity(intent_count);

    for intent in analysis.intents() {
        command_fingerprints.push(super::Text::from(intent.fingerprint()));
        let intent_decision = evaluate_intent(intent, request.rules, &mut matched_rules);
        final_decision = final_decision.most_restrictive(intent_decision);
    }

    Evaluation {
        decision: final_decision,
        matched_rules,
        command_fingerprints,
        sandbox: request.sandbox.clone(),
        shell_analysis: analysis.kind(),
        redaction_marker: super::REDACTION_COMMAND_HASH_ONLY,
    }
}

fn evaluate_intent(
    intent: &super::CommandIntent,
    rules: &[super::Rule],
    matched_rules: &mut super::Collection<RuleEvidence>,
) -> super::Decision {
    let mut decision = None;
    for rule in rules {
        if rule.matches_intent(intent) {
            matched_rules.push(RuleEvidence::new(rule, intent));
            decision = Some(
                decision.map_or(rule.decision(), |current: super::Decision| current.most_restrictive(rule.decision())),
            );
        }
    }
    decision.unwrap_or(super::Decision::Forbid)
}
