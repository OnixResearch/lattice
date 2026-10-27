const EMPTY_ENTRY_COUNT: usize = 0;

/// Replays the selected active branch of a validated session tree snapshot.
///
/// # Errors
///
/// Returns [`super::super::flow::Error`] when tree shape validation fails, the active branch
/// contains unsupported entries, a tool result is orphaned, or a summary has no
/// prior model-visible span to replace.
pub fn replay_snapshot(
    snapshot: &super::Snapshot,
) -> Result<super::super::flow::Report, super::super::flow::Error> {
    validate_snapshot(snapshot)?;
    let selected = active_path(snapshot)?;
    replay_selected(&selected, snapshot.labels())
}

fn replay_selected(
    selected: &[&super::Node],
    labels: &[super::Label],
) -> Result<super::super::flow::Report, super::super::flow::Error> {
    debug_assert!(!selected.is_empty());
    debug_assert!(labels.iter().all(|label| !label.text().is_empty()));
    let mut context = alloc::vec::Vec::with_capacity(selected.len());
    let mut counts = super::super::ReplayCounts::default();
    let mut has_model_visible_span = false;

    for node in selected {
        counts.record_entry();
        match node.entry() {
            super::super::Entry::Summary { id, content, .. } => {
                if !has_model_visible_span {
                    return Err(super::super::flow::Error::MalformedSummaryBoundary {
                        entry_id: super::super::flow::id_string(id),
                    });
                }
                context.clear();
                counts.reset_model_visible_items();
                counts.record_summary();
                context.push(super::super::flow::model_item(
                    &mut counts,
                    super::super::ContextRole::System,
                    id,
                    content,
                ));
                has_model_visible_span = true;
            }
            super::super::Entry::UsageObservation { .. } => counts.record_usage(),
            super::super::Entry::SafeReceipt { .. } => counts.record_receipt(),
            entry => {
                if let Some(item) = super::super::flow::replay_entry(entry, &mut counts)? {
                    context.push(item);
                    has_model_visible_span = true;
                }
            }
        }
    }

    Ok(super::super::flow::Report::new(
        context,
        counts,
        labels.to_vec(),
    ))
}

fn validate_snapshot(snapshot: &super::Snapshot) -> Result<(), super::super::flow::Error> {
    if snapshot.entries().len() == EMPTY_ENTRY_COUNT {
        return Err(super::super::flow::Error::EmptyLedger);
    }
    validate_unique_ids(snapshot)?;
    validate_active_leaf(snapshot)?;
    for node in snapshot.entries() {
        validate_supported_entry(node.entry())?;
        validate_parent(snapshot, node)?;
        validate_tool_parent(snapshot, node)?;
        validate_node_cycle(snapshot, node)?;
    }
    validate_labels(snapshot)
}

fn validate_unique_ids(snapshot: &super::Snapshot) -> Result<(), super::super::flow::Error> {
    let entries = snapshot.entries();
    for (index, node) in entries.iter().enumerate() {
        let has_duplicate = entries
            .iter()
            .skip(index.saturating_add(1))
            .any(|candidate| candidate.entry().id() == node.entry().id());
        if has_duplicate {
            return Err(super::super::flow::Error::DuplicateEntryId {
                entry_id: super::super::flow::id_string(node.entry().id()),
            });
        }
    }
    Ok(())
}

fn validate_active_leaf(snapshot: &super::Snapshot) -> Result<(), super::super::flow::Error> {
    if find_node(snapshot, snapshot.active_leaf()).is_some() {
        Ok(())
    } else {
        Err(super::super::flow::Error::InvalidActiveLeaf {
            active_leaf: super::super::flow::id_string(snapshot.active_leaf()),
        })
    }
}

fn validate_supported_entry(entry: &super::super::Entry) -> Result<(), super::super::flow::Error> {
    match entry {
        super::super::Entry::Unsupported { id, kind_label }
        | super::super::Entry::AdapterOwned { id, kind_label } => {
            Err(super::super::flow::unsupported_entry(id, kind_label))
        }
        super::super::Entry::UserMessage { .. }
        | super::super::Entry::AssistantMessage { .. }
        | super::super::Entry::ToolResult { .. }
        | super::super::Entry::Summary { .. }
        | super::super::Entry::UsageObservation { .. }
        | super::super::Entry::SafeReceipt { .. } => Ok(()),
    }
}

