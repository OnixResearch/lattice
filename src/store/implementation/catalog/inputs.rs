//! Retained template bindings for saved workflows and the input identities of bound runs.
//!
//! Both tables are created on first write. A store that never saved a bound template or ran
//! with inputs has neither table, so its bytes match a store from before run inputs existed.
use redb::ReadableDatabase as _;

const TEMPLATE_BINDINGS: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("workflow_template_bindings_v1");
const RUN_INPUTS: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("run_inputs_v1");

/// The persisted shape of `RunInputRecord`, borrowing the identity it records.
#[derive(serde::Serialize)]
struct RunInputRecordRef<'a> {
    run_id: crate::engine::RunId,
    inputs: &'a crate::workflow_template::inputs::InputIdentity,
}

fn db_error(error: impl std::fmt::Display) -> crate::store::StoreError {
    crate::store::StoreError::Db(error.to_string())
}

fn table_error(error: impl std::fmt::Display) -> crate::store::StoreError {
    crate::store::StoreError::Table(error.to_string())
}

/// Open a table that may not exist yet; a missing table reads as empty.
fn open_optional<K: redb::Key + 'static>(
    transaction: &redb::ReadTransaction,
    definition: redb::TableDefinition<K, &'static [u8]>,
) -> Result<Option<redb::ReadOnlyTable<K, &'static [u8]>>, crate::store::StoreError> {
    match transaction.open_table(definition) {
        Ok(table) => Ok(Some(table)),
        Err(redb::TableError::TableDoesNotExist(_)) => Ok(None),
        Err(error) => Err(table_error(error)),
    }
}

fn read_by_name<T: serde::de::DeserializeOwned>(
    transaction: &redb::ReadTransaction,
    definition: redb::TableDefinition<&'static str, &'static [u8]>,
    name: &str,
) -> Result<Option<T>, crate::store::StoreError> {
    let Some(table) = open_optional(transaction, definition)? else {
        return Ok(None);
    };
    crate::store::deserialize_optional(table.get(name).map_err(db_error)?)
}

impl crate::store::Store {
    // r[impl lattice.workflow_templates.run_inputs.retained]
    /// Retain the template binding of a saved workflow, keyed by its workflow name.
    ///
    /// A later save of the same name replaces it. A binding whose workflow revision no longer
    /// matches the saved workflow is stale, and inputs for that workflow are refused.
    pub fn save_workflow_template_binding(
        &self,
        binding: &crate::workflow_template::inputs::RetainedBinding,
    ) -> Result<(), crate::store::StoreError> {
        let data = crate::store::serialize(binding)?;
        let transaction = self.db.begin_write().map_err(db_error)?;
        {
            let mut table = transaction.open_table(TEMPLATE_BINDINGS).map_err(table_error)?;
            table.insert(binding.workflow.workflow_name.as_str(), data.as_slice()).map_err(db_error)?;
        }
        transaction.commit().map_err(db_error)
    }

    pub fn load_workflow_template_binding(
        &self,
        workflow_name: &str,
    ) -> Result<Option<crate::workflow_template::inputs::RetainedBinding>, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(db_error)?;
        read_by_name(&transaction, TEMPLATE_BINDINGS, workflow_name)
    }

    /// Load a saved workflow and its retained binding without creating a database, tables, or
    /// write transactions.
    pub fn load_saved_template_facts_read_only(
        path: &std::path::Path,
        workflow_name: &str,
    ) -> Result<
        (Option<crate::engine::Workflow>, Option<crate::workflow_template::inputs::RetainedBinding>),
        crate::store::StoreError,
    > {
        let database = redb::ReadOnlyDatabase::open(path).map_err(db_error)?;
        let transaction = database.begin_read().map_err(db_error)?;
        let workflow = read_by_name::<crate::engine::Workflow>(&transaction, crate::store::WORKFLOWS, workflow_name)?
            .map(super::authoring::with_revision)
            .transpose()?;
        let binding = read_by_name(&transaction, TEMPLATE_BINDINGS, workflow_name)?;
        debug_assert!(workflow.as_ref().is_none_or(|workflow| workflow.meta.name == workflow_name));
        Ok((workflow, binding))
    }

    // r[impl lattice.workflow_templates.run_inputs.record]
    /// Reserve a run id and record its admitted input identity in one transaction, after every
    /// input check passed and before the first node runs.
    pub fn reserve_run_with_inputs(
        &self,
        inputs: &crate::workflow_template::inputs::InputIdentity,
    ) -> Result<crate::engine::RunId, crate::store::StoreError> {
        let transaction = self.db.begin_write().map_err(db_error)?;
        let run_id = crate::store::reserve_run_id(&transaction)?;
        // Serializes exactly as `RunInputRecord` without copying the identity.
        let data = crate::store::serialize(&RunInputRecordRef { run_id, inputs })?;
        {
            let mut table = transaction.open_table(RUN_INPUTS).map_err(table_error)?;
            if table.insert(run_id.0, data.as_slice()).map_err(db_error)?.is_some() {
                return Err(crate::store::StoreError::Conflict(format!(
                    "run identity {run_id} already has recorded inputs"
                )));
            }
        }
        transaction.commit().map_err(db_error)?;
        debug_assert!(!data.is_empty());
        Ok(run_id)
    }

    pub fn load_run_inputs(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<crate::workflow_template::inputs::RunInputRecord>, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(db_error)?;
        let Some(table) = open_optional(&transaction, RUN_INPUTS)? else {
            return Ok(None);
        };
        crate::store::deserialize_optional(table.get(run_id.0).map_err(db_error)?)
    }
}
