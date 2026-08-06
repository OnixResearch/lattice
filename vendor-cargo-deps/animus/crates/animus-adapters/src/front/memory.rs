#![allow(unknown_lints)]
#![allow(non_trait_imports, path_segment_repetition)]

use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use super::guidance::PromptResourceScope;
use super::prompt_assembly::PromptAssembly;
use super::prompt_assembly::PromptAssemblyError;
use super::prompt_assembly::PromptAssemblyMode;
use super::prompt_assembly::PromptAssemblyPolicy;
use super::prompt_assembly::PromptAssemblyPolicyDraft;
use super::prompt_assembly::PromptBudgetClass;
use super::prompt_assembly::PromptRedactionClass;
use super::prompt_assembly::PromptSectionKind;
use super::prompt_assembly::PromptSourceAdmissionStatus;
use super::prompt_assembly::PromptSourceDraft;
use super::prompt_assembly::PromptSourceKind;
use super::prompt_assembly::PromptSourcePriority;
use super::prompt_assembly::PromptSourceRequirement;
use super::prompt_assembly::PromptSourceTrustStatus;
use super::prompt_assembly::assemble_prompt_sources;

const DESCRIPTOR_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_descriptor_id");
const MEMORY_SOURCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_source_label");
const MEMORY_CATEGORY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_category_label");
const DECISION_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_decision_label");
const RECEIPT_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_receipt_label");
const MEMORY_SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_summary_label");
const MEMORY_THREAD_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_source_thread_id");
const MEMORY_EVIDENCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_evidence_handle");
const MAX_MEMORY_DESCRIPTORS: usize = 32;
const MIN_MEMORY_CONFIDENCE_PERCENT: u8 = 1;
const MAX_MEMORY_CONFIDENCE_PERCENT: u8 = 100;

const _: () = assert!(MAX_MEMORY_DESCRIPTORS > 0);
const _: () = assert!(MAX_MEMORY_CONFIDENCE_PERCENT >= MIN_MEMORY_CONFIDENCE_PERCENT);

include!("memory/impl/base/impl001.rs");
include!("memory/impl/base/impl002.rs");
include!("memory/impl/base/impl003.rs");
include!("memory/impl/base/impl004.rs");
#[path = "memory/impl/optin/impl005.rs"]
mod optin_impl005;
pub use optin_impl005::*;
include!("memory/impl/optin/impl006.rs");
include!("memory/impl/optin/impl007.rs");
include!("memory/impl/optin/impl008.rs");
#[path = "memory/impl/generic/adapter/records.rs"]
mod generic_adapter_records;
pub use generic_adapter_records::*;
#[path = "memory/impl/generic/operation/controls.rs"]
mod generic_operation_controls;
pub use generic_operation_controls::*;
#[path = "memory/impl/generic/conformance/matrix.rs"]
mod generic_conformance_matrix;
pub use generic_conformance_matrix::*;

#[cfg(test)]
mod tests {
    include!("memory/tests/support.rs");
    include!("memory/tests/tests001.rs");
    include!("memory/tests/tests002.rs");
    include!("memory/tests/tests003.rs");
    include!("memory/tests/tests006.rs");
    include!("memory/tests/tests004.rs");
    include!("memory/tests/tests005.rs");
}
