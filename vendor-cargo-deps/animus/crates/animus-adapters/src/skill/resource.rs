/// Model-visible skill summary that omits full body content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary {
    pub(super) name: super::Text,
    pub(super) description: super::Text,
    pub(super) category: super::Text,
    pub(super) tags: super::Collection<super::Text>,
    pub(super) provenance: super::Text,
}

impl Summary {
    /// Returns the skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe human-facing summary.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the host-provided category metadata.
    #[must_use]
    pub fn category(&self) -> &str {
        self.category.as_str()
    }

    /// Returns sorted tag labels.
    #[must_use]
    pub fn tags(&self) -> &[super::Text] {
        self.tags.as_slice()
    }

    /// Returns the safe provenance label.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }
}

/// Skill resource advertisement catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    summaries: super::Collection<Summary>,
    diagnostics: super::Collection<Diagnostic>,
}

impl Catalog {
    /// Returns deterministic skill summaries admitted for advertisement.
    #[must_use]
    pub fn summaries(&self) -> &[Summary] {
        self.summaries.as_slice()
    }

    /// Returns deterministic catalog diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Skill catalog issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// More than one manifest used the same skill name.
    DuplicateSkillName,
    /// A related skill name did not resolve to a supplied descriptor.
    UnresolvedRelatedSkill,
}

/// Safe diagnostic emitted while building skill resource summaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    issue: Issue,
    name: super::Text,
    related_skill: Option<super::Text>,
}

impl Diagnostic {
    /// Returns the diagnostic issue class.
    #[must_use]
    pub const fn issue(&self) -> Issue {
        self.issue
    }

    /// Returns the skill name associated with the issue.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the unresolved related skill, when present.
    #[must_use]
    pub fn related_skill(&self) -> Option<&str> {
        self.related_skill.as_deref()
    }
}

/// Builds deterministic resource summaries from already validated manifests.
///
/// # Panics
///
/// Panics when callers pass more manifests than the descriptor item bound.
#[must_use]
pub fn advertise(manifests: &[super::Descriptor]) -> Catalog {
    let manifest_count = manifests.len();
    assert!(
        manifest_count <= super::MAX_ITEMS,
        "manifest inputs stay within the descriptor item bound"
    );
    let diagnostic_slots_count = manifest_count.saturating_mul(super::MAX_ITEMS);
    assert!(
        diagnostic_slots_count >= manifest_count,
        "diagnostic capacity covers at least one issue per manifest"
    );
    let mut summaries = super::Collection::with_capacity(manifest_count);
    let mut diagnostics = super::Collection::with_capacity(diagnostic_slots_count);

    for manifest in manifests {
        push_related_diagnostics(manifest, manifests, &mut diagnostics);
        if duplicate_name(manifest.name(), manifests) {
            push_duplicate_diagnostic(manifest, &mut diagnostics);
            continue;
        }
        summaries.push(manifest.resource_summary());
    }

    summaries.sort_by(|left, right| left.name.cmp(&right.name));
    diagnostics.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.related_skill.cmp(&right.related_skill))
    });
    Catalog {
        summaries,
        diagnostics,
    }
}

fn push_related_diagnostics(
    manifest: &super::Descriptor,
    manifests: &[super::Descriptor],
    diagnostics: &mut super::Collection<Diagnostic>,
) {
    for related_skill in manifest.related_skills() {
        if !contains_manifest_name(manifests, related_skill.as_str()) {
            diagnostics.push(Diagnostic {
                issue: Issue::UnresolvedRelatedSkill,
                name: super::Text::from(manifest.name()),
                related_skill: Some(related_skill.clone()),
            });
        }
    }
}

fn push_duplicate_diagnostic(
    manifest: &super::Descriptor,
    diagnostics: &mut super::Collection<Diagnostic>,
) {
    if diagnostics.iter().any(|diagnostic| {
        diagnostic.issue == Issue::DuplicateSkillName
            && diagnostic.name == manifest.name()
            && diagnostic.related_skill.is_none()
    }) {
        return;
    }
    diagnostics.push(Diagnostic {
        issue: Issue::DuplicateSkillName,
        name: super::Text::from(manifest.name()),
        related_skill: None,
    });
}

pub(super) fn duplicate_name(name: &str, manifests: &[super::Descriptor]) -> bool {
    manifests
        .iter()
        .filter(|manifest| manifest.name() == name)
        .count()
        > 1
}

fn contains_manifest_name(manifests: &[super::Descriptor], name: &str) -> bool {
    manifests.iter().any(|manifest| manifest.name() == name)
}
