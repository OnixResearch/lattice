//! The artifact's wire document and its contract checks, keys in sorted order at every level.

/// Provenance kinds an artifact may declare.
const PROVENANCE_KINDS: [&str; 3] = ["fixture", "trained", "distilled"];

/// The artifact document.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PolicyDocument {
    pub(super) action_schema: ActionSchemaDocument,
    pub(super) bounds: BoundsDocument,
    pub(super) feature_schema: FeatureSchemaDocument,
    pub(super) model: crate::inference::ModelDocument,
    pub(super) provenance: ProvenanceDocument,
    pub(super) runtime: RuntimeDocument,
    pub(super) schema: alloc::string::String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ActionSchemaDocument {
    pub(super) actions: alloc::vec::Vec<alloc::string::String>,
    pub(super) id: alloc::string::String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BoundsDocument {
    pub(super) abstain_below: Option<i64>,
    pub(super) operation_budget: u64,
    pub(super) score_max: i64,
    pub(super) score_min: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FeatureSchemaDocument {
    pub(super) features: alloc::vec::Vec<FeatureDocument>,
    pub(super) id: alloc::string::String,
    pub(super) normalization: alloc::string::String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FeatureDocument {
    pub(super) max: i64,
    pub(super) min: i64,
    pub(super) name: alloc::string::String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProvenanceDocument {
    pub(super) evaluation: alloc::string::String,
    pub(super) kind: alloc::string::String,
    pub(super) non_claims: alloc::vec::Vec<alloc::string::String>,
    pub(super) producer: alloc::string::String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeDocument {
    pub(super) artifact_api: u32,
    #[serde(rename = "crate")]
    pub(super) crate_name: alloc::string::String,
}

/// What the contract checks admitted, ready for the admitted policy.
pub(super) struct Checked {
    pub(super) features: alloc::vec::Vec<super::FeatureBound>,
    pub(super) actions: alloc::vec::Vec<alloc::string::String>,
}

/// One bounded artifact document.
pub(super) fn parse(bytes: &[u8]) -> Result<PolicyDocument, super::AdmissionError> {
    if bytes.is_empty() {
        return Err(super::AdmissionError::Unavailable);
    }
    if bytes.len() > super::MAX_ARTIFACT_BYTES {
        return Err(super::AdmissionError::TooLarge);
    }
    serde_json::from_slice(bytes).map_err(|_| super::AdmissionError::Malformed)
}

/// The canonical identity of `document` under the artifact schema.
pub(super) fn canonical_identity(
    document: &PolicyDocument,
) -> Result<alloc::string::String, super::AdmissionError> {
    let canonical =
        serde_json::to_string(document).map_err(|_| super::AdmissionError::Malformed)?;
    Ok(crate::digest::domain_identity(
        super::ARTIFACT_SCHEMA,
        &[&canonical],
    ))
}

/// Every contract except the model's own shape, in a fixed order.
pub(super) fn check(document: &PolicyDocument) -> Result<Checked, super::AdmissionError> {
    if document.schema != super::ARTIFACT_SCHEMA {
        return Err(super::AdmissionError::Schema);
    }
    let runtime = &document.runtime;
    if runtime.crate_name != super::RUNTIME_CRATE || runtime.artifact_api != super::ARTIFACT_API {
        return Err(super::AdmissionError::Runtime);
    }
    let features = check_features(&document.feature_schema)?;
    let actions = check_actions(&document.action_schema)?;
    check_bounds(&document.bounds)?;
    check_provenance(&document.provenance)?;
    debug_assert!(!features.is_empty() && features.len() <= super::MAX_FEATURES);
    debug_assert!(!actions.is_empty() && actions.len() <= super::MAX_ACTIONS);
    Ok(Checked { features, actions })
}

fn check_features(
    schema: &FeatureSchemaDocument,
) -> Result<alloc::vec::Vec<super::FeatureBound>, super::AdmissionError> {
    let is_declared = crate::digest::is_safe_identity(&schema.id)
        && schema.normalization == super::NORMALIZATION
        && (1..=super::MAX_FEATURES).contains(&schema.features.len())
        && crate::digest::is_unique_label_set(
            schema.features.iter().map(|feature| feature.name.as_str()),
        );
    let is_bounded = schema.features.iter().all(|feature| {
        feature.min < feature.max
            && feature.min >= -super::MAX_FEATURE_MAGNITUDE
            && feature.max <= super::MAX_FEATURE_MAGNITUDE
    });
    if !is_declared || !is_bounded {
        return Err(super::AdmissionError::FeatureSchema);
    }
    debug_assert!(schema.features.len() <= super::MAX_FEATURES);
    Ok(schema
        .features
        .iter()
        .map(|feature| super::FeatureBound {
            name: feature.name.clone(),
            min: feature.min,
            max: feature.max,
        })
        .collect())
}

fn check_actions(
    schema: &ActionSchemaDocument,
) -> Result<alloc::vec::Vec<alloc::string::String>, super::AdmissionError> {
    let is_declared = crate::digest::is_safe_identity(&schema.id)
        && (1..=super::MAX_ACTIONS).contains(&schema.actions.len())
        && crate::digest::is_unique_label_set(
            schema.actions.iter().map(alloc::string::String::as_str),
        );
    if !is_declared {
        return Err(super::AdmissionError::ActionSchema);
    }
    debug_assert!(!schema.actions.is_empty());
    Ok(schema.actions.clone())
}

/// The score bounds, the abstention threshold within them, and the operation budget.
fn check_bounds(bounds: &BoundsDocument) -> Result<(), super::AdmissionError> {
    let is_score_range_bounded = bounds.score_min < bounds.score_max
        && bounds.score_min >= -super::MAX_PARAMETER_MAGNITUDE
        && bounds.score_max <= super::MAX_PARAMETER_MAGNITUDE;
    let is_threshold_bounded = bounds
        .abstain_below
        .is_none_or(|threshold| (bounds.score_min..=bounds.score_max).contains(&threshold));
    let is_budget_bounded = (1..=super::MAX_OPERATION_BUDGET).contains(&bounds.operation_budget);
    if !is_score_range_bounded || !is_threshold_bounded || !is_budget_bounded {
        return Err(super::AdmissionError::Bounds);
    }
    Ok(())
}

fn check_provenance(provenance: &ProvenanceDocument) -> Result<(), super::AdmissionError> {
    let is_declared = PROVENANCE_KINDS.contains(&provenance.kind.as_str())
        && !provenance.producer.trim().is_empty()
        && !provenance.evaluation.trim().is_empty()
        && !provenance.non_claims.is_empty()
        && provenance
            .non_claims
            .iter()
            .all(|claim| !claim.trim().is_empty());
    if !is_declared {
        return Err(super::AdmissionError::Provenance);
    }
    Ok(())
}
