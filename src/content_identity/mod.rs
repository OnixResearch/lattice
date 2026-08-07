//! Lattice-owned content identity roles over the reviewed shared framing core.
//!
//! Lattice owns every domain, canonical input, output spelling, persistence rule,
//! and claim. The shared package supplies only admitted unkeyed framing mechanics.

pub const SCHEDULE_OCCURRENCE_DOMAIN: &str = "lattice-schedule-occurrence-v1";
const SCHEDULE_ID_TAG: &[u8] = b"schedule-id";
const DUE_EPOCH_MS_TAG: &[u8] = b"due-epoch-ms";
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";

/// Canonical Lattice inputs for one durable schedule-occurrence identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleOccurrenceInput<'a> {
    pub schedule_id: &'a str,
    pub due_epoch_ms: i64,
}

/// Build the durable schedule-occurrence identity with unchanged Lattice bytes.
///
/// The role wrapper fixes the product domain, tags, integer encoding, and output
/// spelling. The shared core only performs the existing hashed-part framing.
pub fn schedule_occurrence(input: ScheduleOccurrenceInput<'_>) -> Result<String, content_identity_core::IdentityError> {
    let domain = content_identity_core::Domain::new(SCHEDULE_OCCURRENCE_DOMAIN)?;
    let mut hasher = content_identity_core::IdentityHasher::new(domain);
    hasher
        .update_hashed_part(SCHEDULE_ID_TAG, input.schedule_id.as_bytes())
        .update_hashed_part(DUE_EPOCH_MS_TAG, &input.due_epoch_ms.to_be_bytes());
    Ok(format!("{BLAKE3_IDENTITY_PREFIX}{}", hasher.finish().to_hex()))
}
