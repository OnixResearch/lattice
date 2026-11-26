//! Snapshot the edit base and commit only a successfully compiled draft.
use redb::ReadableDatabase;
use redb::ReadableTable;

use crate::authoring::ContentIdentity;
use crate::authoring::SourceDraft;
use crate::authoring::source_edit::ExpectedDraft;
use crate::authoring::source_edit::NativeReceipt;
use crate::authoring::source_edit::Rejection;
use crate::authoring::source_edit::draft_revision;
use crate::engine::Workflow;

impl crate::store::Store {
    // r[impl lattice.source_edit.view]
    pub(crate) fn source_edit_snapshot(
        &self,
        workflow: &str,
        head: &ContentIdentity,
        draft_id: &str,
    ) -> Result<(Workflow, Option<SourceDraft>), Rejection> {
        source_edit_snapshot(self.db.as_ref(), workflow, head, draft_id)
    }

    pub(crate) fn read_source_edit_snapshot(
        path: &std::path::Path,
        workflow: &str,
        head: &ContentIdentity,
        draft_id: &str,
    ) -> Result<(Workflow, Option<SourceDraft>), Rejection> {
        let db = redb::ReadOnlyDatabase::open(path).map_err(|_| Rejection::StoreError)?;
        source_edit_snapshot(&db, workflow, head, draft_id)
    }

    // r[impl lattice.source_edit.receipt]
    pub(crate) fn save_source_edit(
        &self,
        mut draft: SourceDraft,
        expected: &ExpectedDraft,
    ) -> Result<(ContentIdentity, ContentIdentity, NativeReceipt), Rejection> {
        self.ensure_not_quarantined().map_err(|_| Rejection::StoreError)?;
        crate::authoring::shell::preview(&mut draft).map_err(|_| Rejection::ValidationFailed)?;
        let preview = draft.preview.as_ref().ok_or(Rejection::ValidationFailed)?;
        let source = preview.candidate.authoring.as_ref().ok_or(Rejection::ValidationFailed)?;
        if source.compiler != preview.compiler {
            return Err(Rejection::ValidationFailed);
        }
        let source_revision = preview.input_identity.clone();
        let receipt = NativeReceipt {
            compiler: preview.compiler.clone(),
            evaluated_output: source.evaluated_output.clone(),
            expanded_artifact: source.expanded_artifact.clone(),
        };
        let revision = draft_revision(&draft).map_err(|_| Rejection::StoreError)?;
        self.commit_source_edit(&draft, expected)?;
        Ok((revision, source_revision, receipt))
    }

    // r[impl lattice.source_edit.save]
    fn commit_source_edit(&self, draft: &SourceDraft, expected: &ExpectedDraft) -> Result<(), Rejection> {
        self.ensure_not_quarantined().map_err(|_| Rejection::StoreError)?;
        let head = draft.expected_revision.as_ref().ok_or(Rejection::HeadConflict)?;
        let bytes = crate::store::serialize(draft).map_err(|_| Rejection::StoreError)?;
        let txn = self.db.begin_write().map_err(|_| Rejection::StoreError)?;
        crate::store::quarantine::require_no_pending_reopen(&txn).map_err(|_| Rejection::StoreError)?;
        {
            let workflows = txn.open_table(crate::store::WORKFLOWS).map_err(|_| Rejection::StoreError)?;
            require_head(&workflows, &draft.workflow_name, head)?;
        }
        {
            let mut drafts = txn.open_table(super::DRAFTS).map_err(|_| Rejection::StoreError)?;
            let current = read_draft(&drafts, &draft.id)?;
            require_draft_target(current.as_ref(), &draft.workflow_name, head)?;
            let current_revision =
                current.as_ref().map(draft_revision).transpose().map_err(|_| Rejection::StoreError)?;
            if current_revision != expected.0 {
                return Err(Rejection::DraftConflict);
            }
            drafts.insert(draft.id.as_str(), bytes.as_slice()).map_err(|_| Rejection::StoreError)?;
        }
        txn.commit().map_err(|_| Rejection::CommitUnknown)
    }
}

fn source_edit_snapshot(
    db: &impl ReadableDatabase,
    workflow: &str,
    head: &ContentIdentity,
    draft_id: &str,
) -> Result<(Workflow, Option<SourceDraft>), Rejection> {
    let txn = db.begin_read().map_err(|_| Rejection::StoreError)?;
    let workflows = txn.open_table(crate::store::WORKFLOWS).map_err(|_| Rejection::StoreError)?;
    let workflow = require_head(&workflows, workflow, head)?;
    let draft = match txn.open_table(super::DRAFTS) {
        Ok(table) => read_draft(&table, draft_id)?,
        Err(redb::TableError::TableDoesNotExist(_)) => None,
        Err(_) => return Err(Rejection::StoreError),
    };
    require_draft_target(draft.as_ref(), &workflow.meta.name, head)?;
    Ok((workflow, draft))
}

