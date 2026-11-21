//! Version-checked persistence of agent revision lifecycles (Lattice #25).
//!
//! One row per definition identity holds the lifecycle and a version that
//! increases by one on every committed transition. A commit names the version
//! it read; any other current version is a conflict, and the caller reloads
//! and reapplies its pure transition. Version 0 means no row exists.

// r[impl lattice.agent_revisions.persistence]

use redb::ReadableDatabase;
use redb::ReadableTable;

const LIFECYCLES: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("agent_revision_lifecycles_v1");
const LIFECYCLE_VERSION_INCREMENT: u64 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    version: u64,
    lifecycle: lattice_core::agent_revision::state::Lifecycle,
}

fn decode(bytes: &[u8], definition: &str) -> Result<Row, super::StoreError> {
    let row: Row = super::deserialize(bytes)?;
    if row.lifecycle.definition != definition || row.version == 0 {
        return Err(super::StoreError::Conflict("agent revision lifecycle row is inconsistent".to_owned()));
    }
    Ok(row)
}

impl super::Store {
    /// The stored lifecycle of `definition` and its version, if any.
    pub fn load_lifecycle(
        &self,
        definition: &str,
    ) -> Result<Option<(u64, lattice_core::agent_revision::state::Lifecycle)>, super::StoreError> {
        let transaction = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let table = match transaction.open_table(LIFECYCLES) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(error) => return Err(super::StoreError::Table(error.to_string())),
        };
        let stored = table.get(definition).map_err(|error| super::StoreError::Db(error.to_string()))?;
        let row = stored.map(|guard| decode(guard.value(), definition)).transpose()?;
        Ok(row.map(|row| (row.version, row.lifecycle)))
    }

    /// Commit `lifecycle` if its stored version is still `expected_version`,
    /// and return the new version.
    pub fn commit_lifecycle(
        &self,
        expected_version: u64,
        lifecycle: &lattice_core::agent_revision::state::Lifecycle,
    ) -> Result<u64, super::StoreError> {
        let definition = lifecycle.definition.as_str();
        let version = expected_version
            .checked_add(LIFECYCLE_VERSION_INCREMENT)
            .ok_or_else(|| super::StoreError::Conflict("agent revision lifecycle version is exhausted".to_owned()))?;
        let transaction = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        {
            let mut table =
                transaction.open_table(LIFECYCLES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            let current = {
                let stored = table.get(definition).map_err(|error| super::StoreError::Db(error.to_string()))?;
                stored.map(|guard| decode(guard.value(), definition)).transpose()?.map_or(0, |row| row.version)
            };
            if current != expected_version {
                return Err(super::StoreError::Conflict("agent revision lifecycle version changed".to_owned()));
            }
            let data = super::serialize(&Row {
                version,
                lifecycle: lifecycle.clone(),
            })?;
            table
                .insert(definition, data.as_slice())
                .map_err(|error| super::StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| super::StoreError::Db(error.to_string()))?;
        debug_assert!(version > expected_version);
        debug_assert_eq!(version.checked_sub(expected_version), Some(LIFECYCLE_VERSION_INCREMENT));
        Ok(version)
    }
}
