use super::*;

/// Builds a deterministic action registry snapshot.
#[must_use]
pub fn build_registry(actions: Vec<ActionDescriptor>) -> ActionRegistryBuild {
    let action_count = actions.len();
    let duplicate_ids = duplicate_ids(&actions);
    let mut issues = Vec::with_capacity(action_count);
    let mut valid_actions = Vec::with_capacity(action_count);
    debug_assert!(issues.capacity() >= action_count);
    debug_assert!(valid_actions.capacity() >= action_count);

    for action in actions {
        let mut action_issues = validate_descriptor(&action);
        if duplicate_ids.contains(&action.id) {
            action_issues.push(ActionRegistryIssue::field_issue(
                ActionRegistryIssueCode::DuplicateId,
                action.id.clone(),
                FIELD_ID,
                "duplicate action id",
            ));
        }
        if action_issues.is_empty() {
            valid_actions.push(action);
        } else {
            issues.extend(action_issues);
        }
    }

    valid_actions.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.id.cmp(&right.id))
    });

    debug_assert!(valid_actions.len() <= action_count);
    debug_assert!(issues.len() <= issues.capacity());

    let (shortcut_issues, blocked_reasons) = shortcut_conflicts(&valid_actions);
    issues.extend(shortcut_issues);
    issues.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.action_ids.cmp(&right.action_ids))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.message.cmp(&right.message))
    });

    ActionRegistryBuild {
        snapshot: ActionRegistrySnapshot {
            actions: valid_actions,
            blocked_reasons,
        },
        issues,
    }
}

impl ActionShortcut {
    fn canonical_key(&self) -> ShortcutConflictKey {
        let mut modifiers = self
            .modifiers
            .iter()
            .map(|modifier| modifier.trim().to_lowercase())
            .collect::<Vec<_>>();
        modifiers.sort();
        ShortcutConflictKey {
            scope: self.scope.trim().to_lowercase(),
            key: self.key.trim().to_lowercase(),
            modifiers,
        }
    }
}

fn duplicate_ids(actions: &[ActionDescriptor]) -> std::collections::BTreeSet<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = std::collections::BTreeSet::new();
    for action in actions {
        let id = action.id.trim().to_string();
        if id.is_empty() {
            continue;
        }
        if !seen.insert(id.clone()) {
            duplicates.insert(id);
        }
    }
    duplicates
}

fn validate_descriptor(action: &ActionDescriptor) -> Vec<ActionRegistryIssue> {
    debug_assert!(action.id.trim().len() <= action.id.len());
    debug_assert!(action.label.trim().len() <= action.label.len());
    let mut issues = Vec::new();
    push_if_empty(
        &mut issues,
        action.id.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptyId,
            action.id.clone(),
            FIELD_ID,
            "action id must not be empty",
        ),
    );
    push_if_empty(
        &mut issues,
        action.label.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptyLabel,
            action.id.clone(),
            FIELD_LABEL,
            "action label must not be empty",
        ),
    );
    push_if_empty(
        &mut issues,
        action.category.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptyCategory,
            action.id.clone(),
            FIELD_CATEGORY,
            "action category must not be empty",
        ),
    );
    push_if_empty(
        &mut issues,
        action.source.name.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptySource,
            action.id.clone(),
            FIELD_SOURCE,
            "action source must not be empty",
        ),
    );
    if let Some(shortcut) = &action.shortcut {
        issues.extend(validate_shortcut(action, shortcut));
    }
    issues
}

fn validate_shortcut(
    action: &ActionDescriptor,
    shortcut: &ActionShortcut,
) -> Vec<ActionRegistryIssue> {
    let issue_bound = shortcut.modifiers.len().saturating_add(2);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(shortcut.modifiers.len() <= issue_bound);
    push_if_empty(
        &mut issues,
        shortcut.scope.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptyShortcutScope,
            action.id.clone(),
            FIELD_SHORTCUT_SCOPE,
            "shortcut scope must not be empty",
        ),
    );
    push_if_empty(
        &mut issues,
        shortcut.key.trim(),
        ActionRegistryIssue::field_issue(
            ActionRegistryIssueCode::EmptyShortcutKey,
            action.id.clone(),
            FIELD_SHORTCUT_KEY,
            "shortcut key must not be empty",
        ),
    );
    for modifier in &shortcut.modifiers {
        push_if_empty(
            &mut issues,
            modifier.trim(),
            ActionRegistryIssue::field_issue(
                ActionRegistryIssueCode::EmptyShortcutModifier,
                action.id.clone(),
                FIELD_SHORTCUT_MODIFIER,
                "shortcut modifier must not be empty",
            ),
        );
    }
    issues
}

fn push_if_empty(
    issues: &mut impl Extend<ActionRegistryIssue>,
    value: &str,
    issue: ActionRegistryIssue,
) {
    if value.is_empty() {
        issues.extend(std::iter::once(issue));
    }
}

fn shortcut_conflicts(
    actions: &[ActionDescriptor],
) -> (
    Vec<ActionRegistryIssue>,
    std::collections::BTreeMap<String, String>,
) {
    let mut by_shortcut: std::collections::BTreeMap<ShortcutConflictKey, Vec<String>> =
        std::collections::BTreeMap::new();
    for action in actions {
        if !action.enablement.is_enabled() {
            continue;
        }
        let Some(shortcut) = &action.shortcut else {
            continue;
        };
        by_shortcut
            .entry(shortcut.canonical_key())
            .or_default()
            .push(action.id.clone());
    }

    debug_assert!(by_shortcut.len() <= actions.len());
    debug_assert!(
        by_shortcut
            .values()
            .all(|action_ids| !action_ids.is_empty())
    );

    let conflicts: Vec<(ShortcutConflictKey, Vec<String>)> = by_shortcut
        .into_iter()
        .filter(|(_shortcut, action_ids)| action_ids.len() > 1)
        .collect();
    let blocked = conflicts
        .iter()
        .flat_map(|(_shortcut, action_ids)| action_ids.iter())
        .map(|action_id| (action_id.clone(), BLOCKED_CONFLICT_REASON.to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut issues = Vec::with_capacity(conflicts.len());
    debug_assert!(issues.capacity() >= conflicts.len());
    debug_assert!(blocked.len() >= conflicts.len());
    for (shortcut, action_ids) in conflicts {
        issues.push(ActionRegistryIssue::multi_action_issue(
            ActionRegistryIssueCode::ShortcutConflict,
            action_ids,
            format!(
                "shortcut conflict in scope `{}` for `{}`",
                shortcut.scope, shortcut.key
            ),
        ));
    }
    (issues, blocked)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ShortcutConflictKey {
    scope: String,
    key: String,
    modifiers: Vec<String>,
}
