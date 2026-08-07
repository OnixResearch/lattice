use content_identity_core::Domain;
use content_identity_core::IdentityError;
use content_identity_core::IdentityHasher;
use lattice::content_identity::SCHEDULE_OCCURRENCE_DOMAIN;
use lattice::content_identity::ScheduleOccurrenceInput;
use lattice::content_identity::schedule_occurrence;

const SCHEDULE_ID: &str = "nightly";
const DUE_EPOCH_MS: i64 = 1_000;
const SCHEDULE_ID_TAG: &[u8] = b"schedule-id";
const DUE_EPOCH_MS_TAG: &[u8] = b"due-epoch-ms";
const EXPECTED_IDENTITY: &str = "b3:0f3b3ea4c2e1cb44d54ab759e6c69cd6e1d91af09a8bef1b503aa81dc176596c";
const SHARED_IMPORT: &str = "content_identity_core";
const WRAPPER_IMPORT: &str = "crate::content_identity";

fn legacy_schedule_occurrence_identity(schedule_id: &str, due_epoch_ms: i64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SCHEDULE_OCCURRENCE_DOMAIN.as_bytes());
    update_legacy_part(&mut hasher, SCHEDULE_ID_TAG, schedule_id.as_bytes());
    update_legacy_part(&mut hasher, DUE_EPOCH_MS_TAG, &due_epoch_ms.to_be_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

fn update_legacy_part(hasher: &mut blake3::Hasher, tag: &[u8], value: &[u8]) {
    let mut part = blake3::Hasher::new();
    part.update(tag);
    part.update(value);
    hasher.update(part.finalize().as_bytes());
}

fn forbidden_shared_import<'a>(source: &'a str, forbidden: &'a [&str]) -> Option<&'a str> {
    forbidden.iter().copied().find(|term| source.contains(term))
}

#[test]
fn schedule_occurrence_wrapper_matches_fixed_legacy_vector() -> Result<(), IdentityError> {
    let legacy = legacy_schedule_occurrence_identity(SCHEDULE_ID, DUE_EPOCH_MS);
    let shared = schedule_occurrence(ScheduleOccurrenceInput {
        schedule_id: SCHEDULE_ID,
        due_epoch_ms: DUE_EPOCH_MS,
    })?;

    assert_eq!(legacy, EXPECTED_IDENTITY);
    assert_eq!(shared, EXPECTED_IDENTITY);
    assert_eq!(shared, legacy);
    Ok(())
}

#[test]
fn wrong_domain_order_and_field_omission_change_the_identity() -> Result<(), IdentityError> {
    let expected = schedule_occurrence(ScheduleOccurrenceInput {
        schedule_id: SCHEDULE_ID,
        due_epoch_ms: DUE_EPOCH_MS,
    })?;

    let mut wrong_domain = IdentityHasher::new(Domain::new("node-execution:v1")?);
    wrong_domain
        .update_hashed_part(SCHEDULE_ID_TAG, SCHEDULE_ID.as_bytes())
        .update_hashed_part(DUE_EPOCH_MS_TAG, &DUE_EPOCH_MS.to_be_bytes());

    let mut wrong_order = IdentityHasher::new(Domain::new(SCHEDULE_OCCURRENCE_DOMAIN)?);
    wrong_order
        .update_hashed_part(DUE_EPOCH_MS_TAG, &DUE_EPOCH_MS.to_be_bytes())
        .update_hashed_part(SCHEDULE_ID_TAG, SCHEDULE_ID.as_bytes());

    let mut omitted = IdentityHasher::new(Domain::new(SCHEDULE_OCCURRENCE_DOMAIN)?);
    omitted.update_hashed_part(SCHEDULE_ID_TAG, SCHEDULE_ID.as_bytes());

    assert_ne!(expected, format!("b3:{}", wrong_domain.finish().to_hex()));
    assert_ne!(expected, format!("b3:{}", wrong_order.finish().to_hex()));
    assert_ne!(expected, format!("b3:{}", omitted.finish().to_hex()));
    Ok(())
}

#[test]
fn invalid_domains_and_duplicate_inputs_fail_closed() -> Result<(), IdentityError> {
    assert!(matches!(Domain::new("lattice-schedule-occurrence"), Err(IdentityError::UnversionedDomain { .. })));
    assert!(matches!(Domain::new("lattice-vault-stream-v1"), Err(IdentityError::SecuritySensitiveDomain { .. })));

    let mut hasher = IdentityHasher::new(Domain::new(SCHEDULE_OCCURRENCE_DOMAIN)?);
    assert!(matches!(
        hasher.update_sorted_map(b"inputs", &[("key", "one"), ("key", "two")]),
        Err(IdentityError::DuplicateMapKey { .. })
    ));
    Ok(())
}

#[test]
fn security_owners_do_not_import_shared_content_identity() {
    let credential_source = include_str!("../src/credentials.rs");
    let trigger_source = include_str!("../src/triggers.rs");
    let forbidden = [SHARED_IMPORT, WRAPPER_IMPORT];

    assert_eq!(forbidden_shared_import(credential_source, &forbidden), None);
    assert_eq!(forbidden_shared_import(trigger_source, &forbidden), None);
    assert_eq!(
        forbidden_shared_import("use content_identity_core::IdentityHasher;", &forbidden),
        Some(SHARED_IMPORT)
    );
}
