//! Identity-based active, selection, anchor, and semantic projection.

use super::model::*;

/// Stable identity state independent of presentation indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionIdentityState<Id> {
    /// Active item, if any.
    pub active: Option<Id>,
    /// Selected identities.
    pub selected: std::collections::BTreeSet<Id>,
    /// Scroll anchor, if any.
    pub anchor: Option<Id>,
}

/// Repair policy when an active item or anchor disappears.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityRepairPolicy {
    /// Choose the nearest surviving eligible identity by prior order.
    NearestSurvivor,
    /// Clear missing identity references.
    Clear,
}

/// Accepted identity reconciliation plus visible repair diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityReconciliation<Id> {
    /// Reconciled state.
    pub state: CollectionIdentityState<Id>,
    /// Stable diagnostics for repaired references.
    pub diagnostics: Vec<CollectionDiagnostic>,
}

/// Preserves surviving identities and deterministically repairs removed ones.
pub fn reconcile_collection_state<Id: Clone + Ord>(
    previous: &CollectionSnapshot<Id>,
    candidate: &CollectionSnapshot<Id>,
    state: &CollectionIdentityState<Id>,
    disabled: &std::collections::BTreeSet<Id>,
    policy: IdentityRepairPolicy,
) -> Result<IdentityReconciliation<Id>, CollectionDiagnostic> {
    if previous.generation.next() != Some(candidate.generation) {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidGeneration, "identity-state"));
    }
    debug_assert_eq!(previous.generation.next(), Some(candidate.generation));
    debug_assert!(candidate.items.len() <= MAX_COLLECTION_ITEMS);
    let candidate_ids: std::collections::BTreeSet<&Id> = candidate.items.iter().collect();
    let mut diagnostics = Vec::with_capacity(2);
    let (active, active_repaired) = repair_reference(RepairInput {
        current: state.active.as_ref(),
        previous,
        candidate,
        ineligible: disabled,
        policy,
    });
    if active_repaired {
        diagnostics.push(CollectionDiagnostic::new(CollectionDiagnosticCode::IdentityRepaired, "active"));
    }
    let empty = std::collections::BTreeSet::new();
    let (anchor, anchor_repaired) = repair_reference(RepairInput {
        current: state.anchor.as_ref(),
        previous,
        candidate,
        ineligible: &empty,
        policy,
    });
    if anchor_repaired {
        diagnostics.push(CollectionDiagnostic::new(CollectionDiagnosticCode::IdentityRepaired, "anchor"));
    }
    let selected = state.selected.iter().filter(|id| candidate_ids.contains(id)).cloned().collect();
    Ok(IdentityReconciliation {
        state: CollectionIdentityState {
            active,
            selected,
            anchor,
        },
        diagnostics,
    })
}

struct RepairInput<'a, Id> {
    current: Option<&'a Id>,
    previous: &'a CollectionSnapshot<Id>,
    candidate: &'a CollectionSnapshot<Id>,
    ineligible: &'a std::collections::BTreeSet<Id>,
    policy: IdentityRepairPolicy,
}

fn repair_reference<Id: Clone + Ord>(input: RepairInput<'_, Id>) -> (Option<Id>, bool) {
    debug_assert!(input.previous.items.len() <= MAX_COLLECTION_ITEMS);
    debug_assert!(input.candidate.items.len() <= MAX_COLLECTION_ITEMS);
    let Some(current) = input.current else {
        return (None, false);
    };
    if input.candidate.items.contains(current) && !input.ineligible.contains(current) {
        return (Some(current.clone()), false);
    }
    if input.policy == IdentityRepairPolicy::Clear {
        return (None, true);
    }
    let prior_index = input.previous.items.iter().position(|id| id == current).unwrap_or(0);
    let replacement = input
        .candidate
        .items
        .iter()
        .enumerate()
        .filter(|(_, id)| !input.ineligible.contains(*id))
        .min_by_key(|(index, _)| index.abs_diff(prior_index))
        .map(|(_, id)| id.clone());
    (replacement, true)
}

/// Renderer support for offscreen semantic collection items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OffscreenSemanticPolicy {
    /// Project every identity independent of visual materialization.
    FullCollection,
    /// Project mounted identities only, with an explicit limitation reason.
    MaterializedOnly {
        /// Renderer capability limitation.
        reason: String,
    },
}

/// Domain-owned semantic facts for one ordered identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSemanticInput<Id> {
    /// Stable identity.
    pub id: Id,
    /// One-based hierarchy level.
    pub level: u64,
    /// One-based position among siblings or rows.
    pub position: u64,
    /// Sibling/row set size.
    pub set_size: u64,
    /// Accepted selection state.
    pub selected: bool,
    /// Accepted expansion state where meaningful.
    pub expanded: Option<bool>,
    /// Accepted disabled state.
    pub disabled: bool,
}

/// Semantic facts kept separate from visual children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSemanticProjection<Id> {
    /// Stable identity.
    pub id: Id,
    /// One-based hierarchy level.
    pub level: u64,
    /// One-based collection position.
    pub position: u64,
    /// Collection set size.
    pub set_size: u64,
    /// Accepted selection state.
    pub selected: bool,
    /// Accepted expansion state.
    pub expanded: Option<bool>,
    /// Accepted disabled state.
    pub disabled: bool,
    /// Whether a visual child is requested.
    pub materialized: bool,
}

/// Projects bounded semantic collection metadata without constructing nodes.
pub fn project_collection_semantics<Id: Clone + Ord>(
    inputs: &[CollectionSemanticInput<Id>],
    materialized: &[MaterializedItem<Id>],
    policy: &OffscreenSemanticPolicy,
) -> Result<Vec<CollectionSemanticProjection<Id>>, CollectionDiagnostic> {
    debug_assert!(inputs.len() <= MAX_COLLECTION_ITEMS);
    debug_assert!(materialized.len() <= MAX_COLLECTION_ITEMS);
    if matches!(policy, OffscreenSemanticPolicy::MaterializedOnly { reason } if reason.trim().is_empty()) {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidSemantics, "offscreen-semantic-reason"));
    }
    let mounted: std::collections::BTreeSet<&Id> = materialized.iter().map(|item| &item.id).collect();
    let mut identities = std::collections::BTreeSet::new();
    let mut output = Vec::with_capacity(inputs.len());
    for input in inputs {
        if !identities.insert(&input.id) {
            return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::DuplicateIdentity, "collection-semantics"));
        }
        if !valid_semantic_position(input) {
            return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidSemantics, "collection-semantics"));
        }
        let is_materialized = mounted.contains(&input.id);
        if is_materialized || policy == &OffscreenSemanticPolicy::FullCollection {
            output.push(CollectionSemanticProjection {
                id: input.id.clone(),
                level: input.level,
                position: input.position,
                set_size: input.set_size,
                selected: input.selected,
                expanded: input.expanded,
                disabled: input.disabled,
                materialized: is_materialized,
            });
        }
    }
    Ok(output)
}

fn valid_semantic_position<Id>(input: &CollectionSemanticInput<Id>) -> bool {
    let has_positive_values = input.level > 0 && input.position > 0 && input.set_size > 0;
    has_positive_values && input.position <= input.set_size
}
