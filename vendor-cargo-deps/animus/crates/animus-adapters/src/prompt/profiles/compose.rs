/// Admission policy for profile assembly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SourceAdmission {
    /// Only trusted sources may contribute model-visible prompt text.
    #[default]
    TrustedOnly,
    /// Untrusted sources may be admitted by an explicit host policy.
    AdmitUntrusted,
}

/// Policy input for deterministic profile assembly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AssemblyPolicy {
    /// Source admission behavior for untrusted profiles.
    pub source_admission: SourceAdmission,
}

/// Model-visible segment produced by profile assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    profile_id: super::support::Text,
    source_identity: super::support::Text,
    posture: super::types::Mode,
    text: super::support::Text,
}

impl Segment {
    /// Returns the profile that produced this segment.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the safe source identity label.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the requested posture metadata.
    #[must_use]
    pub const fn posture(&self) -> super::types::Mode {
        self.posture
    }

    /// Returns model-visible profile text after directive stripping.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Safe reason a profile did not contribute model-visible text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmittedReason {
    /// Source trust was not admitted by assembly policy.
    UntrustedSource,
    /// Profile requires a separate host admission step.
    PendingAdmission,
    /// Inclusion policy forbids model-visible text.
    InclusionForbidden,
    /// Directive syntax was invalid, so the profile failed closed.
    InvalidDirective,
}

/// Diagnostic emitted during profile assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssemblyDiagnostic {
    profile_id: super::support::Text,
    reason: OmittedReason,
}

impl AssemblyDiagnostic {
    /// Returns the profile that triggered this diagnostic.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the safe omission reason.
    #[must_use]
    pub const fn reason(&self) -> OmittedReason {
        self.reason
    }
}

/// Deterministic profile assembly report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assembly {
    segments: super::support::Collection<Segment>,
    diagnostics: super::support::Collection<AssemblyDiagnostic>,
    receipts: super::support::Collection<super::evidence::Receipt>,
}

impl Assembly {
    /// Returns model-visible segments in precedence order.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        self.segments.as_slice()
    }

    /// Returns safe assembly diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[AssemblyDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns bounded safe receipts for all evaluated profiles.
    #[must_use]
    pub fn receipts(&self) -> &[super::evidence::Receipt] {
        self.receipts.as_slice()
    }
}

/// Parses and assembles profiles into model-visible prompt segments.
#[must_use]
pub fn assemble(
    profiles: &[super::types::Descriptor],
    policy: AssemblyPolicy,
    supported_chain_targets: &[super::types::Mode],
) -> Assembly {
    assert!(profiles.len() <= super::support::MAX_ASSEMBLY_PROFILES);
    assert!(supported_chain_targets.len() <= super::support::MAX_CHAIN_TARGETS);
    let mut ordered = profiles.to_vec();
    ordered.sort_by(|left, right| {
        left.precedence()
            .cmp(&right.precedence())
            .then_with(|| left.profile_id().cmp(right.profile_id()))
    });

    assemble_ordered(&ordered, policy, supported_chain_targets)
}

fn assemble_ordered(
    ordered: &[super::types::Descriptor],
    policy: AssemblyPolicy,
    supported_chain_targets: &[super::types::Mode],
) -> Assembly {
    let profile_count = ordered.len();
    let mut segments = super::support::Collection::with_capacity(profile_count);
    let mut diagnostics = super::support::Collection::with_capacity(profile_count);
    let mut receipts = super::support::Collection::with_capacity(profile_count);

    for profile in ordered {
        let evaluated = evaluate_profile(profile, policy, supported_chain_targets);
        receipts.push(evaluated.receipt);
        match evaluated.outcome {
            EvaluationOutcome::Segment(segment) => segments.push(segment),
            EvaluationOutcome::Diagnostic(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    Assembly {
        segments,
        diagnostics,
        receipts,
    }
}

struct EvaluatedProfile {
    receipt: super::evidence::Receipt,
    outcome: EvaluationOutcome,
}

enum EvaluationOutcome {
    Segment(Segment),
    Diagnostic(AssemblyDiagnostic),
}

fn evaluate_profile(
    profile: &super::types::Descriptor,
    policy: AssemblyPolicy,
    supported_chain_targets: &[super::types::Mode],
) -> EvaluatedProfile {
    let parsed_directives = super::directives::parse(profile.body(), supported_chain_targets);
    let omission = profile_omission(profile, policy, &parsed_directives);
    let receipt =
        super::evidence::Receipt::from_profile(profile, &parsed_directives, None, omission);
    let outcome = match omission {
        Some(reason) => EvaluationOutcome::Diagnostic(AssemblyDiagnostic {
            profile_id: super::support::Text::from(profile.profile_id()),
            reason,
        }),
        None => EvaluationOutcome::Segment(segment(profile, &parsed_directives)),
    };

    EvaluatedProfile { receipt, outcome }
}

fn segment(
    profile: &super::types::Descriptor,
    parsed_directives: &super::directives::Report,
) -> Segment {
    Segment {
        profile_id: super::support::Text::from(profile.profile_id()),
        source_identity: super::support::Text::from(profile.source_identity()),
        posture: parsed_directives.mode().unwrap_or(profile.action_posture()),
        text: super::support::Text::from(parsed_directives.visible_body()),
    }
}

fn profile_omission(
    profile: &super::types::Descriptor,
    policy: AssemblyPolicy,
    parsed_directives: &super::directives::Report,
) -> Option<OmittedReason> {
    assert!(profile.precedence() >= super::support::MIN_PRECEDENCE);
    assert!(profile.precedence() <= super::support::MAX_PRECEDENCE);
    if parsed_directives.has_blocking_diagnostics() {
        return Some(OmittedReason::InvalidDirective);
    }

    match profile.inclusion_policy() {
        super::types::InclusionPolicy::IncludeWhenTrusted => source_omission(profile, policy),
        super::types::InclusionPolicy::WithholdPendingAdmission => {
            Some(OmittedReason::PendingAdmission)
        }
        super::types::InclusionPolicy::NeverInclude => Some(OmittedReason::InclusionForbidden),
    }
}

fn source_omission(
    profile: &super::types::Descriptor,
    policy: AssemblyPolicy,
) -> Option<OmittedReason> {
    match (profile.source_trust(), policy.source_admission) {
        (super::types::SourceTrust::Trusted, _) | (_, SourceAdmission::AdmitUntrusted) => None,
        (super::types::SourceTrust::Untrusted, SourceAdmission::TrustedOnly) => {
            Some(OmittedReason::UntrustedSource)
        }
    }
}
