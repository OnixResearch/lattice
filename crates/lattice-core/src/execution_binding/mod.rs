//! Pure worker execution binding over provider-owned compute allocations.
//!
//! Lattice consumes Aspen/Molten compute-capacity records only as the
//! Lattice-owned `lattice.compute-allocation-observation.v1` DTO. This core
//! validates that DTO, admits and revalidates one exact binding for an
//! implementation attempt, and projects profile, instance and usage facts. It
//! reads no files, environment, clocks, processes or network, and it never
//! reserves, activates, releases or probes compute.
//!
//! Producer record references are opaque canonical `blake3:` identifiers. They
//! are validated for shape and compared exactly; Lattice does not decode
//! Preserves or recompute them.

pub mod admission;
pub mod capability;
pub mod model;
pub mod projection;
pub mod request;
pub mod revalidation;
pub mod serving;
pub mod validation;

/// Serialized DTO ceiling in bytes.
pub const MAX_OBSERVATION_BYTES: u64 = 65_536;
/// Producer token bound in bytes (`molten_core::compute_capacity::MAX_TEXT`).
pub(crate) const MAX_TOKEN_BYTES: usize = 256;
/// Producer list bound (`molten_core::compute_capacity::MAX_ITEMS`).
pub(crate) const MAX_ITEMS: usize = 128;
/// Named model routes one worker profile may declare.
pub(crate) const MAX_ROUTES: usize = 8;
/// Producer granularity variants; an accepted set never exceeds them.
pub(crate) const MAX_GRANULARITIES: usize = 6;
const REFERENCE_PREFIX: &str = "blake3:";
const REFERENCE_HEX_BYTES: usize = 64;

/// Nonempty, at most 256 bytes and free of control characters.
pub(crate) fn is_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TOKEN_BYTES && !value.chars().any(char::is_control)
}

/// Canonical `blake3:` plus 64 lowercase hexadecimal digits.
pub(crate) fn is_reference(value: &str) -> bool {
    value.strip_prefix(REFERENCE_PREFIX).is_some_and(|digest| {
        digest.len() == REFERENCE_HEX_BYTES && digest.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}

/// Every element is distinct; duplicates are detected without loop growth.
pub(crate) fn is_distinct<'a>(values: impl Iterator<Item = &'a str> + Clone) -> bool {
    let unique = values.clone().collect::<std::collections::BTreeSet<_>>();
    unique.len() == values.count()
}