fn require_head(
    table: &impl ReadableTable<&'static str, &'static [u8]>,
    name: &str,
    expected: &ContentIdentity,
) -> Result<Workflow, Rejection> {
    let row = table.get(name).map_err(|_| Rejection::StoreError)?.ok_or(Rejection::HeadConflict)?;
    let workflow: Workflow = crate::store::deserialize(row.value()).map_err(|_| Rejection::StoreError)?;
    let workflow = super::with_revision(workflow).map_err(|_| Rejection::StoreError)?;
    if workflow.catalog_revision.as_ref() != Some(expected) || workflow.meta.name != name {
        return Err(Rejection::HeadConflict);
    }
    if workflow.authoring.is_none() {
        return Err(Rejection::SourceUnavailable);
    }
    Ok(workflow)
}

fn read_draft(
    table: &impl ReadableTable<&'static str, &'static [u8]>,
    id: &str,
) -> Result<Option<SourceDraft>, Rejection> {
    let draft: Option<SourceDraft> = table
        .get(id)
        .map_err(|_| Rejection::StoreError)?
        .map(|row| crate::store::deserialize(row.value()).map_err(|_| Rejection::StoreError))
        .transpose()?;
    if draft.as_ref().is_some_and(|draft| draft.id != id) {
        return Err(Rejection::StoreError);
    }
    Ok(draft)
}

fn require_draft_target(
    draft: Option<&SourceDraft>,
    workflow: &str,
    expected_head: &ContentIdentity,
) -> Result<(), Rejection> {
    if let Some(draft) = draft {
        if draft.workflow_name != workflow {
            return Err(Rejection::DraftTargetMismatch);
        }
        if draft.expected_revision.as_ref() != Some(expected_head) {
            return Err(Rejection::HeadConflict);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::AssociationState;
    use crate::authoring::CapturedProgram;
    use crate::authoring::ProgramKind;
    use crate::engine::workflow::WorkflowTimestamp;
    use crate::store::Store;

    // r[verify lattice.source_edit.save]
    #[test]
    fn source_edit_commit_rejects_both_interleaved_state_changes() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::new(directory.path().join("state.redb")).unwrap();
        let program = CapturedProgram {
            entry: "entry.ncl".into(),
            files: [("entry.ncl".into(), b"original".to_vec())].into(),
            kind: ProgramKind::Workflow,
            parameters: Default::default(),
        };
        let mut workflow = Workflow::new("test", WorkflowTimestamp::epoch());
        workflow.add_shell_node("node", "printf must-not-run");
        let workflow = crate::authoring::shell::admit_evaluated_snapshot(
            &program,
            serde_json::to_value(crate::workflow_format::workflow_to_portable(&workflow)).unwrap(),
            AssociationState::RetainedEntrySnapshot,
        )
        .unwrap();
        let mut head = store.admit_imported_workflow(&workflow, None).unwrap();
        let mut candidate = SourceDraft {
            id: "edit".into(),
            workflow_name: "test".into(),
            expected_revision: head.catalog_revision.clone(),
            program,
            diagnostic: None,
            preview: None,
        };

        // Exercise the terminal transaction with a simulated precomputed
        // candidate. The real save path first obtains native validation.
        let (_, observed) =
            store.source_edit_snapshot("test", head.catalog_revision.as_ref().unwrap(), "edit").unwrap();
        assert!(observed.is_none());
        store.save_source_draft(&candidate).unwrap();
        assert_eq!(store.commit_source_edit(&candidate, &ExpectedDraft(None)), Err(Rejection::DraftConflict));
        let expected = ExpectedDraft(Some(draft_revision(&candidate).unwrap()));
        candidate.program.files.insert("entry.ncl".into(), b"new".to_vec());
        let old = head.catalog_revision.clone();
        head.meta.description = "interleaved materialized edit".into();
        let newer = store.save_workflow_cas(&head, old.as_ref()).unwrap();
        assert_ne!(newer.catalog_revision, old);
        assert_eq!(store.commit_source_edit(&candidate, &expected), Err(Rejection::HeadConflict));
        assert_eq!(store.load_workflow("test").unwrap().unwrap().catalog_revision, newer.catalog_revision);
        let unchanged = store.load_source_draft("edit").unwrap().unwrap();
        assert_eq!(Some(draft_revision(&unchanged).unwrap()), expected.0);

        // With unchanged head, a competing draft replacement still beats a
        // formerly valid, non-null draft reservation.
        candidate.expected_revision = newer.catalog_revision;
        store.save_source_draft(&candidate).unwrap();
        assert_eq!(store.commit_source_edit(&candidate, &expected), Err(Rejection::DraftConflict));
        let current = store.load_source_draft("edit").unwrap().unwrap();
        assert_eq!(draft_revision(&current).unwrap(), draft_revision(&candidate).unwrap());
    }
}