fn validate_parent(
    snapshot: &super::Snapshot,
    node: &super::Node,
) -> Result<(), super::super::flow::Error> {
    if let Some(parent_id) = node.parent_id() {
        if find_node(snapshot, parent_id).is_none() {
            return Err(super::super::flow::Error::MissingParent {
                entry_id: super::super::flow::id_string(node.entry().id()),
                parent_id: super::super::flow::id_string(parent_id),
            });
        }
    }
    Ok(())
}

fn validate_tool_parent(
    snapshot: &super::Snapshot,
    node: &super::Node,
) -> Result<(), super::super::flow::Error> {
    if node.entry().kind() != super::super::EntryKind::ToolResult {
        return Ok(());
    }
    if let Some(parent_id) = node.parent_id() {
        if find_node(snapshot, parent_id).map(|parent| parent.entry().kind())
            == Some(super::super::EntryKind::AssistantMessage)
        {
            return Ok(());
        }
    }
    Err(super::super::flow::Error::OrphanToolResult {
        entry_id: super::super::flow::id_string(node.entry().id()),
    })
}

fn validate_node_cycle(
    snapshot: &super::Snapshot,
    node: &super::Node,
) -> Result<(), super::super::flow::Error> {
    debug_assert!(!snapshot.entries().is_empty());
    debug_assert!(find_node(snapshot, node.entry().id()).is_some());
    let mut seen = alloc::vec::Vec::with_capacity(snapshot.entries().len());
    let mut current_id = node.entry().id();
    let mut remaining_hops = snapshot.entries().len();
    while remaining_hops > EMPTY_ENTRY_COUNT {
        remaining_hops = remaining_hops.saturating_sub(1);
        if seen.iter().any(|seen_id| *seen_id == current_id.as_str()) {
            return Err(super::super::flow::Error::ParentCycle {
                entry_id: super::super::flow::id_string(node.entry().id()),
                cycle_entry_id: super::super::flow::id_string(current_id),
            });
        }
        seen.push(current_id.as_str());
        let current = find_node(snapshot, current_id).ok_or_else(|| {
            super::super::flow::Error::MissingParent {
                entry_id: super::super::flow::id_string(node.entry().id()),
                parent_id: super::super::flow::id_string(current_id),
            }
        })?;
        if let Some(parent_id) = current.parent_id() {
            current_id = parent_id;
        } else {
            return Ok(());
        }
    }
    Err(super::super::flow::Error::ParentCycle {
        entry_id: super::super::flow::id_string(node.entry().id()),
        cycle_entry_id: super::super::flow::id_string(current_id),
    })
}

fn validate_labels(snapshot: &super::Snapshot) -> Result<(), super::super::flow::Error> {
    for label in snapshot.labels() {
        if find_node(snapshot, label.target_id()).is_none() {
            return Err(super::super::flow::Error::MissingLabelTarget {
                target_id: super::super::flow::id_string(label.target_id()),
            });
        }
    }
    Ok(())
}

fn active_path(
    snapshot: &super::Snapshot,
) -> Result<alloc::vec::Vec<&super::Node>, super::super::flow::Error> {
    debug_assert!(!snapshot.entries().is_empty());
    debug_assert!(find_node(snapshot, snapshot.active_leaf()).is_some());
    let mut path = alloc::vec::Vec::with_capacity(snapshot.entries().len());
    let mut current_id = snapshot.active_leaf();
    let mut remaining_hops = snapshot.entries().len();
    while remaining_hops > EMPTY_ENTRY_COUNT {
        remaining_hops = remaining_hops.saturating_sub(1);
        let current = find_node(snapshot, current_id).ok_or_else(|| {
            super::super::flow::Error::InvalidActiveLeaf {
                active_leaf: super::super::flow::id_string(snapshot.active_leaf()),
            }
        })?;
        path.push(current);
        if let Some(parent_id) = current.parent_id() {
            current_id = parent_id;
        } else {
            path.reverse();
            return Ok(path);
        }
    }
    Err(super::super::flow::Error::ParentCycle {
        entry_id: super::super::flow::id_string(snapshot.active_leaf()),
        cycle_entry_id: super::super::flow::id_string(current_id),
    })
}

fn find_node<'a>(
    snapshot: &'a super::Snapshot,
    id: &super::super::EntryId,
) -> Option<&'a super::Node> {
    snapshot
        .entries()
        .iter()
        .find(|node| node.entry().id() == id)
}
