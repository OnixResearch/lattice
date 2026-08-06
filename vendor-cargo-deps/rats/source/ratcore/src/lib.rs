//! Pure logic building blocks shared across Rat frontends.

#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
mod identity;

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
#[path = "validated/core.rs"]
mod boundary_core;
#[path = "calendar/date.rs"]
pub mod caldate;

/// Shared validated priority cookie used by agenda and tag consumers.
pub mod priority {
    pub use crate::priority_core::*;
}
#[path = "priority.rs"]
mod priority_core;
/// Focused finite and bounded values shared by workspace boundaries.
pub mod validated_values {
    pub use crate::boundary_core::*;
}
#[path = "docking/shell.rs"]
mod chrome_contract_core;
#[path = "docking/pointer.rs"]
mod panel_bridge_core;
#[path = "docking/drag.rs"]
mod pointer_state_core;
/// Renderer-neutral docking and panel layout core.
pub mod docking {
    pub use crate::arrange_core::*;
    pub use crate::chrome_contract_core::*;
    pub use crate::panel_bridge_core::*;
    pub use crate::pointer_state_core::*;
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
/// Renderer-neutral command palette model and reducer.
pub mod command_palette;
#[path = "interaction/core.rs"]
mod interaction_core;
/// Renderer-neutral bounded prompt sessions and supplied completion snapshots.
pub mod prompt_completion {
    pub use crate::interaction_core::*;
}
#[path = "evidence/core.rs"]
mod evidence_core;
#[path = "headless/core.rs"]
mod shared_behavior;
/// Executable renderer-neutral component contracts and tabs behavior.
pub mod headless_components {
    pub use crate::shared_behavior::*;
}
#[path = "simulation/core.rs"]
mod simcore;
/// Provider-neutral deterministic-simulation controls, probes, state, and oracles.
pub mod deterministic_simulation {
    pub use crate::simcore::*;
}
#[path = "notification/activity.rs"]
mod notification_activity_core;
/// Shared desktop notifications, activity history, and status projections.
pub mod notification_activity_center {
    pub use crate::notification_activity_core::*;
}
#[path = "generated/theme/adaptive.rs"]
pub mod generated_adaptive_theme_tokens;
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
#[path = "host_portability.rs"]
mod host_portability_core;
#[path = "renderer/parity.rs"]
mod parity_core;
#[path = "subwayrat_web/core.rs"]
mod sr_core;
#[path = "renderer/taxonomy.rs"]
mod targeting_core;
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
/// Typed renderer families, targets, evidence adapters, scopes, and lifecycle policy.
pub mod renderer_target_policy {
    pub use crate::targeting_core::*;
}
/// Host-family, exact-target, capability, effect, ownership, and evidence policy.
pub mod host_portability {
    pub use crate::host_portability_core::*;
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
