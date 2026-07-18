//! Pure Meteorite desktop chrome models and projections.
//!
//! Dioxus components can consume these validated projections to render title
//! bars, menu bars, toolbars, status bars, breadcrumbs, overlays, and dialogs.
//! The core keeps labels, actions, truncation, diagnostics, and content slots in
//! plain data so caller-owned body content remains outside chrome ownership.

use std::collections::BTreeSet;

const METEORITE_CHROME_CLASS: &str = "met-chrome";
const UNTITLED_SURFACE: &str = "Untitled";
const MIN_CONTAINER_WIDTH: u16 = 1;
const SEPARATOR_WIDTH: u16 = 1;

/// Stable chrome surface identifier.
pub type ChromeSurfaceId = String;

/// Stable chrome item identifier.
pub type ChromeItemId = String;

/// Stable chrome action identifier.
pub type ChromeActionId = String;

/// Meteorite chrome surface kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeSurfaceKind {
    /// Title bar.
    TitleBar,
    /// Menu bar or dropdown menu.
    MenuBar,
    /// Toolbar.
    Toolbar,
    /// Status bar.
    StatusBar,
    /// Breadcrumb trail.
    Breadcrumbs,
    /// Overlay frame.
    OverlayFrame,
    /// Dialog frame.
    DialogFrame,
}

/// Chrome item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeItemKind {
    /// Actionable control.
    Control,
    /// Non-action label.
    Label,
    /// Separator.
    Separator,
}

/// Pure chrome item model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeItem {
    /// Stable item identifier.
    pub id: ChromeItemId,
    /// Display label.
    pub label: String,
    /// Item kind.
    pub kind: ChromeItemKind,
    /// Action emitted by actionable controls.
    pub action_id: Option<ChromeActionId>,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item is active or selected.
    pub active: bool,
    /// Optional diagnostic metadata.
    pub diagnostic: Option<String>,
}

impl ChromeItem {
    /// Creates an actionable chrome control.
    #[must_use]
    pub fn control(
        id: impl Into<ChromeItemId>,
        label: impl Into<String>,
        action_id: impl Into<ChromeActionId>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: ChromeItemKind::Control,
            action_id: Some(action_id.into()),
            enabled: true,
            active: false,
            diagnostic: None,
        }
    }

    /// Creates a label item.
    #[must_use]
    pub fn label(id: impl Into<ChromeItemId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: ChromeItemKind::Label,
            action_id: None,
            enabled: true,
            active: false,
            diagnostic: None,
        }
    }

    /// Creates a separator.
    #[must_use]
    pub fn separator(id: impl Into<ChromeItemId>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            kind: ChromeItemKind::Separator,
            action_id: None,
            enabled: false,
            active: false,
            diagnostic: None,
        }
    }

    /// Sets enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets active state.
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

/// Pure Meteorite chrome model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteChromeModel {
    /// Stable surface identifier.
    pub surface_id: ChromeSurfaceId,
    /// Surface kind.
    pub kind: ChromeSurfaceKind,
    /// Surface title.
    pub title: String,
    /// Available width supplied by caller measurement.
    pub container_width: u16,
    /// Caller-owned content slot identifier.
    pub content_slot_id: String,
    /// Chrome items.
    pub items: Vec<ChromeItem>,
}

impl MeteoriteChromeModel {
    /// Creates a chrome model.
    #[must_use]
    pub fn new(
        surface_id: impl Into<ChromeSurfaceId>,
        kind: ChromeSurfaceKind,
        title: impl Into<String>,
        container_width: u16,
    ) -> Self {
        let surface_id = surface_id.into();
        Self {
            content_slot_id: surface_id.clone(),
            surface_id,
            kind,
            title: title.into(),
            container_width,
            items: Vec::new(),
        }
    }

    /// Sets content slot identifier.
    #[must_use]
    pub fn with_content_slot(mut self, content_slot_id: impl Into<String>) -> Self {
        self.content_slot_id = content_slot_id.into();
        self
    }

    /// Sets items.
    #[must_use]
    pub fn with_items(mut self, items: Vec<ChromeItem>) -> Self {
        self.items = items;
        self
    }
}

/// Chrome diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeDiagnosticCode {
    /// Duplicate item identifier.
    DuplicateItemId,
    /// Required label is empty.
    EmptyLabel,
    /// Disabled item was marked active.
    DisabledActiveItem,
    /// Container is too small to show preferred chrome.
    TinyContainer,
    /// Content overflows available width.
    Overflow,
    /// Control has no action identifier.
    MissingAction,
}

/// Chrome diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: ChromeDiagnosticCode,
    /// Surface associated with the diagnostic.
    pub surface_id: ChromeSurfaceId,
    /// Item associated with the diagnostic.
    pub item_id: Option<ChromeItemId>,
    /// Human-readable message.
    pub message: String,
}

impl ChromeDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: ChromeDiagnosticCode,
        surface_id: impl Into<ChromeSurfaceId>,
        item_id: Option<ChromeItemId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            surface_id: surface_id.into(),
            item_id,
            message: message.into(),
        }
    }
}

