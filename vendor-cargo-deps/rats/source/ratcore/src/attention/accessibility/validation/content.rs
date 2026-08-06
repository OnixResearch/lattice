pub(super) fn node_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    let mut issues = name_issues(node);
    issues.extend(state_issues(node));
    issues.extend(value_issues(node));
    issues.extend(collection_issues(node));
    issues.extend(action_issues(node));
    issues.extend(announcement_issues(node));
    issues
}

fn name_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    [missing_name_issue(node), blank_description_issue(node)].into_iter().flatten().collect()
}

fn missing_name_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let is_labelled = node
        .relations
        .iter()
        .any(|relation| relation.kind == super::super::SemanticRelationKind::LabelledBy);
    (node.name.trim().is_empty() && !is_labelled).then(|| {
        super::issue(
            super::super::SemanticIssueCode::MissingName,
            node.id.as_str(),
            "semantic node needs a name or labelled-by relation",
        )
    })
}

fn blank_description_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    node.description.as_ref().is_some_and(|value| value.trim().is_empty()).then(|| {
        super::issue(
            super::super::SemanticIssueCode::MissingName,
            node.id.as_str(),
            "semantic description must not be blank",
        )
    })
}

fn state_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(4);
    issues.extend(selected_issue(node));
    issues.extend(checked_issue(node));
    issues.extend(expanded_issue(node));
    issues.extend(presence_flag_issues(node));
    issues
}

fn selected_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let can_select = matches!(
        node.role,
        super::super::SemanticRole::MenuItem | super::super::SemanticRole::TreeItem | super::super::SemanticRole::Tab
    );
    (node.state.selected.is_some() && !can_select).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleState,
            node.id.as_str(),
            "selected state requires an item role",
        )
    })
}

fn checked_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let can_check = matches!(node.role, super::super::SemanticRole::Button | super::super::SemanticRole::MenuItem);
    (node.state.checked.is_some() && !can_check).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleState,
            node.id.as_str(),
            "checked state requires a button or menu item",
        )
    })
}

fn expanded_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    (node.state.expanded.is_some() && node.role != super::super::SemanticRole::TreeItem).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleState,
            node.id.as_str(),
            "expanded state requires a tree item",
        )
    })
}

fn presence_flag_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    [bounded_flag_issue(node), required_flag_issue(node)].into_iter().flatten().collect()
}

fn bounded_flag_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let has_only_bounded_flags = node.state.flags.iter().all(|flag| {
        matches!(
            flag,
            super::super::SemanticStateKind::Disabled
                | super::super::SemanticStateKind::Hidden
                | super::super::SemanticStateKind::Required
                | super::super::SemanticStateKind::Invalid
        )
    });
    (!has_only_bounded_flags).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleState,
            node.id.as_str(),
            "value-bearing state kinds cannot be used as presence-only flags",
        )
    })
}

fn required_flag_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let can_require = !node.state.is_required() || node.role == super::super::SemanticRole::Progress;
    (!can_require).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleState,
            node.id.as_str(),
            "required state is unsupported for this bounded role vocabulary",
        )
    })
}

fn value_issues(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let value = node.value.as_ref()?;
    let is_invalid = node.role != super::super::SemanticRole::Progress || malformed_value(value);
    is_invalid.then(|| {
        super::issue(
            super::super::SemanticIssueCode::InvalidValue,
            node.id.as_str(),
            "semantic value is malformed or incompatible with its role",
        )
    })
}

fn malformed_value(value: &super::super::SemanticValue) -> bool {
    match value {
        super::super::SemanticValue::Text(text) => text.trim().is_empty(),
        super::super::SemanticValue::Range(range) => {
            range.minimum > range.current
                || range.current > range.maximum
                || range.minimum > range.maximum
                || range.step == 0
        }
    }
}

fn collection_issues(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let collection = node.collection?;
    let is_role_valid = matches!(
        node.role,
        super::super::SemanticRole::MenuItem | super::super::SemanticRole::TreeItem | super::super::SemanticRole::Tab
    );
    let is_valid = is_role_valid && valid_collection(collection);
    (!is_valid).then(|| {
        super::issue(
            super::super::SemanticIssueCode::InvalidCollection,
            node.id.as_str(),
            "collection metadata has an incompatible role or invalid position",
        )
    })
}

