use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl crate::store::Store {
    pub fn save_source_draft(&self, draft: &crate::authoring::SourceDraft) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        draft.validate().map_err(super::error)?;
        let bytes = crate::store::serialize(draft)?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(super::DRAFTS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(draft.id.as_str(), bytes.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn load_source_draft(
        &self,
        id: &str,
    ) -> Result<Option<crate::authoring::SourceDraft>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = match txn.open_table(super::DRAFTS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(crate::store::StoreError::Table(e.to_string())),
        };
        table
            .get(id)
            .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
            .map(|guard| crate::store::deserialize(guard.value()))
            .transpose()
    }

    /// Persist a failed editable draft without touching the admitted workflow.
    pub fn preview_source_draft(
        &self,
        draft: &mut crate::authoring::SourceDraft,
    ) -> Result<(), crate::store::StoreError> {
        let outcome = crate::authoring::shell::preview(draft);
        self.save_source_draft(draft)?;
        outcome.map_err(super::error)
    }

    // r[impl lattice.source_authoring.admission]
    // r[impl lattice.graph_construction.editing.scope]
    pub fn admit_source_draft(
        &self,
        draft: &crate::authoring::SourceDraft,
    ) -> Result<crate::engine::Workflow, crate::store::StoreError> {
        let outcome = self.admit_source_draft_inner(draft);
        if matches!(outcome, Err(crate::store::StoreError::Conflict(_))) {
            self.retain_admission_failure(draft)?;
        }
        outcome
    }

    fn retain_admission_failure(&self, draft: &crate::authoring::SourceDraft) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(super::DRAFTS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let current: Option<crate::authoring::SourceDraft> = table
                .get(draft.id.as_str())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
                .map(|guard| crate::store::deserialize(guard.value()))
                .transpose()?;
            if let Some(mut current) = current
                && crate::authoring::identity("lattice.source-draft.v1", &current).map_err(super::error)?
                    == crate::authoring::identity("lattice.source-draft.v1", draft).map_err(super::error)?
            {
                current.diagnostic = Some("source admission failed; preview was not admitted".into());
                let bytes = crate::store::serialize(&current)?;
                table
                    .insert(draft.id.as_str(), bytes.as_slice())
                    .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    fn admit_source_draft_inner(
        &self,
        draft: &crate::authoring::SourceDraft,
    ) -> Result<crate::engine::Workflow, crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let preview = checked_preview(draft)?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        require_durable_preview(&txn, draft)?;
        let admitted = super::publish(
            &txn,
            &preview.candidate,
            draft.expected_revision.as_ref(),
            super::Publication::SourceReadmission,
        )?;
        debug_assert!(admitted.catalog_revision.is_some());
        debug_assert_eq!(
            admitted.authoring.as_ref().map(|source| &source.source_revision),
            Some(&preview.input_identity)
        );
        let mut completed = draft.clone();
        completed.expected_revision = admitted.catalog_revision.clone();
        completed.preview = None;
        completed.diagnostic = None;
        {
            let bytes = crate::store::serialize(&completed)?;
            let mut table =
                txn.open_table(super::DRAFTS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(draft.id.as_str(), bytes.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(admitted)
    }
}

fn checked_preview(
    draft: &crate::authoring::SourceDraft,
) -> Result<&crate::authoring::SourcePreview, crate::store::StoreError> {
    draft.validate().map_err(super::error)?;
    let preview = draft.preview.as_ref().ok_or_else(|| super::error("draft has no successful preview"))?;
    if preview.input_identity != draft.program.identity().map_err(super::error)?
        || preview.compiler != crate::authoring::CompilerIdentity::current().map_err(super::error)?
    {
        return Err(super::error("source inputs or compiler changed since preview"));
    }
    let association = preview
        .candidate
        .authoring
        .as_ref()
        .ok_or_else(|| super::error("preview has no source association"))?;
    association.validate().map_err(super::error)?;
    if association.program.as_ref() != &draft.program
        || association.compiler != preview.compiler
        || association.state != crate::authoring::AssociationState::CapturedModules
    {
        return Err(super::error("preview candidate changed since compilation"));
    }
    if association.expanded_artifact != crate::authoring::expanded_identity(&preview.candidate).map_err(super::error)?
        || preview.candidate.meta.name != draft.workflow_name
    {
        return Err(super::error("preview candidate changed since compilation"));
    }
    let rebuilt = crate::authoring::shell::compile(&draft.program).map_err(super::error)?;
    if crate::authoring::expanded_identity(&rebuilt).map_err(super::error)?
        != crate::authoring::expanded_identity(&preview.candidate).map_err(super::error)?
        || rebuilt.authoring.as_ref().and_then(|source| source.observation.as_ref()) != association.observation.as_ref()
        || rebuilt.authoring.as_ref().and_then(|source| source.construction.as_ref())
            != association.construction.as_ref()
    {
        return Err(super::error("source preview differs from current captured compilation"));
    }
    let diagnostics = crate::workflow_validation::validate_workflow(&preview.candidate);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(super::error("current native workflow validation rejected preview"));
    }
    if preview.candidate.construction_profile {
        // The fresh rebuild above re-evaluated the selected worker and compared
        // the entire observation and any native construction binding.
        crate::authoring::construction::admit_materialized(&preview.candidate).map_err(super::error)?;
    } else {
        crate::workflow_format::choregraph::lower_portable(&crate::workflow_format::workflow_to_portable(
            &preview.candidate,
        ))
        .map_err(|_| super::error("current graph admission rejected preview"))?;
    }
    debug_assert_eq!(preview.candidate.meta.name, draft.workflow_name);
    debug_assert_eq!(association.program.as_ref(), &draft.program);
    Ok(preview)
}

fn require_durable_preview(
    txn: &redb::WriteTransaction,
    draft: &crate::authoring::SourceDraft,
) -> Result<(), crate::store::StoreError> {
    let table = txn.open_table(super::DRAFTS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
    let durable: crate::authoring::SourceDraft = table
        .get(draft.id.as_str())
        .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
        .map(|guard| crate::store::deserialize(guard.value()))
        .transpose()?
        .ok_or_else(|| super::error("draft is not durably previewed"))?;
    if crate::authoring::identity("lattice.source-draft.v1", &durable).map_err(super::error)?
        != crate::authoring::identity("lattice.source-draft.v1", draft).map_err(super::error)?
    {
        return Err(super::error("durable draft changed since preview"));
    }
    Ok(())
}
