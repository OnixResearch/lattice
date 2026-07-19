//! Renderer-neutral showcase fixture schema and validation.
//!
//! Fixtures describe component examples without embedding ratatui or web code.
//! Renderer-specific showcases can project the same fixture data into their own
//! presentation shells and report unsupported capabilities explicitly.

const SUPPORTED_FAMILIES: &[&str] = &[
    "action-registry",
    "chrome",
    "dialog",
    "fixture",
    "knowledge",
    "theme",
    "tree",
];

mod web;
pub use web::*;
mod metrics;
mod showcase;
pub use showcase::{
    FixtureParitySummary, FixtureParitySummaryRow, build_fixture_parity_summary,
    built_in_fixture_parity_rows, built_in_fixture_parity_summary, built_in_fixtures,
};
mod validation;
pub use validation::{
    fixtures_for_renderer, validate_fixture_parity_baseline, validate_showcase_fixtures,
};

mod model;
pub use model::*;
mod cases;
pub use cases::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
