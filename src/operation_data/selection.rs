#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "selection formulas use the local operation-data error and model vocabulary throughout"
)]
#![allow(
    assertion_density,
    reason = "untrusted selectors and bound plans return typed diagnostics instead of terminating the process"
)]

use super::core::Error;
use super::model::*;

pub fn reduce_query(
    candidates: &[(RecordMetadata, Option<serde_json::Value>)],
    query: &Query,
    authority: QueryAuthority,
) -> Result<Vec<QueryResult>, Error> {
    validate_query(query)?;
    if candidates.len() > MAX_QUERY_CANDIDATES {
        return Err(Error::Limit("query candidates"));
    }
    for (metadata, value) in candidates {
        super::core::validate_metadata(metadata)?;
        if let Some(value) = value {
            validate_resource_value(metadata, value)?;
        }
    }
    let mut selected = candidates
        .iter()
        .filter(|(metadata, _)| matches_query(metadata, query))
        .map(|(metadata, value)| {
            if metadata.sensitivity == SensitivityClass::Sensitive && !authority.allow_sensitive {
                return Err(Error::Secret);
            }
            let selected_value = match (&query.field_path, metadata.record_class, value) {
                (None, _, _) => None,
                (Some(path), RecordClass::Resource, Some(value)) => Some(select_declared_path(metadata, value, path)?),
                (Some(_), RecordClass::Resource, None) => return Err(Error::Invalid("resource payload unavailable")),
                (Some(_), RecordClass::File, _) => return Err(Error::Invalid("file payload is opaque")),
            };
            Ok(QueryResult {
                metadata: metadata.clone(),
                selected_value,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    selected.sort_by(|left, right| {
        left.metadata
            .sequence
            .cmp(&right.metadata.sequence)
            .then_with(|| left.metadata.version_identity.cmp(&right.metadata.version_identity))
    });
    if query.order == QueryOrder::VersionDescending {
        selected.reverse();
    }
    selected.truncate(query.maximum_result_count);
    Ok(selected)
}

pub fn resolve_exact(results: &[QueryResult]) -> Result<ResolvedInput, Error> {
    let [result] = results else {
        return if results.is_empty() {
            Err(Error::Empty)
        } else {
            Err(Error::Ambiguous)
        };
    };
    super::core::validate_metadata(&result.metadata)?;
    Ok(ResolvedInput {
        version_identity: result.metadata.version_identity.clone(),
        content_identity: result.metadata.payload.content_identity.clone(),
    })
}

pub fn resolve_latest(results: &[QueryResult]) -> Result<ResolvedInput, Error> {
    for result in results {
        super::core::validate_metadata(&result.metadata)?;
    }
    let maximum = results.iter().map(|result| result.metadata.sequence).max().ok_or(Error::Empty)?;
    let latest = results.iter().filter(|result| result.metadata.sequence == maximum).cloned().collect::<Vec<_>>();
    resolve_exact(&latest)
}

pub fn recheck(expected: &ResolvedInput, current: &ResolvedInput) -> Result<(), Error> {
    if expected == current { Ok(()) } else { Err(Error::Stale) }
}

// r[impl lattice.operation_data.query.bound]
pub fn bind_resolved_inputs(
    plan: &crate::operation::InvocationPlan,
    mut inputs: Vec<ResolvedInput>,
) -> Result<BoundInvocationPlan, Error> {
    crate::operation::planning::validate_plan_identity(plan).map_err(|_| Error::Invalid("base invocation plan"))?;
    inputs.sort_by(|left, right| left.version_identity.cmp(&right.version_identity));
    for window in inputs.windows(2) {
        if window[0].version_identity == window[1].version_identity {
            return Err(Error::Conflict);
        }
    }
    for input in &inputs {
        super::core::validate_identity(&input.version_identity)?;
        super::core::validate_identity(&input.content_identity)?;
    }
    let identity = super::core::hash_value("lattice.operation-data-bound-invocation.v1", &(&plan.identity, &inputs))?;
    let bound = BoundInvocationPlan {
        schema: "lattice.operation-data-bound-invocation.v1".to_string(),
        identity,
        base_invocation_identity: plan.identity.clone(),
        invocation: plan.clone(),
        resolved_data_inputs: inputs,
    };
    validate_bound_invocation_plan(&bound)?;
    Ok(bound)
}

pub fn validate_bound_invocation_plan(plan: &BoundInvocationPlan) -> Result<(), Error> {
    if plan.schema != "lattice.operation-data-bound-invocation.v1"
        || plan.base_invocation_identity != plan.invocation.identity
    {
        return Err(Error::Invalid("bound invocation schema"));
    }
    crate::operation::planning::validate_plan_identity(&plan.invocation)
        .map_err(|_| Error::Invalid("base invocation plan"))?;
    if plan
        .resolved_data_inputs
        .windows(2)
        .any(|window| window[0].version_identity >= window[1].version_identity)
    {
        return Err(Error::Invalid("resolved input order"));
    }
    for input in &plan.resolved_data_inputs {
        super::core::validate_identity(&input.version_identity)?;
        super::core::validate_identity(&input.content_identity)?;
    }
    let expected = super::core::hash_value(
        "lattice.operation-data-bound-invocation.v1",
        &(&plan.invocation.identity, &plan.resolved_data_inputs),
    )?;
    if plan.identity != expected {
        return Err(Error::Invalid("bound invocation identity"));
    }
    Ok(())
}

fn validate_query(query: &Query) -> Result<(), Error> {
    if query.schema != QUERY_SCHEMA {
        return Err(Error::Invalid("query schema"));
    }
    if query.maximum_result_count == 0 || query.maximum_result_count > MAX_QUERY_RESULTS {
        return Err(Error::Limit("query result limit"));
    }
    for identity in [
        query.version_identity.as_ref(),
        query.record_identity.as_ref(),
        query.schema_identity.as_ref(),
        query.workflow_revision.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        super::core::validate_identity(identity)?;
    }
    for text in [
        query.operation_type.as_ref(),
        query.method.as_ref(),
        query.instance.as_ref(),
        query.output_name.as_ref(),
        query.outcome.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        super::core::validate_text(text)?;
    }
    if let Some(tag) = &query.tag {
        super::core::validate_text(&tag.key)?;
        super::core::validate_text(&tag.value)?;
    }
    if let Some(path) = &query.field_path {
        if path.is_empty() || path.len() > MAX_FIELD_PATH_PARTS {
            return Err(Error::Limit("field path"));
        }
        for part in path {
            super::core::validate_text(part)?;
        }
    }
    Ok(())
}

fn matches_query(metadata: &RecordMetadata, query: &Query) -> bool {
    query.version_identity.as_ref().is_none_or(|value| value == &metadata.version_identity)
        && query.record_identity.as_ref().is_none_or(|value| value == &metadata.record_identity)
        && query.record_class.is_none_or(|value| value == metadata.record_class)
        && query.operation_type.as_ref().is_none_or(|value| value == &metadata.producer.operation_type)
        && query.method.as_ref().is_none_or(|value| value == &metadata.producer.method)
        && query.instance.as_ref().is_none_or(|value| value == &metadata.producer.instance)
        && query.output_name.as_ref().is_none_or(|value| value == &metadata.output_name)
        && query
            .schema_identity
            .as_ref()
            .is_none_or(|value| metadata.payload.schema_identity.as_ref() == Some(value))
        && query.tag.as_ref().is_none_or(|tag| metadata.tags.get(&tag.key) == Some(&tag.value))
        && query.producer_run_id.is_none_or(|value| value == metadata.producer.run_id)
        && query.workflow_revision.as_ref().is_none_or(|value| value == &metadata.producer.workflow_revision)
        && query.outcome.as_ref().is_none_or(|value| value == &metadata.outcome)
}

pub(super) fn select_declared_path(
    metadata: &RecordMetadata,
    value: &serde_json::Value,
    path: &[String],
) -> Result<serde_json::Value, Error> {
    if !metadata.payload.selectable_paths.iter().any(|allowed| allowed == path) {
        return Err(Error::Invalid("undeclared structured field path"));
    }
    let mut selected = value;
    for part in path {
        selected = selected.get(part).ok_or(Error::Invalid("structured field path"))?;
    }
    Ok(selected.clone())
}

fn validate_resource_value(metadata: &RecordMetadata, value: &serde_json::Value) -> Result<(), Error> {
    if metadata.record_class != RecordClass::Resource {
        return Err(Error::Invalid("structured value for opaque file"));
    }
    let bytes = super::core::canonical_json_bytes(value)?;
    let byte_length_bytes = u64::try_from(bytes.len()).map_err(|_| Error::Limit("resource payload bytes"))?;
    let content_identity = crate::operation::identity::hash_bytes(super::core::CONTENT_DOMAIN, &bytes);
    if byte_length_bytes != metadata.payload.byte_length_bytes || content_identity != metadata.payload.content_identity
    {
        return Err(Error::Invalid("resource payload identity"));
    }
    Ok(())
}
