const MAX_CATALOG_ISSUES: usize = 196;

/// Validates an atomic catalog and selected/focused state candidate.
#[must_use]
pub fn validate_tabs_candidate(
    catalog: &super::TabsCatalog,
    selected: &super::TabId,
    focused: &super::TabId,
) -> Vec<super::TabsDiagnostic> {
    let mut issues = validate_catalog(catalog);
    issues.reserve(2);
    if catalog.tab(selected).is_none() {
        issues.push(tabs_issue(super::TabsDiagnosticCode::UnknownSelection, selected.as_str()));
    } else if !catalog.is_eligible(selected) {
        issues.push(tabs_issue(super::TabsDiagnosticCode::IneligibleSelection, selected.as_str()));
    }
    if catalog.tab(focused).is_none() {
        issues.push(tabs_issue(super::TabsDiagnosticCode::UnknownFocus, focused.as_str()));
    } else if !catalog.is_eligible(focused) {
        issues.push(tabs_issue(super::TabsDiagnosticCode::IneligibleFocus, focused.as_str()));
    }
    issues.sort();
    issues.dedup();
    debug_assert!(issues.len() <= MAX_CATALOG_ISSUES);
    debug_assert!(issues.iter().all(|issue| issue.subject.chars().count() <= super::MAX_TAB_TEXT_BYTES));
    issues
}

fn validate_catalog(catalog: &super::TabsCatalog) -> Vec<super::TabsDiagnostic> {
    if catalog.tabs.len() > super::MAX_TABS || catalog.panels.len() > super::MAX_TABS {
        return vec![tabs_issue(super::TabsDiagnosticCode::CatalogTooLarge, "catalog")];
    }
    let mut issues = Vec::with_capacity(MAX_CATALOG_ISSUES);
    if catalog.tabs.is_empty() || catalog.panels.is_empty() {
        issues.push(tabs_issue(super::TabsDiagnosticCode::EmptyCatalog, "catalog"));
    }
    issues.extend(validate_tabs(catalog));
    issues.extend(validate_panels(catalog));
    if catalog.eligible_tabs().next().is_none() {
        issues.push(tabs_issue(super::TabsDiagnosticCode::NoEligibleTab, "catalog"));
    }
    debug_assert!(issues.len() <= MAX_CATALOG_ISSUES);
    debug_assert!(catalog.tabs.len() <= super::MAX_TABS);
    issues
}

fn validate_tabs(catalog: &super::TabsCatalog) -> Vec<super::TabsDiagnostic> {
    let mut issues = Vec::with_capacity(catalog.tabs.len().saturating_mul(3));
    let mut tab_ids = std::collections::BTreeSet::new();
    for tab in &catalog.tabs {
        if !tab_ids.insert(tab.id.as_str()) {
            issues.push(tabs_issue(super::TabsDiagnosticCode::DuplicateTab, tab.id.as_str()));
        }
        if !valid_tab_identity(tab.id.as_str())
            || !valid_tab_identity(tab.panel_id.as_str())
            || !valid_label(&tab.label)
        {
            issues.push(tabs_issue(super::TabsDiagnosticCode::InvalidField, tab.id.as_str()));
        }
        if catalog.panel(&tab.panel_id).is_none() {
            issues.push(tabs_issue(super::TabsDiagnosticCode::MissingPanel, tab.id.as_str()));
        }
    }
    debug_assert!(issues.len() <= catalog.tabs.len().saturating_mul(3));
    debug_assert!(tab_ids.len() <= catalog.tabs.len());
    issues
}

fn validate_panels(catalog: &super::TabsCatalog) -> Vec<super::TabsDiagnostic> {
    let mut issues = Vec::with_capacity(catalog.panels.len().saturating_mul(3));
    let mut panel_ids = std::collections::BTreeSet::new();
    for panel in &catalog.panels {
        if !panel_ids.insert(panel.id.as_str()) {
            issues.push(tabs_issue(super::TabsDiagnosticCode::DuplicatePanel, panel.id.as_str()));
        }
        if !valid_tab_identity(panel.id.as_str()) || !valid_tab_identity(panel.slot_id.as_str()) {
            issues.push(tabs_issue(super::TabsDiagnosticCode::InvalidField, panel.id.as_str()));
        }
        if !catalog.tabs.iter().any(|tab| tab.panel_id == panel.id) {
            issues.push(tabs_issue(super::TabsDiagnosticCode::OrphanPanel, panel.id.as_str()));
        }
    }
    debug_assert!(issues.len() <= catalog.panels.len().saturating_mul(3));
    debug_assert!(panel_ids.len() <= catalog.panels.len());
    issues
}

fn valid_tab_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::MAX_TAB_TEXT_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= super::MAX_TAB_TEXT_BYTES && !value.contains(['\n', '\r'])
}

pub(crate) fn tabs_issue(code: super::TabsDiagnosticCode, subject: &str) -> super::TabsDiagnostic {
    super::TabsDiagnostic {
        code,
        subject: subject.chars().take(super::MAX_TAB_TEXT_BYTES).collect(),
    }
}
