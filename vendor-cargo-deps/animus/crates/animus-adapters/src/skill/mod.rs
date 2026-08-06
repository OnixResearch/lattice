//! Host-provided skill manifest descriptors and progressive disclosure.
//!
//! Hosts parse files such as `SKILL.md`, databases, bundles, or other stores and
//! pass typed fields into this module. The reusable adapter crate validates data
//! only; it does not discover directories, follow symlinks, read home folders, or
//! load full skill bodies by ambient path.

mod disclosure;
mod manifest;
mod resource;
#[cfg(test)]
mod tests;
mod validation;

pub use disclosure::BodyDraft;
pub use disclosure::Context;
pub use disclosure::LoadedBody;
pub use disclosure::Request;
pub use disclosure::build_context;
pub use manifest::Descriptor;
pub use manifest::Draft;
pub use manifest::Error;
pub use manifest::TrustStatus;
pub use resource::Catalog;
pub use resource::Diagnostic;
pub use resource::Issue;
pub use resource::Summary;
pub use resource::advertise;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_name");
const DESCRIPTION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_description");
const VERSION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_version");
const CATEGORY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_category");
const TAG_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_tag");
const RELATED_SKILL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("related_skill");
const PROVENANCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_provenance");
const TRUST_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_trust_status");
const CONTENT_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("skill_content_identity");
const BODY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_body");

const TRUSTED_LABEL: &str = "trusted";
const UNTRUSTED_LABEL: &str = "untrusted";
const VERSION_SEPARATOR: char = '.';
const NAME_SEPARATOR_DASH: char = '-';
const NAME_SEPARATOR_UNDERSCORE: char = '_';
const SEMVER_SEGMENT_COUNT: usize = 3;
const MAX_ITEMS: usize = 32;
const MAX_DESCRIPTION_CHARS: usize = 512;

const _: () = assert!(SEMVER_SEGMENT_COUNT > 0);
const _: () = assert!(MAX_ITEMS > 0);
const _: () = assert!(MAX_DESCRIPTION_CHARS >= crate::validation::MIN_ADAPTER_FIELD_CHARS);

/// Descriptor-side effects that skill manifest validation explicitly does not perform.
pub const PROHIBITED_MANIFEST_EFFECTS: &[&str] = &[
    "home-directory scan",
    "project-directory scan",
    "symlink resolution",
    "runtime-cache read",
    "skill body file read",
];

/// Character length of a lowercase BLAKE3 skill content identity.
pub const CONTENT_HASH_HEX_CHARS: usize = crate::artifact::BLAKE3_HASH_HEX_CHARS;
