//! VM execution profile (`lattice.vm-execution-profile.v1`, C1) and VM attempt
//! binding (`lattice.vm-attempt-binding.v1`, C2): pure admission, typed
//! refusals and BLAKE3 identities. The guest shell, the host model gateway
//! and Kiln verify the same records; Kiln implements them independently from
//! `fixtures/vm-profile/`. See `docs/vm-execution-profile.md`.

pub mod core;
pub mod model;
#[cfg(test)]
mod tests;

pub use self::core::REQUIRED_NON_CLAIMS;
pub use self::core::admit_binding;
pub use self::core::admit_mobility;
pub use self::core::admit_profile;
pub use self::core::binding_identity;
pub use self::core::drain_disposition;
pub use self::core::framed_identity;
pub use self::core::is_content_id;
pub use self::core::is_identity;
pub use self::core::profile_identity;
pub use self::model::*;
