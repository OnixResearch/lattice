/// Stable tabs fixture classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsFixtureKind {
    /// Ordinary enabled horizontal tabs.
    Normal,
    /// Catalog contains a disabled trigger.
    Disabled,
    /// Catalog contains a hidden trigger.
    Hidden,
    /// Invalid empty catalog candidate.
    Empty,
    /// Catalog replacement repairs selected and focused identities.
    Replacement,
    /// Renderer width is narrow but shared behavior is unchanged.
    Narrow,
    /// Manual activation requires explicit activation.
    Manual,
    /// Automatic activation proposes selection after focus.
    Automatic,
    /// Invalid duplicate identity candidate.
    Invalid,
}

impl TabsFixtureKind {
    /// Returns stable fixture identity.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Normal => "headless-tabs.normal-v1",
            Self::Disabled => "headless-tabs.disabled-v1",
            Self::Hidden => "headless-tabs.hidden-v1",
            Self::Empty => "headless-tabs.empty-v1",
            Self::Replacement => "headless-tabs.replacement-v1",
            Self::Narrow => "headless-tabs.narrow-v1",
            Self::Manual => "headless-tabs.manual-v1",
            Self::Automatic => "headless-tabs.automatic-v1",
            Self::Invalid => "headless-tabs.invalid-v1",
        }
    }
}

/// Generates all checked provider-neutral tabs fixture rows.
#[must_use]
pub fn headless_tabs_fixture_evidence() -> String {
    let kinds = [
        TabsFixtureKind::Normal,
        TabsFixtureKind::Disabled,
        TabsFixtureKind::Hidden,
        TabsFixtureKind::Empty,
        TabsFixtureKind::Replacement,
        TabsFixtureKind::Narrow,
        TabsFixtureKind::Manual,
        TabsFixtureKind::Automatic,
        TabsFixtureKind::Invalid,
    ];
    let mut output = String::from("schema=headless-tabs-fixtures.v1\nevidence_class=semantic-behavior-not-visual\n");
    for kind in kinds {
        output.push_str(&fixture_row(kind));
    }
    output.push_str("not_claimed=markup-parity,pixel-parity,physical-focus-runtime\n");
    debug_assert!(output.starts_with("schema=headless-tabs-fixtures.v1"));
    debug_assert!(kinds.iter().all(|kind| output.contains(kind.id())));
    output
}

fn fixture_row(kind: TabsFixtureKind) -> String {
    match kind {
        TabsFixtureKind::Empty => invalid_row(kind, super::super::TabsCatalog {
            tabs: Vec::new(),
            panels: Vec::new(),
        }),
        TabsFixtureKind::Invalid => invalid_catalog_row(kind),
        TabsFixtureKind::Disabled => valid_row(
            kind,
            super::fixture_catalog(super::FixtureAvailability {
                is_details_disabled: true,
                ..super::FixtureAvailability::default()
            }),
            super::super::TabsActivationPolicy::Automatic,
            vec![super::super::TabsMessage::MoveNext],
        ),
        TabsFixtureKind::Hidden => valid_row(
            kind,
            super::fixture_catalog(super::FixtureAvailability {
                is_history_hidden: true,
                ..super::FixtureAvailability::default()
            }),
            super::super::TabsActivationPolicy::Automatic,
            vec![super::super::TabsMessage::MoveLast],
        ),
        TabsFixtureKind::Manual => valid_row(
            kind,
            super::fixture_catalog(super::FixtureAvailability::default()),
            super::super::TabsActivationPolicy::Manual,
            vec![
                super::super::TabsMessage::MoveNext,
                super::super::TabsMessage::ActivateFocused,
            ],
        ),
        TabsFixtureKind::Replacement => replacement_row(kind),
        TabsFixtureKind::Normal | TabsFixtureKind::Narrow | TabsFixtureKind::Automatic => valid_row(
            kind,
            super::fixture_catalog(super::FixtureAvailability::default()),
            super::super::TabsActivationPolicy::Automatic,
            vec![super::super::TabsMessage::MoveNext],
        ),
    }
}

fn invalid_catalog_row(kind: TabsFixtureKind) -> String {
    let mut catalog = super::fixture_catalog(super::FixtureAvailability::default());
    if let Some(duplicate) = catalog.tabs.first().cloned() {
        catalog.tabs.push(duplicate);
    }
    invalid_row(kind, catalog)
}

fn valid_row(
    kind: TabsFixtureKind,
    catalog: super::super::TabsCatalog,
    activation: super::super::TabsActivationPolicy,
    messages: Vec<super::super::TabsMessage>,
) -> String {
    debug_assert!(messages.len() <= super::MAX_ADAPTER_MESSAGES);
    debug_assert!(catalog.tabs.len() <= super::super::model::MAX_TABS);
    match super::super::TabsState::new(
        catalog,
        super::super::TabId::new("tab.overview"),
        super::super::TabId::new("tab.overview"),
        super::super::TabsOrientation::Horizontal,
        activation,
    ) {
        Ok(mut accepted) => accepted_row(kind, &mut accepted, messages),
        Err(issues) => format!("fixture={} status=unexpected-rejection diagnostics={issues:?}\n", kind.id()),
    }
}

fn accepted_row(
    kind: TabsFixtureKind,
    accepted: &mut super::super::TabsState,
    messages: Vec<super::super::TabsMessage>,
) -> String {
    let receipt = super::run_tabs_adapter_script(accepted, messages);
    format!(
        "fixture={} status=accepted selected={} focused={} semantics={}\n",
        kind.id(),
        receipt.selected.as_str(),
        receipt.focused.as_str(),
        receipt.semantic_identity,
    )
}

fn replacement_row(kind: TabsFixtureKind) -> String {
    let Ok(mut state) = super::canonical_tabs_state(super::super::TabsActivationPolicy::Automatic) else {
        return format!("fixture={} status=fixture-construction-failed\n", kind.id());
    };
    let mut replacement = super::fixture_catalog(super::FixtureAvailability::default());
    debug_assert_eq!(replacement.tabs.len(), 3);
    debug_assert_eq!(replacement.panels.len(), 3);
    replacement.tabs.retain(|tab| tab.id.as_str() != "tab.overview");
    replacement.panels.retain(|panel| panel.id.as_str() != "panel.overview");
    let receipt = super::run_tabs_adapter_script(&mut state, [super::super::TabsMessage::ReplaceCatalog(replacement)]);
    format!(
        "fixture={} status=repaired selected={} focused={} semantics={}\n",
        kind.id(),
        receipt.selected.as_str(),
        receipt.focused.as_str(),
        receipt.semantic_identity,
    )
}

fn invalid_row(kind: TabsFixtureKind, catalog: super::super::TabsCatalog) -> String {
    let diagnostics = super::super::TabsState::new(
        catalog,
        super::super::TabId::new("tab.overview"),
        super::super::TabId::new("tab.overview"),
        super::super::TabsOrientation::Horizontal,
        super::super::TabsActivationPolicy::Automatic,
    )
    .err()
    .unwrap_or_default();
    format!("fixture={} status=rejected diagnostics={diagnostics:?}\n", kind.id())
}
