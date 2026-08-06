//! Checked-in generated adaptive theme definition and renderer artifacts.
//!
//! Source: `themes/rats-theme-tokens.ncl`.

#[path = "adaptive/data.rs"]
mod data;
#[path = "adaptive/mappings.rs"]
mod mappings;

pub use data::GENERATED_ADAPTIVE_THEME;

/// BLAKE3 identity of the typed Nickel source used for adaptive generation.
pub const ADAPTIVE_THEME_SOURCE_BLAKE3: &str = "463e69cf119ca95615d1080aca2096cf4d87dfcd8425bbaf088cb9b00d23dfe8";
pub(crate) const ADAPTIVE_THEME_SOURCE_BYTES: &[u8] = include_bytes!("../../../../themes/rats-theme-tokens.ncl");

/// Generated Ratatui adaptive mapping metadata.
pub const ADAPTIVE_RATATUI_MAPPING: &str = include_str!("adaptive/ratatui.txt");
/// Generated Meteorite adaptive CSS properties.
pub const ADAPTIVE_METEORITE_CSS: &str =
    include_str!("../../../../meteorite/crates/met-core/generated/adaptive-theme-tokens.css");
/// Generated bounded Iced-pilot mapping metadata.
pub const ADAPTIVE_ICED_PILOT_MAPPING: &str = include_str!("adaptive/iced-pilot.txt");
