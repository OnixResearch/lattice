pub(super) fn context(
    profiles: &[super::Descriptor],
    project_trust_mode: super::ProjectTrustMode,
) -> super::CompositionReport {
    let profile_count = profiles.len();
    let mut ordered: super::Collection<&super::Descriptor> = profiles.iter().collect();
    assert_eq!(ordered.len(), profile_count, "all profiles are staged for ordering");
    ordered.sort_by(|left, right| {
        left.precedence_rank()
            .cmp(&right.precedence_rank())
            .then_with(|| right.priority.cmp(&left.priority))
            .then_with(|| left.provenance.cmp(&right.provenance))
    });

    let has_global_destructive_guard = ordered.iter().any(|profile| {
        profile.scope == super::SourceScope::Global && has_constraint(profile, ConstraintMarker::RequireApproval)
    });
    let mut segments = super::Collection::with_capacity(profile_count);
    let mut diagnostics = super::Collection::with_capacity(profile_count);
    for profile in ordered {
        if project_policy_is_omitted(profile, project_trust_mode) {
            diagnostics.push(diagnostic(super::DiagnosticKind::ProjectPolicyOmitted, profile.provenance()));
            continue;
        }
        if project_destructive_allow_is_overridden(profile, has_global_destructive_guard) {
            diagnostics.push(diagnostic(super::DiagnosticKind::GlobalSafetyPrecedence, profile.provenance()));
            continue;
        }
        segments.push(prompt_segment(profile));
    }
    assert!(
        segments.len().saturating_add(diagnostics.len()) <= profile_count,
        "composition output stays within input cardinality"
    );
    super::CompositionReport { segments, diagnostics }
}

/// Returns the runtime guard requirement for a policy-described mutation request.
#[must_use]
pub fn mutation_requires_runtime_guard(descriptor: &super::Descriptor) -> super::RuntimeGuardRequirement {
    if descriptor.validates_as_read_only_guidance() {
        return super::RuntimeGuardRequirement::Required;
    }
    super::RuntimeGuardRequirement::Required
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConstraintMarker {
    RequireApproval,
    AllowDestructive,
}

impl ConstraintMarker {
    const fn as_str(self) -> &'static str {
        match self {
            Self::RequireApproval => super::REQUIRE_APPROVAL_CONSTRAINT,
            Self::AllowDestructive => super::ALLOW_DESTRUCTIVE_CONSTRAINT,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PromptSection {
    Goal,
    Success,
    Constraints,
    Validation,
    Stop,
    Snippets,
}

impl PromptSection {
    const fn label(self) -> &'static str {
        match self {
            Self::Goal => "Goal",
            Self::Success => "Success",
            Self::Constraints => "Constraints",
            Self::Validation => "Validation",
            Self::Stop => "Stop",
            Self::Snippets => "Snippets",
        }
    }
}

struct PromptText {
    value: super::Text,
}

impl PromptText {
    fn new() -> Self {
        Self {
            value: super::Text::new(),
        }
    }

    fn push_section(&mut self, section: PromptSection, value: &str) {
        if !self.value.is_empty() {
            self.value.push_str(super::PROMPT_SECTION_SEPARATOR);
        }
        self.value.push_str(section.label());
        self.value.push_str(": ");
        self.value.push_str(value);
    }

    fn push_items(&mut self, section: PromptSection, values: &[super::Text]) {
        if values.is_empty() {
            return;
        }
        self.push_section(section, values[super::FIRST_PROMPT_ITEM_INDEX].as_str());
        for value in &values[super::REMAINING_PROMPT_ITEMS_START..] {
            self.value.push_str(super::PROMPT_ITEM_SEPARATOR);
            self.value.push_str(value);
        }
    }

    fn finish(self) -> super::Text {
        self.value
    }
}

fn project_policy_is_omitted(descriptor: &super::Descriptor, project_trust_mode: super::ProjectTrustMode) -> bool {
    descriptor.scope == super::SourceScope::Project
        && (descriptor.trust == super::TrustStatus::Untrusted
            || project_trust_mode == super::ProjectTrustMode::DenyProjectPolicy)
}

fn project_destructive_allow_is_overridden(descriptor: &super::Descriptor, has_global_destructive_guard: bool) -> bool {
    has_global_destructive_guard
        && descriptor.scope == super::SourceScope::Project
        && has_constraint(descriptor, ConstraintMarker::AllowDestructive)
}

fn prompt_segment(descriptor: &super::Descriptor) -> super::PromptSegment {
    let mut text = PromptText::new();
    text.push_section(PromptSection::Goal, descriptor.goal());
    text.push_items(PromptSection::Success, descriptor.success_criteria());
    text.push_items(PromptSection::Constraints, descriptor.constraints());
    text.push_items(PromptSection::Validation, descriptor.validation_rules());
    text.push_items(PromptSection::Stop, descriptor.stop_rules());
    text.push_items(PromptSection::Snippets, descriptor.prompt_snippets());
    super::PromptSegment {
        provenance: descriptor.provenance.clone(),
        text: text.finish(),
    }
}

fn diagnostic(kind: super::DiagnosticKind, provenance: &str) -> super::CompositionDiagnostic {
    super::CompositionDiagnostic {
        kind,
        provenance: super::Text::from(provenance),
    }
}

fn has_constraint(descriptor: &super::Descriptor, target: ConstraintMarker) -> bool {
    descriptor.constraints.iter().any(|constraint| constraint == target.as_str())
}
