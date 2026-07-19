mod catalog;
mod core;
mod fixtures;
mod routes;

pub use core::PointerMouseReplayTarget;
pub use fixtures::{
    PointerMouseReplayProof, PointerMouseReplayProofRow, pointer_mouse_negative_replay_script,
    pointer_mouse_positive_replay_script, pointer_mouse_replay_proof, pointer_mouse_replay_target,
};
pub use routes::{PointerMouseRendererArtifact, pointer_mouse_renderer_artifacts};

#[cfg(test)]
#[path = "gestures/tests.rs"]
mod tests;
