//! Transactional property editing and typed draft conversion.

use super::*;

mod admission;
pub(super) mod fields;
mod interaction;
pub(in crate::tui) mod parsing;
mod typed;
