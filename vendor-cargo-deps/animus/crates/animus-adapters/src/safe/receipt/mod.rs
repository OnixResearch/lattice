mod dto;
mod fault;
mod kind;
mod leak;
mod normalize;
mod tally;
#[cfg(test)]
mod tests;

pub use dto::{Draft, Record, SCHEMA};
pub use fault::Error;
pub use kind::{IssueCode, LeakClass, Operation, Redaction, Status};
pub use tally::{BoundedCount, BoundedCountDraft};

/// Largest accepted safe metadata label length.
pub const MAX_LABEL_CHARS: usize = 256;

/// Largest accepted bounded count value in a safe receipt.
pub const MAX_COUNT: u32 = 1_000_000;

const MAX_COLLECTION_ITEMS: usize = 32;
const _: () = assert!(MAX_LABEL_CHARS > 0);
const _: () = assert!(MAX_COUNT > 0);
const _: () = assert!(MAX_COLLECTION_ITEMS > 0);