/// Safe renderable chrome item projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeItemProjection {
    /// Stable item identifier.
    pub id: ChromeItemId,
    /// Safe display label.
    pub label: String,
    /// Item kind.
    pub kind: ChromeItemKind,
    /// Action emitted by this item.
    pub action_id: Option<ChromeActionId>,
    /// Whether the item can be activated.
    pub enabled: bool,
    /// Whether the item is active.
    pub active: bool,
    /// Whether the label was truncated for available width.
    pub truncated: bool,
    /// Optional diagnostic metadata.
    pub diagnostic: Option<String>,
}

/// Validated chrome projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteChromeProjection {
    /// Stable surface identifier.
    pub surface_id: ChromeSurfaceId,
    /// Surface kind.
    pub kind: ChromeSurfaceKind,
    /// Safe title.
    pub title: String,
    /// Caller-owned content slot identifier.
    pub content_slot_id: String,
    /// Safe renderable items.
    pub items: Vec<ChromeItemProjection>,
    /// Diagnostics.
    pub diagnostics: Vec<ChromeDiagnostic>,
}

/// Dioxus component shell projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteChromeComponentProjection {
    /// CSS class root.
    pub root_class: String,
    /// Shared chrome projection.
    pub chrome: MeteoriteChromeProjection,
    /// Typed intents emitted by enabled controls.
    pub emitted_intents: Vec<ChromeActionId>,
}

/// Showcase evidence for chrome states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteChromeShowcaseEvidence {
    /// Renderable state identifiers.
    pub state_ids: Vec<ChromeSurfaceId>,
    /// Semantic parity keys available for comparison with Subwayrat chrome.
    pub parity_keys: Vec<String>,
    /// Diagnostics observed across states.
    pub diagnostics: Vec<ChromeDiagnostic>,
}

/// Validates a Meteorite chrome model and returns a safe projection.
#[must_use]
pub fn validate_meteorite_chrome_model(model: &MeteoriteChromeModel) -> MeteoriteChromeProjection {
    let mut diagnostics = Vec::new();
    let duplicate_ids = duplicate_item_ids(&model.items);
    for item_id in &duplicate_ids {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::DuplicateItemId,
            model.surface_id.clone(),
            Some(item_id.clone()),
            "duplicate chrome item id",
        ));
    }
    if model.container_width < MIN_CONTAINER_WIDTH {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::TinyContainer,
            model.surface_id.clone(),
            None,
            "chrome container is too small",
        ));
    }
    let preferred_width = preferred_model_width(model);
    if preferred_width > model.container_width {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::Overflow,
            model.surface_id.clone(),
            None,
            "chrome content overflows available width",
        ));
    }
    let items = model
        .items
        .iter()
        .map(|item| project_item(model, item, &mut diagnostics))
        .collect();
    MeteoriteChromeProjection {
        surface_id: model.surface_id.clone(),
        kind: model.kind,
        title: safe_title(&model.title),
        content_slot_id: model.content_slot_id.clone(),
        items,
        diagnostics,
    }
}

/// Projects a validated chrome model into a component shell.
#[must_use]
pub fn project_meteorite_chrome_component(
    model: &MeteoriteChromeModel,
) -> MeteoriteChromeComponentProjection {
    let chrome = validate_meteorite_chrome_model(model);
    let emitted_intents = chrome
        .items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| item.action_id.clone())
        .collect();
    MeteoriteChromeComponentProjection {
        root_class: format!("{METEORITE_CHROME_CLASS} {:?}", chrome.kind).to_ascii_lowercase(),
        chrome,
        emitted_intents,
    }
}

/// Builds showcase evidence for normal and edge chrome states.
#[must_use]
pub fn meteorite_chrome_showcase_evidence(
    models: &[MeteoriteChromeModel],
) -> MeteoriteChromeShowcaseEvidence {
    let mut state_ids = Vec::with_capacity(models.len());
    let mut parity_keys = Vec::new();
    let mut diagnostics = Vec::new();
    for model in models {
        let projection = validate_meteorite_chrome_model(model);
        state_ids.push(projection.surface_id.clone());
        parity_keys.extend(projection.items.iter().map(|item| {
            format!(
                "{}:{}:{}:{}",
                projection.surface_id,
                item.id,
                item.action_id.clone().unwrap_or_default(),
                item.enabled
            )
        }));
        diagnostics.extend(projection.diagnostics);
    }
    parity_keys.sort();
    MeteoriteChromeShowcaseEvidence {
        state_ids,
        parity_keys,
        diagnostics,
    }
}

fn duplicate_item_ids(items: &[ChromeItem]) -> BTreeSet<ChromeItemId> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for item in items {
        if !seen.insert(item.id.clone()) {
            duplicates.insert(item.id.clone());
        }
    }
    duplicates
}

