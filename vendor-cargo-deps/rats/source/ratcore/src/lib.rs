//! Pure logic building blocks shared across Rat frontends.

#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]

#[path = "action/registry.rs"]
mod nexus_core;
/// Renderer-neutral action registry and projection helpers.
pub mod action_registry {
    pub use crate::nexus_core::*;
}
#[path = "pointer/routing.rs"]
mod cursor_core;
/// Renderer-neutral pointer hit testing and route selection.
pub mod pointer_input_routing {
    pub use crate::cursor_core::*;
}
#[path = "adapter/core.rs"]
mod adapter_core;
#[path = "arrange/core.rs"]
mod arrange_core;
#[path = "attention/core.rs"]
mod attention_core;
#[path = "calendar/date.rs"]
pub mod caldate;
#[path = "docking/drag.rs"]
mod docking_drag_core;
#[path = "docking/shell.rs"]
mod docking_shell_core;
#[path = "docking/pointer.rs"]
mod panel_bridge_core;
/// Renderer-neutral docking and panel layout core.
pub mod docking {
    pub use crate::arrange_core::*;
    pub use crate::docking_drag_core::*;
    pub use crate::docking_shell_core::*;
    pub use crate::panel_bridge_core::*;
}
/// Renderer-neutral focus graph and accessibility metadata.
pub mod focus {
    pub use crate::attention_core::*;
}
#[path = "search/core.rs"]
mod search_core;
/// Framework-neutral fuzzy scoring helpers.
pub mod fuzzy {
    pub use crate::search_core::*;
}
#[path = "animation.rs"]
mod animation_core;
#[path = "evidence/core.rs"]
mod evidence_core;
#[path = "notification/activity.rs"]
mod notification_activity_core;
/// Shared desktop notifications, activity history, and status projections.
pub mod notification_activity_center {
    pub use crate::notification_activity_core::*;
}
#[path = "generated/theme/tokens.rs"]
pub mod generated_theme_tokens;
pub mod inline;
/// Renderer-neutral animation intent, adapters, and parity evidence.
pub mod animation_parity {
    pub use crate::animation_core::*;
}
/// Renderer-neutral interaction replay scripts and deterministic runner.
pub mod interaction_replay {
    pub use crate::evidence_core::*;
}
/// Renderer-neutral knowledge workflow adapter core.
pub mod knowledge_kit {
    pub use crate::adapter_core::*;
}
#[path = "filter/core.rs"]
mod filter_core;
pub mod leaderkey;
/// Query schemas, predicates, facets, and evaluation helpers.
pub mod query {
    pub use crate::filter_core::*;
}
#[path = "catalog/core.rs"]
mod catalog_core;
#[path = "surface/core.rs"]
mod surface_core;
#[path = "workspace_shell/core.rs"]
mod terminalkit;
/// Showcase fixture validation and renderer coverage models.
pub mod showcase_fixtures {
    pub use crate::catalog_core::*;
}
#[path = "manual/core.rs"]
mod manual_core;
/// Snapshot manifest validation and generated component documentation helpers.
pub mod snapshot_docs {
    pub use crate::manual_core::*;
}
/// Renderer-neutral terminal IDE and workspace shell models.
pub mod terminal_ide_kit {
    pub use crate::surface_core::*;
    pub use crate::terminalkit::*;
}
#[path = "renderer/capabilities.rs"]
mod capability_core;
#[path = "renderer/parity.rs"]
mod parity_core;
#[path = "subwayrat_web/core.rs"]
mod sr_core;
#[path = "ratatui/coverage.rs"]
mod tui_cov_core;
#[path = "ratatui/rules/core.rs"]
mod tui_rules_core;
/// Subwayrat Ratzilla web-renderer evidence and input adapter contracts.
pub mod subwayrat_web_renderer {
    pub use crate::sr_core::*;
}
/// Cross-renderer feature parity matrix validation.
pub mod renderer_parity {
    pub use crate::capability_core::*;
    pub use crate::parity_core::*;
}
/// `Ratatui` API-family coverage inventory and feature-policy validation.
pub mod ratatui_integration {
    pub use crate::tui_cov_core::*;
    pub use crate::tui_rules_core::*;
}
#[path = "chrome/meteorite.rs"]
mod meteorite_chrome_core;
#[path = "chrome/overlay/semantics.rs"]
mod metov_core;
#[path = "palette/core.rs"]
mod palette_core;
/// Meteorite desktop chrome models, component projections, and evidence helpers.
pub mod chrome_primitives {
    pub use crate::meteorite_chrome_core::*;
    pub use crate::metov_core::*;
}
/// Theme token validation and CSS export helpers.
pub mod theme_tokens {
    pub use crate::palette_core::*;
}
#[path = "hierarchy/core.rs"]
mod hierarchy_core;
/// Tree flattening and visibility helpers.
pub mod tree {
    pub use crate::hierarchy_core::*;
}
#[path = "window/core.rs"]
mod window_core;
/// Renderer-neutral viewport, cursor, and selection logic.
pub mod viewport {
    pub use crate::window_core::*;
}
#[cfg(test)]
mod invariant {
    mod tests;
}
