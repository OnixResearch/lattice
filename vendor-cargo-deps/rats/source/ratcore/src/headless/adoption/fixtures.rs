/// Returns the reviewed component-family inventory used by documentation and tests.
#[must_use]
pub fn built_in_headless_inventory() -> Vec<super::model::HeadlessComponentInventoryRow> {
    vec![
        conforming_tabs_row("ratcore"),
        adapter_tabs_row("meteorite"),
        adapter_tabs_row("subwayrat"),
        incomplete_row(IncompleteRow {
            component_id: "headless.command-palette.v1",
            owner: "ratcore",
            classification: super::model::HeadlessComponentClassification::Candidate,
            reason: None,
            next_action: "add named parts and semantic-tree association evidence",
        }),
        incomplete_row(IncompleteRow {
            component_id: "headless.overlay-positioning.v1",
            owner: "renderers",
            classification: super::model::HeadlessComponentClassification::RendererOwned,
            reason: Some("placement depends on renderer measurement and viewport geometry"),
            next_action: "retain renderer ownership and share only accepted dismissal intents",
        }),
        incomplete_row(IncompleteRow {
            component_id: "headless.date-picker.v1",
            owner: "renderers",
            classification: super::model::HeadlessComponentClassification::IntentionallyDivergent,
            reason: Some("terminal and browser date entry expose different interaction strengths"),
            next_action: "review calendar selection separately from physical entry widgets",
        }),
        incomplete_row(IncompleteRow {
            component_id: "headless.listbox.v1",
            owner: "workspace",
            classification: super::model::HeadlessComponentClassification::Blocked,
            reason: None,
            next_action: "first reconcile existing select and searchable-select authorities",
        }),
    ]
}

fn conforming_tabs_row(owner: &str) -> super::model::HeadlessComponentInventoryRow {
    super::model::HeadlessComponentInventoryRow {
        component_id: "headless.tabs.v1".to_string(),
        owner: owner.to_string(),
        classification: super::model::HeadlessComponentClassification::Conforming.as_str().to_string(),
        reason: None,
        next_action: None,
        capabilities: direct_capabilities(owner),
    }
}

fn direct_capabilities(owner: &str) -> Vec<super::model::HeadlessCapabilityEntry> {
    super::model::HeadlessComponentCapability::all()
        .into_iter()
        .map(|capability| super::model::HeadlessCapabilityEntry {
            capability,
            support: super::model::HeadlessCapabilitySupport::Direct,
            evidence: Some(format!("headless-tabs.{owner}.v1")),
            reason: None,
            next_action: None,
        })
        .collect()
}

fn adapter_tabs_row(owner: &str) -> super::model::HeadlessComponentInventoryRow {
    super::model::HeadlessComponentInventoryRow {
        component_id: "headless.tabs.v1".to_string(),
        owner: owner.to_string(),
        classification: super::model::HeadlessComponentClassification::Candidate.as_str().to_string(),
        reason: None,
        next_action: Some("promote after maintained physical-focus runtime evidence is checked".to_string()),
        capabilities: adapter_capabilities(owner),
    }
}

fn adapter_capabilities(owner: &str) -> Vec<super::model::HeadlessCapabilityEntry> {
    super::model::HeadlessComponentCapability::all()
        .into_iter()
        .map(|capability| adapter_capability(owner, capability))
        .collect()
}

fn adapter_capability(
    owner: &str,
    capability: super::model::HeadlessComponentCapability,
) -> super::model::HeadlessCapabilityEntry {
    let is_runtime_focus = capability == super::model::HeadlessComponentCapability::Focus;
    super::model::HeadlessCapabilityEntry {
        capability,
        support: if is_runtime_focus {
            super::model::HeadlessCapabilitySupport::Partial
        } else {
            super::model::HeadlessCapabilitySupport::Direct
        },
        evidence: Some(format!("headless-tabs.{owner}.v1")),
        reason: is_runtime_focus
            .then(|| "semantic focus agrees; maintained physical-focus runtime evidence is absent".to_string()),
        next_action: is_runtime_focus
            .then(|| "add maintained browser or terminal physical-focus integration evidence".to_string()),
    }
}

#[derive(Debug, Clone, Copy)]
struct IncompleteRow<'a> {
    component_id: &'a str,
    owner: &'a str,
    classification: super::model::HeadlessComponentClassification,
    reason: Option<&'a str>,
    next_action: &'a str,
}

fn incomplete_row(input: IncompleteRow<'_>) -> super::model::HeadlessComponentInventoryRow {
    super::model::HeadlessComponentInventoryRow {
        component_id: input.component_id.to_string(),
        owner: input.owner.to_string(),
        classification: input.classification.as_str().to_string(),
        reason: input.reason.map(str::to_string),
        next_action: Some(input.next_action.to_string()),
        capabilities: Vec::new(),
    }
}
