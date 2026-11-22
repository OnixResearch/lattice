//! Whether a materialized workflow still equals the expansion its source association recorded.
//! Pure.

/// Whether the workflow no longer equals the expansion its source association recorded, so
/// reconciliation would mark the association a materialized fork. `None` without an association.
pub fn diverges_from_expansion(workflow: &crate::engine::Workflow) -> Result<Option<bool>, &'static str> {
    let Some(association) = workflow.authoring.as_ref() else {
        return Ok(None);
    };
    association.validate()?;
    Ok(Some(super::expanded_identity(workflow)? != association.expanded_artifact))
}
