/// Renderer target with distinct accessibility capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticRendererTarget {
    /// Meteorite DOM and browser accessibility mapping.
    MeteoriteDom,
    /// Subwayrat structured terminal summary.
    SubwayratTerminal,
    /// Future renderer-owned native accessibility adapter.
    NativeAccessibility,
}

/// One bounded vocabulary feature requiring renderer support policy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticFeature {
    /// Semantic role.
    Role(super::SemanticRole),
    /// Semantic state.
    State(super::SemanticStateKind),
    /// Semantic property.
    Property(super::SemanticProperty),
    /// Semantic relationship.
    Relation(super::SemanticRelationKind),
    /// Semantic action.
    Action(super::SemanticAction),
}

/// Truthful renderer mapping classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticSupportLevel {
    /// Renderer maps the feature to its accessibility mechanism.
    Direct,
    /// Renderer exposes a structured summary or fallback only.
    Summary,
    /// Renderer cannot expose the feature.
    Unsupported,
    /// Planned native mapping has no runtime evidence.
    Unverified,
}

/// Checked renderer support decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSupport {
    /// Renderer target.
    pub target: SemanticRendererTarget,
    /// Bounded feature.
    pub feature: SemanticFeature,
    /// Mapping level.
    pub level: SemanticSupportLevel,
    /// Required reason for non-direct support.
    pub reason: Option<String>,
}

/// Stable key a native adapter may join with renderer-measured bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSemanticJoinKey {
    /// Accepted snapshot generation.
    pub generation: super::SemanticGeneration,
    /// Stable node identity.
    pub node: super::SemanticNodeId,
}

/// Renderer-neutral semantic summary projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProjection {
    /// Target whose support policy was applied.
    pub target: SemanticRendererTarget,
    /// Accepted generation.
    pub generation: super::SemanticGeneration,
    /// Ordered semantic node summaries.
    pub nodes: Vec<SemanticProjectionNode>,
    /// Validation diagnostics; invalid snapshots emit no nodes.
    pub issues: Vec<super::SemanticIssue>,
}

/// One node summary shared by renderer-owned adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProjectionNode {
    /// Stable node identity.
    pub id: super::SemanticNodeId,
    /// Stable role name.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Stable state summaries.
    pub states: Vec<String>,
    /// Directed relationships.
    pub relations: Vec<super::SemanticRelation>,
    /// Available actions.
    pub actions: Vec<super::SemanticAction>,
    /// Value summary when present.
    pub value: Option<String>,
    /// Checked support rows for used features.
    pub support: Vec<SemanticSupport>,
}

/// Returns the explicit support policy for one target and feature.
#[must_use]
pub fn semantic_support(target: SemanticRendererTarget, feature: SemanticFeature) -> SemanticSupport {
    let (level, reason) = support_decision(target, &feature);
    SemanticSupport {
        target,
        feature,
        level,
        reason,
    }
}

fn support_decision(
    target: SemanticRendererTarget,
    feature: &SemanticFeature,
) -> (SemanticSupportLevel, Option<String>) {
    match target {
        SemanticRendererTarget::MeteoriteDom => meteorite_support(feature),
        SemanticRendererTarget::SubwayratTerminal => terminal_support(feature),
        SemanticRendererTarget::NativeAccessibility => (
            SemanticSupportLevel::Unverified,
            Some("native adapter, measured bounds, and runtime evidence are not implemented".into()),
        ),
    }
}

fn meteorite_support(feature: &SemanticFeature) -> (SemanticSupportLevel, Option<String>) {
    match feature {
        SemanticFeature::Action(_) => (
            SemanticSupportLevel::Unverified,
            Some("DOM event normalization and browser assistive-action runtime are untested".into()),
        ),
        _ => (SemanticSupportLevel::Direct, None),
    }
}

