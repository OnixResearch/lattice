pub(super) const LABELS_FIELD: &str = "receipt_labels";
pub(super) const COUNTS_FIELD: &str = "receipt_counts";
pub(super) const HASHES_FIELD: &str = "receipt_artifact_hashes";
pub(super) const REDACTIONS_FIELD: &str = "receipt_redactions";
pub(super) const BACKEND_LABEL_FIELD: MetadataField = MetadataField::new("receipt_backend_label");
pub(super) const INTERCHANGE_LABEL_FIELD: MetadataField = MetadataField::new("receipt_interchange_label");
pub(super) const LABEL_FIELD: MetadataField = MetadataField::new("receipt_label");
const COUNT_LABEL_FIELD: MetadataField = MetadataField::new("receipt_count_label");
const NUL_CHARACTER: char = '\0';

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MetadataField {
    name: &'static str,
}

impl MetadataField {
    const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub(super) const fn name(self) -> &'static str {
        self.name
    }
}

pub(super) fn labels(values: &[&str]) -> Result<alloc::vec::Vec<alloc::string::String>, super::Error> {
    ensure_required_collection(LABELS_FIELD, values.len())?;
    let mut labels = values
        .iter()
        .map(|value| required_label(LABEL_FIELD, value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()?;
    labels.sort();
    labels.dedup();
    Ok(labels)
}

pub(super) fn counts(
    values: &[super::BoundedCountDraft<'_>],
) -> Result<alloc::vec::Vec<super::BoundedCount>, super::Error> {
    ensure_optional_collection(COUNTS_FIELD, values.len())?;
    let mut counts = values.iter().map(count).collect::<Result<alloc::vec::Vec<_>, _>>()?;
    counts.sort();
    counts.dedup();
    Ok(counts)
}

pub(super) fn hashes(values: &[crate::ArtifactHash]) -> Result<alloc::vec::Vec<crate::ArtifactHash>, super::Error> {
    ensure_optional_collection(HASHES_FIELD, values.len())?;
    let mut hashes = values.to_vec();
    hashes.sort();
    hashes.dedup();
    Ok(hashes)
}

pub(super) fn redactions(values: &[super::Redaction]) -> Result<alloc::vec::Vec<super::Redaction>, super::Error> {
    ensure_required_collection(REDACTIONS_FIELD, values.len())?;
    let mut redactions = values.to_vec();
    redactions.sort();
    redactions.dedup();
    Ok(redactions)
}

pub(super) fn backend_label(value: Option<&str>) -> Result<Option<alloc::string::String>, super::Error> {
    optional_label(BACKEND_LABEL_FIELD, value)
}

pub(super) fn interchange_label(value: Option<&str>) -> Result<Option<alloc::string::String>, super::Error> {
    optional_label(INTERCHANGE_LABEL_FIELD, value)
}

fn count(value: &super::BoundedCountDraft<'_>) -> Result<super::BoundedCount, super::Error> {
    let label = required_label(COUNT_LABEL_FIELD, value.label)?;
    if value.value > super::MAX_COUNT {
        return Err(super::Error::CountTooLarge {
            field: COUNTS_FIELD,
            max: super::MAX_COUNT,
            actual: value.value,
        });
    }
    Ok(super::BoundedCount::new(label, value.value))
}

fn ensure_required_collection(field: &'static str, actual_items: usize) -> Result<(), super::Error> {
    ensure_optional_collection(field, actual_items)?;
    if actual_items == 0 {
        return Err(super::Error::EmptyCollection { field });
    }
    Ok(())
}

fn ensure_optional_collection(field: &'static str, actual_items: usize) -> Result<(), super::Error> {
    if actual_items > super::MAX_COLLECTION_ITEMS {
        return Err(super::Error::CollectionTooLarge {
            field,
            max_items: super::MAX_COLLECTION_ITEMS,
            actual_items,
        });
    }
    Ok(())
}

fn optional_label(field: MetadataField, value: Option<&str>) -> Result<Option<alloc::string::String>, super::Error> {
    value.map(|label| required_label(field, label)).transpose()
}

fn required_label(field: MetadataField, value: &str) -> Result<alloc::string::String, super::Error> {
    assert!(!field.name().is_empty(), "metadata field name must not be empty");
    assert!(value.len() >= value.chars().count(), "label byte length must cover character count");
    reject_oversized(field, value)?;
    reject_nul(field, value)?;
    let normalized = value.trim();
    reject_empty(field, normalized)?;
    reject_unsafe(field, normalized)?;
    Ok(alloc::string::String::from(normalized))
}

fn reject_oversized(field: MetadataField, value: &str) -> Result<(), super::Error> {
    let actual_chars = value.chars().count();
    if actual_chars > super::MAX_LABEL_CHARS {
        return Err(super::Error::UnsafeMetadata {
            field: field.name(),
            class: super::LeakClass::UnboundedOutput,
        });
    }
    Ok(())
}

fn reject_nul(field: MetadataField, value: &str) -> Result<(), super::Error> {
    if let Some((byte_index, _character)) =
        value.char_indices().find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(super::Error::NulCharacter {
            field: field.name(),
            byte_index,
        });
    }
    Ok(())
}

fn reject_empty(field: MetadataField, value: &str) -> Result<(), super::Error> {
    if value.is_empty() {
        return Err(super::Error::EmptyField { field: field.name() });
    }
    Ok(())
}

fn reject_unsafe(field: MetadataField, value: &str) -> Result<(), super::Error> {
    if let Some(class) = super::leak::class(value) {
        return Err(super::Error::UnsafeMetadata {
            field: field.name(),
            class,
        });
    }
    Ok(())
}
