const ENTRY_METEORITE_WEB: &str = "meteorite.met-showcase.web";
const ENTRY_RATZILLA_WEB: &str = "subwayrat.ratzilla.web";
const TITLE_METEORITE_WEB: &str = "Meteorite web showcase";
const TITLE_RATZILLA_WEB: &str = "Subwayrat Ratzilla web showcase";
const RENDERER_METEORITE_DOM: &str = crate::targeting_core::TARGET_METEORITE_DOM;
const RENDERER_RATZILLA_WEB: &str = "subwayrat-ratzilla-web";
const PACKAGE_MET_SHOWCASE: &str = "met-showcase";
const PACKAGE_RAT_RATZILLA: &str = "rat-ratzilla";
const PATH_MET_SHOWCASE: &str = "meteorite/crates/met-showcase";
const PATH_RAT_RATZILLA: &str = "subwayrat/crates/rat-ratzilla";
const METEORITE_LAUNCH_COMMAND: &str = "cd meteorite/crates/met-showcase && nix develop ../.. -c dx serve";
const METEORITE_BUNDLE_COMMAND: &str =
    "cd meteorite/crates/met-showcase && nix develop ../.. -c dx build --platform web";
const RATZILLA_LAUNCH_COMMAND: &str =
    "cd subwayrat/crates/rat-ratzilla && nix develop ../../../meteorite -c trunk serve --config Trunk.toml";
const RATZILLA_BUNDLE_COMMAND: &str =
    "cd subwayrat/crates/rat-ratzilla && nix develop ../../../meteorite -c trunk build --config Trunk.toml";
const PROFILE_MET_SHOWCASE: &str = "meteorite-showcase-web";
const PROFILE_SUBWAYRAT_WEB: &str = "subwayrat-web";
const DOC_METEORITE_README: &str = "meteorite/README.md";
const DOC_METEORITE_RUNTIME: &str = "docs/meteorite-showcase-web-runtime.md";
const DOC_WORKSPACE_VALIDATION: &str = "docs/workspace-validation.md";
const DOC_SUBWAYRAT_WEB: &str = "docs/subwayrat-web-renderer.md";
const DOC_RATZILLA_INPUT: &str = "docs/ratzilla-browser-input-lifecycle.md";
const FIXTURE_DIALOG_BASIC: &str = "dialog.basic";
const FIXTURE_CHROME_SHELL: &str = "chrome.shell";
const FIXTURE_KNOWLEDGE_WORKFLOW: &str = "knowledge.workflow";
const FIXTURE_THEME_TOKENS: &str = "theme.tokens";
const RATZILLA_GAP_KNOWLEDGE: &str =
    "knowledge.workflow is not admitted because no Ratzilla knowledge-widget interaction adapter exists";
const RATZILLA_GAP_NATIVE: &str = "raw-mode and interactive nodegraph paths remain native-only";
const RATZILLA_GAP_INPUT: &str = "wheel, drag-session visual replay, and mouse Meta remain partial";
const RATZILLA_GAP_SCOPE: &str = "per-fixture evidence does not imply whole-Subwayrat browser parity";
const STATUS_COMPLETE: &str = "complete";
const STATUS_PARTIAL: &str = "partial";
const STATUS_UNSUPPORTED: &str = "unsupported";
const FIELD_ID: &str = "id";
const FIELD_TITLE: &str = "title";
const FIELD_RENDERER_LABEL: &str = "renderer_label";
const FIELD_PACKAGE_NAME: &str = "package_name";
const FIELD_CRATE_PATH: &str = "crate_path";
const FIELD_LAUNCH_COMMAND: &str = "launch_command";
const FIELD_BUNDLE_COMMAND: &str = "bundle_command";
const FIELD_VALIDATION_PROFILE: &str = "validation_profile";
const FIELD_SUPPORTED_FIXTURE_IDS: &str = "supported_fixture_ids";
const FIELD_EVIDENCE_LINKS: &str = "evidence_links";
const FIELD_UNSUPPORTED_REASONS: &str = "unsupported_reasons";
const EMPTY_VALUE: &str = "";
const LIST_SEPARATOR: &str = ", ";
const REASON_SEPARATOR: &str = "; ";

fn legacy_browser_target(label: &str) -> crate::targeting_core::RendererTargetId {
    match label {
        RENDERER_RATZILLA_WEB => {
            crate::targeting_core::RendererTargetId::new(crate::targeting_core::TARGET_SUBWAYRAT_RATZILLA)
        }
        _ => crate::targeting_core::RendererTargetId::new(label),
    }
}

mod data;
pub use data::built_in_browser_showcase_entries;
pub use data::built_in_browser_showcase_index;
mod routes;
pub use routes::browser_fixture_navigation_rows;
mod landing;
pub use landing::render_browser_showcase_portal_markdown;
mod model;
pub use model::*;
mod rules;
pub use rules::validate_browser_showcase_index;
pub use rules::validate_browser_showcase_index_with_policy;

#[cfg(test)]
#[path = "web/tests.rs"]
mod tests;
