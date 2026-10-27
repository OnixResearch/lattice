#![no_std]
#![forbid(unsafe_code)]
//! Pure, bounded ATIF-v1.7 contracts and transformations.

extern crate alloc;

mod diagnostic;
mod model;
mod policy;
mod projection;
mod receipt;
mod validate;
mod value;

#[cfg(feature = "json-codec")]
mod codec;

pub use diagnostic::*;
pub use model::*;
pub use policy::*;
pub use projection::*;
pub use receipt::*;
pub use validate::*;
pub use value::*;

#[cfg(feature = "json-codec")]
pub use codec::*;

/// Exact reviewed Harbor release.
pub const HARBOR_RELEASE: &str = "v0.18.0";
/// Exact reviewed Harbor revision.
pub const HARBOR_REVISION: &str = "527d50deb63a5d279e8c20593c18a2cbc7f61f9e";
/// Exact supported ATIF schema.
pub const ATIF_SCHEMA: &str = "ATIF-v1.7";
/// Exact source cohort identity.
pub const SOURCE_COHORT_IDENTITY: &str =
    "b3:78d4a45481206483c89269eeb515854552bb4185b4db41a966047d8f124cfe4f";
/// Codec contract identity.
pub const CODEC_IDENTITY: &str = "animus.harbor-atif-json-codec.v1";

pub(crate) fn hash_identity(domain: &str, parts: &[&[u8]]) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, domain.as_bytes());
    for part in parts {
        frame(&mut hasher, part);
    }
    alloc::format!("b3:{}", hasher.finalize().to_hex())
}

pub(crate) fn hash_strings(domain: &str, parts: &[&str]) -> alloc::string::String {
    let bytes = parts
        .iter()
        .map(|part| part.as_bytes())
        .collect::<alloc::vec::Vec<_>>();
    hash_identity(domain, &bytes)
}

fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

#[cfg(all(test, feature = "json-codec"))]
mod tests;