fn preferred_model_width(model: &MeteoriteChromeModel) -> u16 {
    let title_width = string_width(&model.title);
    let item_width = model
        .items
        .iter()
        .map(preferred_item_width)
        .fold(0_u16, u16::saturating_add);
    title_width.saturating_add(item_width)
}

fn preferred_item_width(item: &ChromeItem) -> u16 {
    match item.kind {
        ChromeItemKind::Separator => SEPARATOR_WIDTH,
        ChromeItemKind::Control | ChromeItemKind::Label => string_width(&item.label),
    }
}

fn project_item(
    model: &MeteoriteChromeModel,
    item: &ChromeItem,
    diagnostics: &mut Vec<ChromeDiagnostic>,
) -> ChromeItemProjection {
    if item.kind != ChromeItemKind::Separator && item.label.trim().is_empty() {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::EmptyLabel,
            model.surface_id.clone(),
            Some(item.id.clone()),
            "chrome item label is empty",
        ));
    }
    if item.kind == ChromeItemKind::Control && item.action_id.is_none() {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::MissingAction,
            model.surface_id.clone(),
            Some(item.id.clone()),
            "chrome control has no action",
        ));
    }
    let active = if item.active && !item.enabled {
        diagnostics.push(ChromeDiagnostic::new(
            ChromeDiagnosticCode::DisabledActiveItem,
            model.surface_id.clone(),
            Some(item.id.clone()),
            "disabled chrome item cannot be active",
        ));
        false
    } else {
        item.active
    };
    let item_width = preferred_item_width(item);
    ChromeItemProjection {
        id: item.id.clone(),
        label: item.label.clone(),
        kind: item.kind,
        action_id: item.action_id.clone(),
        enabled: item.enabled
            && item
                .action_id
                .as_ref()
                .map_or(true, |action_id| !action_id.is_empty()),
        active,
        truncated: item_width > model.container_width,
        diagnostic: item.diagnostic.clone(),
    }
}

fn safe_title(title: &str) -> String {
    if title.trim().is_empty() {
        return UNTITLED_SURFACE.to_string();
    }
    title.to_string()
}

fn string_width(value: &str) -> u16 {
    let width = value.chars().count();
    u16::try_from(width).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE_CONTAINER: u16 = 120;
    const TINY_CONTAINER: u16 = 0;
    const NARROW_CONTAINER: u16 = 3;

    fn title_bar() -> MeteoriteChromeModel {
        MeteoriteChromeModel::new(
            "title",
            ChromeSurfaceKind::TitleBar,
            "Workspace",
            WIDE_CONTAINER,
        )
        .with_content_slot("workspace-body")
        .with_items(vec![
            ChromeItem::control("close", "Close", "window.close"),
            ChromeItem::separator("sep"),
            ChromeItem::label("status", "Ready"),
        ])
    }

    #[test]
    fn valid_model_projects_component_without_owning_body_content() {
        let component = project_meteorite_chrome_component(&title_bar());

        assert_eq!(component.chrome.content_slot_id, "workspace-body");
        assert!(component.chrome.diagnostics.is_empty());
        assert_eq!(component.emitted_intents, vec!["window.close".to_string()]);
        assert!(component.root_class.contains(METEORITE_CHROME_CLASS));
    }

    #[test]
    fn malformed_model_reports_empty_duplicate_disabled_tiny_and_overflow_cases() {
        let model =
            MeteoriteChromeModel::new("bad", ChromeSurfaceKind::Toolbar, "", TINY_CONTAINER)
                .with_items(vec![
                    ChromeItem::control("dup", "", "missing.label"),
                    ChromeItem::control("dup", "Disabled", "disabled")
                        .with_enabled(false)
                        .with_active(true),
                    ChromeItem {
                        id: "missing-action".to_string(),
                        label: "Broken".to_string(),
                        kind: ChromeItemKind::Control,
                        action_id: None,
                        enabled: true,
                        active: false,
                        diagnostic: None,
                    },
                ]);

        let projection = validate_meteorite_chrome_model(&model);

        assert_eq!(projection.title, UNTITLED_SURFACE);
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::DuplicateItemId })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::EmptyLabel })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::DisabledActiveItem })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::TinyContainer })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::Overflow })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ChromeDiagnosticCode::MissingAction })
        );
        assert!(!projection.items[1].active);
    }

    #[test]
    fn truncation_and_showcase_evidence_are_deterministic() {
        let narrow = MeteoriteChromeModel::new(
            "breadcrumbs",
            ChromeSurfaceKind::Breadcrumbs,
            "Workspace",
            NARROW_CONTAINER,
        )
        .with_items(vec![ChromeItem::label("long", "Very long crumb")]);

        let projection = validate_meteorite_chrome_model(&narrow);
        let evidence = meteorite_chrome_showcase_evidence(&[title_bar(), narrow]);

        assert!(projection.items[0].truncated);
        assert_eq!(
            evidence.state_ids,
            vec!["title".to_string(), "breadcrumbs".to_string()]
        );
        assert!(
            evidence
                .parity_keys
                .iter()
                .any(|key| key.contains("window.close"))
        );
    }
}
