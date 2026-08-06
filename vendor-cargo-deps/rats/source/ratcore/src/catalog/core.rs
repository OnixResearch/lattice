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
pub use showcase::FixtureParitySummary;
pub use showcase::FixtureParitySummaryCell;
pub use showcase::FixtureParitySummaryRow;
pub use showcase::build_fixture_parity_summary;
pub use showcase::build_fixture_parity_summary_with_policy;
pub use showcase::built_in_fixture_parity_rows;
pub use showcase::built_in_fixture_parity_summary;
pub use showcase::built_in_fixtures;
mod validation;
pub use validation::fixtures_for_renderer;
pub use validation::validate_fixture_parity_baseline;
pub use validation::validate_fixture_parity_baseline_with_policy;
pub use validation::validate_showcase_fixtures;
pub use validation::validate_showcase_fixtures_with_policy;

mod model;
pub use model::*;
mod cases;
pub use cases::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
