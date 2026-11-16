mod catalog;
mod connection;
mod coordination;
mod deliveries;
pub(super) mod model;
pub(super) mod ports;
mod runtime;

pub(super) fn entry_count(table: &impl redb::ReadableTableMetadata) -> Result<usize, crate::store::StoreError> {
    let count = table.len().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
    usize::try_from(count).map_err(|error| crate::store::StoreError::Db(error.to_string()))
}
