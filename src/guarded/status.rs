//! Persisted guarded summary lookup at the orchestration shell boundary.

pub fn optional(
    store: &crate::store::Store,
    run_id: crate::engine::RunId,
    definition_id: &crate::orchestration::DefinitionId,
) -> Result<Option<String>, crate::store::StoreError> {
    let summary = store
        .load_run_details(run_id)?
        .and_then(|details| super::evidence::summarize(&definition_id.to_string(), &details))
        .map(|summary| summary.render());
    Ok(summary)
}

pub fn recovered_fallback(record: &crate::store::RunRecord) -> String {
    format!(
        "recovered workflow {} run {} with {} failures",
        record.workflow_name, record.run_id, record.failures
    )
}
