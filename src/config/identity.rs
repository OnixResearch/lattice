//! Pure product-identity compatibility decisions.

pub const DEFAULT_CONFIG_PATH: &str = "~/.lattice/config.ncl";
pub const LEGACY_CONFIG_PATH: &str = "~/.n8nclone/config.ncl";
pub const CANONICAL_DATA_DIR_NAME: &str = ".lattice";
pub const LEGACY_DATA_DIR_NAME: &str = ".n8nclone";
pub const CANONICAL_DATA_DIR_ENV: &str = "LATTICE_DATA_DIR";
pub const LEGACY_DATA_DIR_ENV: &str = "N8NCLONE_DATA_DIR";
pub const CANONICAL_STORE_FILE_NAME: &str = "lattice.redb";
pub const LEGACY_STORE_FILE_NAME: &str = "n8nclone.redb";

// r[impl lattice.project_identity.compatibility]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirFacts {
    pub canonical_override: Option<std::path::PathBuf>,
    pub legacy_override: Option<std::path::PathBuf>,
    pub home_dir: Option<std::path::PathBuf>,
    pub has_legacy_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePathFacts {
    pub canonical_path: std::path::PathBuf,
    pub legacy_path: std::path::PathBuf,
    pub has_canonical_store: bool,
    pub has_legacy_store: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPathFacts {
    pub requested_path: std::path::PathBuf,
    pub legacy_path: std::path::PathBuf,
    pub uses_canonical_default: bool,
    pub initializes_config: bool,
    pub has_canonical_config: bool,
    pub has_legacy_config: bool,
}

/// Select the canonical environment override, then legacy or default state.
pub fn select_data_dir(facts: DataDirFacts) -> Option<std::path::PathBuf> {
    facts
        .canonical_override
        .or(facts.legacy_override)
        .or_else(|| select_home_data_dir(facts.home_dir, facts.has_legacy_default))
}

fn select_home_data_dir(home_dir: Option<std::path::PathBuf>, has_legacy_default: bool) -> Option<std::path::PathBuf> {
    home_dir.map(|home| {
        if has_legacy_default {
            home.join(LEGACY_DATA_DIR_NAME)
        } else {
            home.join(CANONICAL_DATA_DIR_NAME)
        }
    })
}

/// Prefer canonical state, then an existing legacy store.
pub fn select_store_path(facts: StorePathFacts) -> std::path::PathBuf {
    if facts.has_canonical_store {
        return facts.canonical_path;
    }
    if facts.has_legacy_store {
        return facts.legacy_path;
    }
    facts.canonical_path
}

/// Select legacy config only for non-init implicit startup.
pub fn select_startup_path(facts: StartupPathFacts) -> std::path::PathBuf {
    if !facts.uses_canonical_default {
        return facts.requested_path;
    }
    if facts.initializes_config {
        return facts.requested_path;
    }
    if facts.has_canonical_config {
        return facts.requested_path;
    }
    if facts.has_legacy_config {
        return facts.legacy_path;
    }
    facts.requested_path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(value: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(value)
    }

    // r[verify lattice.project_identity.compatibility.scenario.environment-precedence]
    #[test]
    fn canonical_data_override_wins() {
        let selected = select_data_dir(DataDirFacts {
            canonical_override: Some(path("canonical")),
            legacy_override: Some(path("legacy")),
            home_dir: Some(path("home")),
            has_legacy_default: true,
        });
        assert_eq!(selected, Some(path("canonical")));
    }

    // r[verify lattice.project_identity.compatibility.scenario.legacy-state]
    #[test]
    fn existing_legacy_data_is_reused() {
        let selected = select_data_dir(DataDirFacts {
            canonical_override: None,
            legacy_override: None,
            home_dir: Some(path("home")),
            has_legacy_default: true,
        });
        assert_eq!(selected, Some(path("home").join(LEGACY_DATA_DIR_NAME)));
    }

    // r[verify lattice.project_identity.compatibility.scenario.canonical-default]
    #[test]
    fn missing_legacy_data_selects_canonical_default() {
        let selected = select_data_dir(DataDirFacts {
            canonical_override: None,
            legacy_override: None,
            home_dir: Some(path("home")),
            has_legacy_default: false,
        });
        assert_eq!(selected, Some(path("home").join(CANONICAL_DATA_DIR_NAME)));
    }

    #[test]
    fn canonical_store_wins_and_legacy_store_falls_back() {
        let canonical = select_store_path(StorePathFacts {
            canonical_path: path("lattice.redb"),
            legacy_path: path("n8nclone.redb"),
            has_canonical_store: true,
            has_legacy_store: true,
        });
        let legacy = select_store_path(StorePathFacts {
            canonical_path: path("lattice.redb"),
            legacy_path: path("n8nclone.redb"),
            has_canonical_store: false,
            has_legacy_store: true,
        });
        assert_eq!(canonical, path("lattice.redb"));
        assert_eq!(legacy, path("n8nclone.redb"));
    }

    #[test]
    fn missing_store_files_select_canonical_store() {
        let selected = select_store_path(StorePathFacts {
            canonical_path: path("lattice.redb"),
            legacy_path: path("n8nclone.redb"),
            has_canonical_store: false,
            has_legacy_store: false,
        });
        assert_eq!(selected, path("lattice.redb"));
    }

    #[test]
    fn implicit_startup_reuses_legacy_config() {
        let selected = select_startup_path(StartupPathFacts {
            requested_path: path("canonical/config.ncl"),
            legacy_path: path("legacy/config.ncl"),
            uses_canonical_default: true,
            initializes_config: false,
            has_canonical_config: false,
            has_legacy_config: true,
        });
        assert_eq!(selected, path("legacy/config.ncl"));
    }

    #[test]
    fn explicit_and_init_config_never_redirect() {
        let mut facts = StartupPathFacts {
            requested_path: path("custom/config.ncl"),
            legacy_path: path("legacy/config.ncl"),
            uses_canonical_default: false,
            initializes_config: false,
            has_canonical_config: false,
            has_legacy_config: true,
        };
        assert_eq!(select_startup_path(facts.clone()), path("custom/config.ncl"));
        facts.requested_path = path("canonical/config.ncl");
        facts.uses_canonical_default = true;
        facts.initializes_config = true;
        assert_eq!(select_startup_path(facts), path("canonical/config.ncl"));
    }
}
