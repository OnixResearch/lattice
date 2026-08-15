#[derive(serde::Serialize)]
struct RecordIdentityMaterial<'a> {
    logical_key: &'a str,
    record_class: RecordClass,
    operation_type: &'a str,
    type_revision: &'a str,
    instance: &'a str,
    method: &'a str,
    output_name: &'a str,
    output_contract_identity: &'a str,
    schema_identity: &'a Option<String>,
    media_class: &'a Option<String>,
    selectable_paths: &'a [Vec<String>],
}

#[derive(serde::Serialize)]
struct VersionIdentityMaterial<'a> {
    record_identity: &'a str,
    sequence: u64,
    producer: &'a Producer,
    tags: &'a std::collections::BTreeMap<String, String>,
    sensitivity: SensitivityClass,
    lifetime: &'a Lifetime,
    retention: &'a RetentionPolicy,
    outcome: &'a str,
    payload: &'a PayloadDescriptor,
    required_non_claims: &'a [String],
}

fn record_identity(metadata: &RecordMetadata) -> Result<String, Error> {
    hash_value(RECORD_DOMAIN, &RecordIdentityMaterial {
        logical_key: &metadata.logical_key,
        record_class: metadata.record_class,
        operation_type: &metadata.producer.operation_type,
        type_revision: &metadata.producer.type_revision,
        instance: &metadata.producer.instance,
        method: &metadata.producer.method,
        output_name: &metadata.output_name,
        output_contract_identity: &metadata.output_contract_identity,
        schema_identity: &metadata.payload.schema_identity,
        media_class: &metadata.payload.media_class,
        selectable_paths: &metadata.payload.selectable_paths,
    })
}

fn version_identity(metadata: &RecordMetadata) -> Result<String, Error> {
    hash_value(VERSION_DOMAIN, &VersionIdentityMaterial {
        record_identity: &metadata.record_identity,
        sequence: metadata.sequence,
        producer: &metadata.producer,
        tags: &metadata.tags,
        sensitivity: metadata.sensitivity,
        lifetime: &metadata.lifetime,
        retention: &metadata.retention,
        outcome: &metadata.outcome,
        payload: &metadata.payload,
        required_non_claims: &metadata.required_non_claims,
    })
}

pub(super) fn hash_value<T: serde::Serialize + ?Sized>(domain: &str, value: &T) -> Result<String, Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| Error::Encoding)?;
    Ok(crate::operation::identity::hash_bytes(domain, &bytes))
}
