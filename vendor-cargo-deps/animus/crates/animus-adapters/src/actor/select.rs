/// Discovery report after deterministic precedence and duplicate-name handling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryReport {
    selected: super::Collection<super::Manifest>,
    diagnostics: super::Collection<Diagnostic>,
}

impl DiscoveryReport {
    /// Returns selected manifests in deterministic name order.
    #[must_use]
    pub fn selected(&self) -> &[super::Manifest] {
        self.selected.as_slice()
    }

    /// Returns safe discovery diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Safe discovery diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    issue: DiscoveryIssue,
    name: super::Text,
    source_scope: super::SourceScope,
}

impl Diagnostic {
    /// Returns the diagnostic issue.
    #[must_use]
    pub const fn issue(&self) -> DiscoveryIssue {
        self.issue
    }

    /// Returns the affected agent name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the duplicate source scope.
    #[must_use]
    pub const fn source_scope(&self) -> super::SourceScope {
        self.source_scope
    }
}

/// Discovery issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryIssue {
    /// More than one manifest used the same name at the selected precedence.
    DuplicateDefinitionName,
}

/// Discovers agent definitions deterministically from already-loaded manifests.
#[must_use]
pub fn discover(manifests: &[super::Manifest]) -> DiscoveryReport {
    assert!(
        manifests.len() <= super::MAX_ITEMS,
        "manifest inputs stay within the descriptor item bound"
    );
    let mut ordered: super::Collection<&super::Manifest> = manifests.iter().collect();
    assert_eq!(
        ordered.len(),
        manifests.len(),
        "all manifests are staged for deterministic discovery"
    );
    ordered.sort_by(|left, right| {
        left.name()
            .cmp(right.name())
            .then_with(|| left.precedence_rank().cmp(&right.precedence_rank()))
            .then_with(|| left.manifest_identity().cmp(right.manifest_identity()))
    });
    build_report(&ordered)
}

fn build_report(ordered: &[&super::Manifest]) -> DiscoveryReport {
    assert!(
        ordered.len() <= super::MAX_ITEMS,
        "ordered manifests are bounded"
    );
    assert!(
        ordered.iter().all(|manifest| !manifest.name().is_empty()),
        "manifest names must be validated before discovery"
    );
    let mut selected = super::Collection::with_capacity(ordered.len());
    let mut diagnostics = super::Collection::with_capacity(ordered.len());
    let mut group_start = 0;
    while group_start < ordered.len() {
        let group_end = next_name_group_end(ordered, group_start);
        choose_name_group(
            &ordered[group_start..group_end],
            &mut selected,
            &mut diagnostics,
        );
        group_start = group_end;
    }
    selected.sort_by(|left, right| left.name().cmp(right.name()));
    diagnostics.sort_by(|left, right| left.name.cmp(&right.name));
    DiscoveryReport {
        selected,
        diagnostics,
    }
}

fn next_name_group_end(ordered: &[&super::Manifest], group_start: usize) -> usize {
    assert!(
        group_start < ordered.len(),
        "agent-definition group must start inside ordered manifests"
    );
    let name = ordered[group_start].name();
    let mut group_end = group_start.saturating_add(1);
    while group_end < ordered.len() && ordered[group_end].name() == name {
        group_end = group_end.saturating_add(1);
    }
    group_end
}

fn choose_name_group(
    group: &[&super::Manifest],
    selected: &mut super::Collection<super::Manifest>,
    diagnostics: &mut super::Collection<Diagnostic>,
) {
    assert!(
        !group.is_empty(),
        "agent-definition group must not be empty"
    );
    let top_rank = group[0].precedence_rank();
    assert!(
        group
            .iter()
            .all(|candidate| candidate.name() == group[0].name()),
        "one discovery group contains one name"
    );
    let top_count = group
        .iter()
        .filter(|candidate| candidate.precedence_rank() == top_rank)
        .count();
    if top_count > 1 {
        diagnostics.push(duplicate_diagnostic(group[0]));
        return;
    }
    selected.push(group[0].clone());
}

fn duplicate_diagnostic(manifest: &super::Manifest) -> Diagnostic {
    Diagnostic {
        issue: DiscoveryIssue::DuplicateDefinitionName,
        name: super::Text::from(manifest.name()),
        source_scope: manifest.source_scope(),
    }
}
