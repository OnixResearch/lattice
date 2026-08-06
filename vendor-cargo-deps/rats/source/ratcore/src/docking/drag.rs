//! Pure drag and resize affordance model for docking shells.
//!
//! Renderers own pointer capture and drawing. This module owns only normalized
//! phases, deterministic session transitions, typed intents, and diagnostics.

include!("drag/geometry.rs");
include!("drag/model.rs");
include!("drag/reducer.rs");
include!("drag/tests.rs");
