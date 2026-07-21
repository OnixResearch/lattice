pub(crate) mod check;
pub(crate) mod contract;
pub(crate) mod facts;
pub(crate) mod failure;
#[cfg(test)]
mod tests;

pub(crate) const PURPOSE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_purpose");
pub(crate) const FIRST_ACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_first_action");
pub(crate) const REPEAT_CONDITION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_repeat_condition");
pub(crate) const STOP_CONDITION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_stop_condition");
pub(crate) const MISSING_BEHAVIOR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_missing_evidence_behavior");
pub(crate) const CITATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_citation_expectation");
pub(crate) const PROGRESS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_progress_guidance");
pub(crate) const PROVENANCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_provenance");
pub(crate) const EVIDENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_missing_evidence");
pub(crate) const SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_source_label");
pub(crate) const REFERENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("retrieval_safe_reference");

pub(crate) const MAX_CONDITIONS: usize = 8;
pub(crate) const MAX_GUIDANCE_CHARS: usize = 512;
pub(crate) const DEFAULT_MAX_RETRIEVALS: u32 = 1;
pub(crate) const UNSAFE_REFERENCE_MARKERS: &[&str] =
    &["/home/", "/tmp/", "../", "C:\\", "\\Users\\"];