fn terminal_support(feature: &SemanticFeature) -> (SemanticSupportLevel, Option<String>) {
    match feature {
        SemanticFeature::Action(_) => (
            SemanticSupportLevel::Unsupported,
            Some("terminal output has no native accessibility action channel".into()),
        ),
        _ => (
            SemanticSupportLevel::Summary,
            Some("terminal exposes structured role, state, focus, and status text only".into()),
        ),
    }
}

/// Builds a checked summary without renderer geometry or platform objects.
#[must_use]
pub fn project_semantic_summary(
    snapshot: &super::SemanticSnapshot,
    target: SemanticRendererTarget,
) -> SemanticProjection {
    let validation = super::validate_semantic_snapshot(snapshot, None);
    if !validation.is_valid() {
        return invalid_projection(snapshot, target, validation.issues);
    }
    let nodes = snapshot.nodes.iter().map(|node| projection_node(node, target)).collect();
    SemanticProjection {
        target,
        generation: snapshot.generation,
        nodes,
        issues: Vec::new(),
    }
}

fn invalid_projection(
    snapshot: &super::SemanticSnapshot,
    target: SemanticRendererTarget,
    issues: Vec<super::SemanticIssue>,
) -> SemanticProjection {
    SemanticProjection {
        target,
        generation: snapshot.generation,
        nodes: Vec::new(),
        issues,
    }
}

fn projection_node(node: &super::SemanticNode, target: SemanticRendererTarget) -> SemanticProjectionNode {
    SemanticProjectionNode {
        id: node.id.clone(),
        role: node.role.name().to_string(),
        name: node.name.clone(),
        description: node.description.clone(),
        states: state_summaries(node),
        relations: node.relations.clone(),
        actions: node.actions.keys().copied().collect(),
        value: node.value.as_ref().map(value_summary),
        support: node_support(node, target),
    }
}

fn node_support(node: &super::SemanticNode, target: SemanticRendererTarget) -> Vec<SemanticSupport> {
    let mut features = property_features(node);
    features.extend(node.relations.iter().map(|relation| SemanticFeature::Relation(relation.kind)));
    features.extend(node.actions.keys().map(|action| SemanticFeature::Action(*action)));
    features.into_iter().map(|feature| semantic_support(target, feature)).collect()
}

fn property_features(node: &super::SemanticNode) -> Vec<SemanticFeature> {
    let mut features = vec![
        SemanticFeature::Role(node.role),
        SemanticFeature::Property(super::SemanticProperty::Name),
    ];
    features.extend(optional_property_features(node));
    features
}

fn optional_property_features(node: &super::SemanticNode) -> Vec<SemanticFeature> {
    [
        (node.description.is_some(), super::SemanticProperty::Description),
        (node.value.is_some(), super::SemanticProperty::Value),
        (node.collection.is_some(), super::SemanticProperty::Collection),
        (node.announcement.is_some(), super::SemanticProperty::Announcement),
    ]
    .into_iter()
    .filter(|(is_used, _)| *is_used)
    .map(|(_, property)| SemanticFeature::Property(property))
    .collect()
}

fn state_summaries(node: &super::SemanticNode) -> Vec<String> {
    let mut states = presence_state_summaries(node);
    states.extend(value_state_summaries(node));
    states
}

fn presence_state_summaries(node: &super::SemanticNode) -> Vec<String> {
    [
        node.state.is_disabled().then(|| "disabled".to_string()),
        node.state.is_hidden().then(|| "hidden".to_string()),
        node.state.is_required().then(|| "required".to_string()),
        node.state.is_invalid().then(|| "invalid".to_string()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn value_state_summaries(node: &super::SemanticNode) -> Vec<String> {
    [
        node.state.selected.map(|value| format!("selected={value}")),
        node.state.checked.map(|value| format!("checked={value:?}")),
        node.state.expanded.map(|value| format!("expanded={value}")),
        node.state.current.map(|value| format!("current={value:?}")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn value_summary(value: &super::SemanticValue) -> String {
    match value {
        super::SemanticValue::Text(text) => text.clone(),
        super::SemanticValue::Range(range) => {
            format!("{} in {}..={} step {}", range.current, range.minimum, range.maximum, range.step)
        }
    }
}