fn valid_collection(collection: super::super::SemanticCollection) -> bool {
    collection.level > 0
        && collection.position > 0
        && collection.set_size > 0
        && collection.position <= collection.set_size
}

fn action_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    let mut issues = node
        .actions
        .iter()
        .flat_map(|(action, target)| {
            [
                incompatible_action_issue(node, *action),
                missing_target_issue(node, target),
            ]
            .into_iter()
            .flatten()
        })
        .collect::<Vec<_>>();
    issues.extend(expansion_action_issue(node));
    issues
}

fn incompatible_action_issue(
    node: &super::super::SemanticNode,
    action: super::super::SemanticAction,
) -> Option<super::super::SemanticIssue> {
    (!action_supported(node.role, action)).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleAction,
            node.id.as_str(),
            "semantic action is incompatible with the node role",
        )
    })
}

fn missing_target_issue(
    node: &super::super::SemanticNode,
    target: &crate::nexus_core::ActionId,
) -> Option<super::super::SemanticIssue> {
    target.as_str().trim().is_empty().then(|| {
        super::issue(
            super::super::SemanticIssueCode::MissingActionTarget,
            node.id.as_str(),
            "semantic action target must not be empty",
        )
    })
}

fn expansion_action_issue(node: &super::super::SemanticNode) -> Option<super::super::SemanticIssue> {
    let is_expanded = node.state.expanded?;
    let incompatible = if is_expanded {
        super::super::SemanticAction::Expand
    } else {
        super::super::SemanticAction::Collapse
    };
    node.actions.contains_key(&incompatible).then(|| {
        super::issue(
            super::super::SemanticIssueCode::IncompatibleAction,
            node.id.as_str(),
            "semantic expansion action contradicts accepted state",
        )
    })
}

fn announcement_issues(node: &super::super::SemanticNode) -> Vec<super::super::SemanticIssue> {
    let Some(announcement) = &node.announcement else {
        return Vec::new();
    };
    let is_valid = node.role == super::super::SemanticRole::Status && !announcement.message.trim().is_empty();
    if is_valid {
        Vec::new()
    } else {
        vec![super::issue(
            super::super::SemanticIssueCode::InvalidAnnouncement,
            node.id.as_str(),
            "announcement requires a status role and non-empty message",
        )]
    }
}

pub(super) const fn action_supported(role: super::super::SemanticRole, action: super::super::SemanticAction) -> bool {
    match role {
        super::super::SemanticRole::Button => {
            matches!(action, super::super::SemanticAction::Activate | super::super::SemanticAction::Focus)
        }
        super::super::SemanticRole::MenuItem => matches!(
            action,
            super::super::SemanticAction::Activate
                | super::super::SemanticAction::Select
                | super::super::SemanticAction::Focus
        ),
        super::super::SemanticRole::TreeItem => matches!(
            action,
            super::super::SemanticAction::Activate
                | super::super::SemanticAction::Select
                | super::super::SemanticAction::Expand
                | super::super::SemanticAction::Collapse
                | super::super::SemanticAction::Focus
        ),
        super::super::SemanticRole::Tab => matches!(
            action,
            super::super::SemanticAction::Activate
                | super::super::SemanticAction::Select
                | super::super::SemanticAction::Focus
        ),
        super::super::SemanticRole::Dialog => {
            matches!(action, super::super::SemanticAction::Dismiss | super::super::SemanticAction::Focus)
        }
        super::super::SemanticRole::Progress => matches!(
            action,
            super::super::SemanticAction::Increment
                | super::super::SemanticAction::Decrement
                | super::super::SemanticAction::SetValue
        ),
        super::super::SemanticRole::Application
        | super::super::SemanticRole::Menu
        | super::super::SemanticRole::Tree
        | super::super::SemanticRole::Heading
        | super::super::SemanticRole::Description
        | super::super::SemanticRole::Status
        | super::super::SemanticRole::Group
        | super::super::SemanticRole::TabList
        | super::super::SemanticRole::TabPanel => false,
    }
}
