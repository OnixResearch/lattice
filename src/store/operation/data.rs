const OPERATION_DATA_RECORDS: TableDefinition<&str, &[u8]> = TableDefinition::new("operation_data_records_v1");
const OPERATION_DATA_LOGICAL_INDEX: TableDefinition<&str, &str> =
    TableDefinition::new("operation_data_logical_index_v1");
const OPERATION_DATA_TAG_INDEX: TableDefinition<&str, &str> = TableDefinition::new("operation_data_tag_index_v1");
const OPERATION_DATA_SCHEMA_INDEX: TableDefinition<&str, &str> = TableDefinition::new("operation_data_schema_index_v1");
const OPERATION_DATA_PRODUCER_INDEX: TableDefinition<&str, &str> =
    TableDefinition::new("operation_data_producer_index_v1");

fn ensure_operation_data_tables(transaction: &redb::WriteTransaction) -> Result<(), StoreError> {
    let _ = transaction
        .open_table(OPERATION_DATA_RECORDS)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    let _ = transaction
        .open_table(OPERATION_DATA_LOGICAL_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    let _ = transaction
        .open_table(OPERATION_DATA_TAG_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    let _ = transaction
        .open_table(OPERATION_DATA_SCHEMA_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    let _ = transaction
        .open_table(OPERATION_DATA_PRODUCER_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    Ok(())
}

impl Store {
    pub fn append_operation_data_metadata(
        &self,
        metadata: &crate::operation_data::RecordMetadata,
    ) -> Result<crate::operation_data::PublicationDecision, StoreError> {
        crate::operation_data::validate_metadata(metadata).map_err(|error| StoreError::Conflict(error.to_string()))?;
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let existing = {
            let table = transaction
                .open_table(OPERATION_DATA_RECORDS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            let value =
                table.get(metadata.version_identity.as_str()).map_err(|error| StoreError::Db(error.to_string()))?;
            deserialize_optional(value)?
        };
        let decision = crate::operation_data::publication_decision(existing.as_ref(), metadata);
        match decision {
            crate::operation_data::PublicationDecision::Idempotent => return Ok(decision),
            crate::operation_data::PublicationDecision::Conflict => {
                return Err(StoreError::Conflict(
                    "operation data version identity conflicts with durable metadata".to_string(),
                ));
            }
            crate::operation_data::PublicationDecision::Append => {}
        }
        let bytes = serialize(metadata)?;
        {
            let mut table = transaction
                .open_table(OPERATION_DATA_RECORDS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            table
                .insert(metadata.version_identity.as_str(), bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        insert_operation_data_indexes(&transaction, metadata)?;
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(decision)
    }

    pub fn load_operation_data_metadata(
        &self,
        version_identity: &str,
    ) -> Result<Option<crate::operation_data::RecordMetadata>, StoreError> {
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = transaction
            .open_table(OPERATION_DATA_RECORDS)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        let metadata =
            deserialize_optional(table.get(version_identity).map_err(|error| StoreError::Db(error.to_string()))?)?;
        if let Some(metadata) = &metadata {
            crate::operation_data::validate_metadata(metadata)
                .map_err(|error| StoreError::Conflict(error.to_string()))?;
        }
        Ok(metadata)
    }

    pub fn list_operation_data_metadata(&self) -> Result<Vec<crate::operation_data::RecordMetadata>, StoreError> {
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = transaction
            .open_table(OPERATION_DATA_RECORDS)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        let mut records = Vec::with_capacity(crate::operation_data::MAX_QUERY_CANDIDATES);
        for entry in table.iter().map_err(|error| StoreError::Db(error.to_string()))? {
            if records.len() == crate::operation_data::MAX_QUERY_CANDIDATES {
                return Err(StoreError::Conflict(
                    "operation data metadata exceeds the bounded query candidate count".to_string(),
                ));
            }
            let (_, value) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            let metadata: crate::operation_data::RecordMetadata = deserialize(value.value())?;
            crate::operation_data::validate_metadata(&metadata)
                .map_err(|error| StoreError::Conflict(error.to_string()))?;
            records.push(metadata);
        }
        records.sort_by(|left, right| {
            left.logical_key
                .cmp(&right.logical_key)
                .then_with(|| left.sequence.cmp(&right.sequence))
                .then_with(|| left.version_identity.cmp(&right.version_identity))
        });
        Ok(records)
    }

    pub fn delete_operation_data_metadata(
        &self,
        version_identity: &str,
    ) -> Result<Option<crate::operation_data::RecordMetadata>, StoreError> {
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let metadata = {
            let table = transaction
                .open_table(OPERATION_DATA_RECORDS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            deserialize_optional(table.get(version_identity).map_err(|error| StoreError::Db(error.to_string()))?)?
        };
        let Some(metadata) = metadata else {
            return Ok(None);
        };
        crate::operation_data::validate_metadata(&metadata).map_err(|error| StoreError::Conflict(error.to_string()))?;
        remove_operation_data_indexes(&transaction, &metadata)?;
        {
            let mut table = transaction
                .open_table(OPERATION_DATA_RECORDS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            table.remove(version_identity).map_err(|error| StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(Some(metadata))
    }
}

fn insert_operation_data_indexes(
    transaction: &redb::WriteTransaction,
    metadata: &crate::operation_data::RecordMetadata,
) -> Result<(), StoreError> {
    debug_assert!(crate::operation_data::validate_metadata(metadata).is_ok());
    debug_assert!(crate::operation::identity::validate(&metadata.version_identity));
    let logical_key = logical_index_key(metadata);
    {
        let mut table = transaction
            .open_table(OPERATION_DATA_LOGICAL_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .insert(logical_key.as_str(), metadata.version_identity.as_str())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    if let Some(schema) = &metadata.payload.schema_identity {
        let key = composite_index_key(CompositeIndexKey {
            namespace: schema,
            version_identity: &metadata.version_identity,
        });
        let mut table = transaction
            .open_table(OPERATION_DATA_SCHEMA_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .insert(key.as_str(), metadata.version_identity.as_str())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    {
        let key = composite_index_key(CompositeIndexKey {
            namespace: &metadata.producer.invocation,
            version_identity: &metadata.version_identity,
        });
        let mut table = transaction
            .open_table(OPERATION_DATA_PRODUCER_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .insert(key.as_str(), metadata.version_identity.as_str())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    let mut table = transaction
        .open_table(OPERATION_DATA_TAG_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    for (key, value) in &metadata.tags {
        let index = tag_index_key(TagIndexKey {
            key,
            value,
            version_identity: &metadata.version_identity,
        });
        table
            .insert(index.as_str(), metadata.version_identity.as_str())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    Ok(())
}

fn remove_operation_data_indexes(
    transaction: &redb::WriteTransaction,
    metadata: &crate::operation_data::RecordMetadata,
) -> Result<(), StoreError> {
    debug_assert!(crate::operation_data::validate_metadata(metadata).is_ok());
    debug_assert!(crate::operation::identity::validate(&metadata.version_identity));
    {
        let mut table = transaction
            .open_table(OPERATION_DATA_LOGICAL_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .remove(logical_index_key(metadata).as_str())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    if let Some(schema) = &metadata.payload.schema_identity {
        let mut table = transaction
            .open_table(OPERATION_DATA_SCHEMA_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .remove(
                composite_index_key(CompositeIndexKey {
                    namespace: schema,
                    version_identity: &metadata.version_identity,
                })
                .as_str(),
            )
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    {
        let mut table = transaction
            .open_table(OPERATION_DATA_PRODUCER_INDEX)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        table
            .remove(
                composite_index_key(CompositeIndexKey {
                    namespace: &metadata.producer.invocation,
                    version_identity: &metadata.version_identity,
                })
                .as_str(),
            )
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    let mut table = transaction
        .open_table(OPERATION_DATA_TAG_INDEX)
        .map_err(|error| StoreError::Table(error.to_string()))?;
    for (key, value) in &metadata.tags {
        table
            .remove(
                tag_index_key(TagIndexKey {
                    key,
                    value,
                    version_identity: &metadata.version_identity,
                })
                .as_str(),
            )
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    Ok(())
}

fn logical_index_key(metadata: &crate::operation_data::RecordMetadata) -> String {
    format!("{}\0{:020}\0{}", metadata.logical_key, metadata.sequence, metadata.version_identity)
}

struct CompositeIndexKey<'a> {
    namespace: &'a str,
    version_identity: &'a str,
}

fn composite_index_key(input: CompositeIndexKey<'_>) -> String {
    format!("{}\0{}", input.namespace, input.version_identity)
}

struct TagIndexKey<'a> {
    key: &'a str,
    value: &'a str,
    version_identity: &'a str,
}

fn tag_index_key(input: TagIndexKey<'_>) -> String {
    format!("{}\0{}\0{}", input.key, input.value, input.version_identity)
}
