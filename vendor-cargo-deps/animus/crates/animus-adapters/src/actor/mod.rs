//! Typed agent-definition manifests, discovery, activation, and receipts.

mod admit;
mod catalog;
mod evidence;
mod fault;
mod select;

#[cfg(test)]
mod tests;

pub use admit::core::activate;
pub use admit::{
    AbilityKind, Activation, ActivationIssue, ActivationPolicy, ActivationRequest,
    ActivationStatus, Admission, AdmissionStatus, DeniedEscalation,
};
pub use catalog::{AuthorityBoundary, Draft, Manifest, SourceScope, TrustStatus};
pub use evidence::{Receipt, ReceiptDraft, ReceiptLeak, validate_text};
pub use fault::Error;
pub use select::{Diagnostic, DiscoveryIssue, DiscoveryReport, discover};

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

pub(super) const NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_name");
pub(super) const DESCRIPTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_description");
pub(super) const MODEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_model_preference");
pub(super) const PROMPT_FRAGMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_prompt_fragment");
pub(super) const CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_capability_ceiling");
pub(super) const TOOLSET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_toolset_limit");
pub(super) const MEMORY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_memory_scope");
pub(super) const CONTEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_context_reference_scope");
pub(super) const WORKFLOW_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_workflow_ability");
pub(super) const SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_source_scope");
pub(super) const TRUST_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_trust_status");
pub(super) const MANIFEST_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_manifest_identity");
pub(super) const REDACTION_MARKER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_receipt_redaction");

pub(super) const USER_SOURCE_LABEL: &str = "user";
pub(super) const PROJECT_SOURCE_LABEL: &str = "project";
pub(super) const HOST_SOURCE_LABEL: &str = "host";
pub(super) const TRUSTED_LABEL: &str = "trusted";
pub(super) const UNTRUSTED_LABEL: &str = "untrusted";
pub(super) const HOST_SOURCE_RANK: u8 = 0;
pub(super) const USER_SOURCE_RANK: u8 = 1;
pub(super) const PROJECT_SOURCE_RANK: u8 = 2;
pub(super) const MAX_ITEMS: usize = 32;
pub(super) const MAX_PROMPT_CHARS: usize = 4_096;

const _: () = assert!(HOST_SOURCE_RANK < USER_SOURCE_RANK);
const _: () = assert!(USER_SOURCE_RANK < PROJECT_SOURCE_RANK);
const _: () = assert!(MAX_ITEMS > 0);
const _: () = assert!(MAX_PROMPT_CHARS > 0);

pub(super) fn required_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, Error> {
    match value {
        Some(raw) => required_text_value(field_name, raw),
        None => Err(Error::MissingField {
            field: field_name.as_str(),
        }),
    }
}

pub(super) fn required_text_value(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<Text, Error> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(Error::from)
}

pub(super) fn required_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, Error> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field_name.as_str(),
        }
        .into());
    }
    collect_texts(field_name, values)
}

pub(super) fn required_label_set(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, Error> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field_name.as_str(),
        }
        .into());
    }
    normalized_label_set(field_name, values)
}

pub(super) fn optional_label_set(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, Error> {
    normalized_label_set(field_name, values)
}

fn normalized_label_set(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, Error> {
    let mut labels = collect_texts(field_name, values)?;
    labels.sort();
    labels.dedup();
    Ok(labels)
}

fn collect_texts(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, Error> {
    validate_item_count(field_name, values.len())?;
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        normalized.push(required_text_value(field_name, value)?);
    }
    Ok(normalized)
}

fn validate_item_count(
    field_name: crate::validation::FieldName,
    item_count: usize,
) -> Result<(), Error> {
    if item_count > MAX_ITEMS {
        return Err(Error::TooManyItems {
            field: field_name.as_str(),
            max_items: MAX_ITEMS,
            actual_items: item_count,
        });
    }
    Ok(())
}

pub(super) fn contains_label(labels: &[&str], target: &str) -> bool {
    labels.contains(&target)
}
