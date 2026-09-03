//! Pure structured-input prompt projection and bounded pilot evaluation.

mod core;
mod evaluation;

pub use core::ProjectionError;
pub use core::ProjectionErrorClass;
pub use core::RenderedStructuredInput;
pub use core::StructuredInputProjection;
pub use core::TOON_V3_IMPLEMENTATION;
pub use core::decode_toon_v3_strict;
pub use core::render_structured_inputs;

pub use evaluation::PilotBindings;
pub use evaluation::PilotObservation;
pub use evaluation::PilotVerdict;
pub use evaluation::ProjectionPilotInput;
pub use evaluation::ProjectionPilotReport;
pub use evaluation::TaskOutcome;
pub use evaluation::evaluate_pilot;

#[cfg(test)]
mod tests;
