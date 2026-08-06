//! Host-provided prompt resource descriptors and progressive disclosure.
//!
//! Hosts discover skills, prompt templates, context guidance, bundles, or
//! application-provided resources and pass typed descriptors into this module.
//! This adapter boundary validates in-memory data only; it never scans
//! directories, installs packages, resolves symlinks, reads full bodies by
//! ambient path, or executes extension code.

#![allow(non_trait_imports)]

mod catalog;
mod fault;
mod record;
pub(crate) mod retrieval;
#[cfg(test)]
mod tests;
mod validation;
mod view;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_kind");
const NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_name");
const DESCRIPTION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_description");
const SCOPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_scope");
const PROVENANCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_provenance");
const TRUST_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_trust_status");
const CONTENT_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("resource_content_identity");
const BODY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("resource_body");

const SKILL_KIND_LABEL: &str = "skill";
const PROMPT_TEMPLATE_KIND_LABEL: &str = "prompt-template";
const CONTEXT_GUIDANCE_KIND_LABEL: &str = "context-guidance";
const HOST_RESOURCE_KIND_LABEL: &str = "host-resource";
const GLOBAL_SCOPE_LABEL: &str = "global";
const PROJECT_SCOPE_LABEL: &str = "project";
const SESSION_SCOPE_LABEL: &str = "session";
const HOST_SCOPE_LABEL: &str = "host";
const TRUSTED_LABEL: &str = "trusted";
const UNTRUSTED_LABEL: &str = "untrusted";
const NAME_SEPARATOR_DASH: char = '-';
const NAME_SEPARATOR_UNDERSCORE: char = '_';
const MAX_DESCRIPTORS: usize = 32;
const MAX_DESCRIPTION_CHARS: usize = 512;
const DIAGNOSTIC_SLOTS_PER_DESCRIPTOR: usize = 2;

const _: () = assert!(MAX_DESCRIPTORS > 0);
const _: () = assert!(MAX_DESCRIPTION_CHARS >= crate::validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(DIAGNOSTIC_SLOTS_PER_DESCRIPTOR > 0);

/// Descriptor-side effects that resource validation and disclosure do not perform.
pub const PROHIBITED_DISCOVERY_EFFECTS: &[&str] = &[
    "filesystem traversal",
    "project-directory scan",
    "package installation",
    "extension code loading",
    "ambient resource body read",
];

const UNSAFE_PROVENANCE_MARKERS: &[&str] = &["../", "/home/", "/tmp/", "C:\\\\", "\\\\Users\\\\"];

pub use catalog::advertise;
pub use catalog::build_context;
pub use fault::Error;
pub use record::Descriptor;
pub use record::Draft;
pub use record::Kind;
pub use record::Scope;
pub use record::TrustStatus;
pub use view::Catalog;
pub use view::ContentDraft;
pub use view::Context;
pub use view::Diagnostic;
pub use view::Issue;
pub use view::LoadedContent;
pub use view::Request;
pub use view::Summary;
