//! Nickel-based configuration layer.
//!
//! Three Nickel roles:
//!  1. **App config** — global settings (data dir, default timeouts, etc.)
//!  2. **Workflow definitions** — import/export workflows as .ncl files.
//!  3. **Node templates** — reusable node type definitions in Nickel.

use nickel_lang_core::error::NullReporter;
use nickel_lang_core::eval::cache::CacheImpl;
use nickel_lang_core::program::Program;
use nickel_lang_core::serialize::{self, ExportFormat};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Application-level configuration loaded from Nickel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory for redb database and other state.
    pub data_dir: PathBuf,
    /// Default shell command timeout in seconds.
    pub default_timeout_secs: u64,
    /// Shell to use for commands (default: "sh").
    pub shell: String,
    /// Max parallel node executions.
    pub max_parallelism: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = default_data_dir().unwrap_or_else(|| PathBuf::from(".n8nclone"));
        Self {
            data_dir,
            default_timeout_secs: 60,
            shell: "sh".into(),
            max_parallelism: 4,
        }
    }
}

/// Default data directory.
fn default_data_dir() -> Option<PathBuf> {
    std::env::var("N8NCLONE_DATA_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| home::home_dir().map(|h| h.join(".n8nclone")))
}

/// A node template defined in Nickel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTemplate {
    pub name: String,
    pub description: String,
    pub kind: String,
    /// Default command (for shell nodes).
    pub command: Option<String>,
    pub input_ports: Vec<PortTemplate>,
    pub output_ports: Vec<PortTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTemplate {
    pub label: String,
    pub type_tag: String,
}

/// Load app config from a Nickel file. Falls back to defaults on any error.
pub fn load_app_config(path: &Path) -> AppConfig {
    if !path.exists() {
        return AppConfig::default();
    }

    match load_nickel_as_json(path) {
        Ok(json) => match serde_json::from_value(json) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "invalid config, using defaults");
                AppConfig::default()
            }
        },
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to load config, using defaults");
            AppConfig::default()
        }
    }
}

/// Load node templates from a directory of .ncl files.
pub fn load_node_templates(dir: &Path) -> Vec<NodeTemplate> {
    let mut templates = Vec::new();

    if !dir.exists() {
        return templates;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return templates,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ncl") {
            match load_nickel_as_json(&path) {
                Ok(json) => match serde_json::from_value(json) {
                    Ok(tmpl) => templates.push(tmpl),
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "invalid template");
                    }
                },
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to eval template");
                }
            }
        }
    }

    templates
}

/// Evaluate a Nickel file and return the result as serde_json::Value.
fn load_nickel_as_json(path: &Path) -> Result<serde_json::Value, String> {
    let mut program =
        Program::<CacheImpl>::new_from_file(path, std::io::sink(), NullReporter {})
            .map_err(|e| format!("failed to load {}: {e:?}", path.display()))?;

    let rt = program
        .eval_full_for_export()
        .map_err(|e| format!("eval error in {}: {e:?}", path.display()))?;

    // Validate and serialize to JSON string.
    serialize::validate(ExportFormat::Json, &rt)
        .map_err(|e| format!("validation error in {}: {e:?}", path.display()))?;

    let mut json_buf = Vec::new();
    serde_json::to_writer(&mut json_buf, &rt)
        .map_err(|e| format!("json serialize error: {e}"))?;

    serde_json::from_slice(&json_buf)
        .map_err(|e| format!("json parse error: {e}"))
}

/// Generate a default config.ncl file.
pub fn default_config_ncl() -> &'static str {
    r#"{
  data_dir = "~/.n8nclone",
  default_timeout_secs = 60,
  shell = "sh",
  max_parallelism = 4,
}
"#
}

/// Generate an example node template.
pub fn example_template_ncl() -> &'static str {
    r#"{
  name = "grep",
  description = "Filter lines matching a pattern",
  kind = "shell",
  command = "grep -E '${pattern}'",
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
"#
}
