mod catalog;
mod core;
mod fixtures;
mod routes;

pub use core::PointerMouseReplayTarget;

pub use fixtures::PointerMouseReplayProof;
pub use fixtures::PointerMouseReplayProofRow;
pub use fixtures::pointer_mouse_negative_replay_script;
pub use fixtures::pointer_mouse_positive_replay_script;
pub use fixtures::pointer_mouse_replay_proof;
pub use fixtures::pointer_mouse_replay_target;
pub use routes::PointerMouseRendererArtifact;
pub use routes::pointer_mouse_renderer_artifacts;

#[cfg(test)]
#[path = "gestures/tests.rs"]
mod tests;
