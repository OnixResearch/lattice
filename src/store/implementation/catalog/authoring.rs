//! Atomic author revision admission and editable drafts, separate from last admission.
mod editing;
use redb::ReadableTable as _;

const DRAFTS: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("workflow_source_drafts_v1");
const GENERATIONS: redb::TableDefinition<&str, u64> = redb::TableDefinition::new("workflow_catalog_generations_v1");

#[derive(Clone, Copy, PartialEq, Eq)]
enum Publication {
    CompareAndSwap,
    LegacyUpsert,
    SourceReadmission,
}

fn error(message: &'static str) -> crate::store::StoreError {
    crate::store::StoreError::Conflict(message.into())
}

fn revision(workflow: &crate::engine::Workflow) -> Result<crate::authoring::ContentIdentity, crate::store::StoreError> {
    workflow
        .catalog_revision
        .clone()
        .map(Ok)
        .unwrap_or_else(|| crate::authoring::identity("lattice.legacy-catalog-revision.v1", workflow).map_err(error))
}

pub(super) fn with_revision(
    mut workflow: crate::engine::Workflow,
) -> Result<crate::engine::Workflow, crate::store::StoreError> {
    workflow.catalog_revision = Some(revision(&workflow)?);
    Ok(workflow)
}

fn prepare(
    mut candidate: crate::engine::Workflow,
    previous: Option<&crate::authoring::ContentIdentity>,
    generation: u64,
) -> Result<crate::engine::Workflow, crate::store::StoreError> {
    crate::authoring::reconcile_materialized(&mut candidate).map_err(error)?;
    candidate.catalog_revision = None;
    let next = crate::authoring::identity("lattice.catalog-revision.v1", &(generation, previous, &candidate))
        .map_err(error)?;
    candidate.catalog_revision = Some(next);
    Ok(candidate)
}

fn source_transition(
    workflow: &crate::engine::Workflow,
    previous: Option<&crate::engine::Workflow>,
    readmission: bool,
) -> Result<crate::engine::Workflow, crate::store::StoreError> {
    let mut candidate = workflow.clone();
    if !readmission
        && previous.and_then(|workflow| workflow.authoring.as_ref()).is_none()
        && candidate.authoring.as_ref().is_some_and(|new| {
            new.state == crate::authoring::AssociationState::CapturedModules
                || new.observation.is_some()
                || new.construction.is_some()
        })
    {
        return Err(error("captured source requires exact source preview admission"));
    }
    if !readmission && let Some(old) = previous.and_then(|workflow| workflow.authoring.as_ref()) {
        if let Some(new) = &candidate.authoring {
            let has_same_program = new.program == old.program && new.source_revision == old.source_revision;
            let has_same_evaluation = new.compiler == old.compiler
                && new.evaluated_output == old.evaluated_output
                && new.expanded_artifact == old.expanded_artifact;
            let has_same_construction = new.observation == old.observation && new.construction == old.construction;
            if !(has_same_program && has_same_evaluation && has_same_construction) {
                return Err(error("source association changed; use exact source preview admission"));
            }
        } else {
            candidate.authoring = Some(old.clone());
        }
        // Ordinary publication may materialize an edit, but cannot choose the
        // provenance state. Keep the stored state (including a sticky fork);
        // prepare() will mark a real expanded-identity delta as a fork.
        if let Some(new) = &mut candidate.authoring {
            new.state = old.state.clone();
        }
    }
    debug_assert_eq!(crate::authoring::expanded_identity(&candidate), crate::authoring::expanded_identity(workflow));
    debug_assert_eq!(candidate.catalog_revision, workflow.catalog_revision);
    Ok(candidate)
}

fn publish(
    txn: &redb::WriteTransaction,
    workflow: &crate::engine::Workflow,
    expected: Option<&crate::authoring::ContentIdentity>,
    mode: Publication,
) -> Result<crate::engine::Workflow, crate::store::StoreError> {
    let mut table = txn
        .open_table(crate::store::WORKFLOWS)
        .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
    let current: Option<crate::engine::Workflow> = table
        .get(workflow.meta.name.as_str())
        .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
        .map(|guard| crate::store::deserialize(guard.value()))
        .transpose()?;
    let current_revision = current.as_ref().map(revision).transpose()?;
    if current_revision.as_ref() != expected
        && !(mode == Publication::LegacyUpsert && current.as_ref().is_none_or(|old| old.authoring.is_none()))
    {
        return Err(error("workflow revision changed; reopen before admission"));
    }
    let generation = {
        let mut generations =
            txn.open_table(GENERATIONS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        let previous_generation = generations
            .get(workflow.meta.name.as_str())
            .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
            .map_or(0, |guard| guard.value());
        let next = previous_generation.checked_add(1).ok_or_else(|| error("workflow catalog generation exhausted"))?;
        generations
            .insert(workflow.meta.name.as_str(), next)
            .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        next
    };
    let candidate = source_transition(workflow, current.as_ref(), mode == Publication::SourceReadmission)?;
    let candidate = prepare(candidate, current_revision.as_ref(), generation)?;
    debug_assert!(candidate.catalog_revision.is_some());
    debug_assert_eq!(candidate.meta.name, workflow.meta.name);
    let data = crate::store::serialize(&candidate)?;
    table
        .insert(candidate.meta.name.as_str(), data.as_slice())
        .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
    Ok(candidate)
}

impl crate::store::Store {
    /// Admit an explicitly imported file-context candidate without weakening draft CAS.
    /// Retained entry bytes are not a complete captured-module evaluation receipt.
    pub fn admit_imported_workflow(
        &self,
        workflow: &crate::engine::Workflow,
        expected: Option<&crate::authoring::ContentIdentity>,
    ) -> Result<crate::engine::Workflow, crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        if let Some(association) = &workflow.authoring {
            association.validate().map_err(error)?;
            if association.state != crate::authoring::AssociationState::RetainedEntrySnapshot
                || association.compiler != crate::authoring::CompilerIdentity::current().map_err(error)?
                || association.expanded_artifact != crate::authoring::expanded_identity(workflow).map_err(error)?
            {
                return Err(error("file import differs from its observed materialized candidate"));
            }
        }
        if crate::workflow_validation::has_fatal_diagnostics(&crate::workflow_validation::validate_workflow(workflow)) {
            return Err(error("current native workflow validation rejected import"));
        }
        crate::workflow_format::choregraph::lower_portable(&crate::workflow_format::workflow_to_portable(workflow))
            .map_err(|_| error("current graph admission rejected import"))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let mode = if workflow.authoring.is_some() {
            Publication::SourceReadmission
        } else {
            Publication::CompareAndSwap
        };
        let candidate = publish(&txn, workflow, expected, mode)?;
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(candidate)
    }

    /// Required CAS path for TUI and source admission. The workflow and its source
    /// association are one record published in one redb transaction.
    pub fn save_workflow_cas(
        &self,
        workflow: &crate::engine::Workflow,
        expected: Option<&crate::authoring::ContentIdentity>,
    ) -> Result<crate::engine::Workflow, crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let candidate = publish(&txn, workflow, expected, Publication::CompareAndSwap)?;
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(candidate)
    }

    pub(super) fn save_catalog_workflow(
        &self,
        workflow: &crate::engine::Workflow,
    ) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        // Preserve old no-source upsert callers. They may not overwrite an author
        // program; associated workflows always require the exact loaded token.
        let mode = if workflow.authoring.is_none() {
            Publication::LegacyUpsert
        } else {
            Publication::CompareAndSwap
        };
        publish(&txn, workflow, workflow.catalog_revision.as_ref(), mode)?;
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }
}
