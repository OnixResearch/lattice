//! Renderer-neutral window and panel shell primitives.
//!
//! Docking owns logical panel placement. This module owns pure frame metadata,
//! validation, and geometry while renderers own drawing and event binding.

include!("shell/model.rs");
include!("shell/projection.rs");
include!("shell/behavior.rs");
include!("shell/tests.rs");
