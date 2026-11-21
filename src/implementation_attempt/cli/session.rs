//! One attempt's ownership and launch: its per-work fence record, its
//! execution intent, its agent-revision session, the harness template its
//! profile revision frames, and the bounded worker step.
pub(super) mod fencing;
pub(super) mod intent;
pub(super) mod launch;
pub(super) mod profile;
pub(super) mod revision;
