use std::io::Write;

const NODE_TEMPLATE_DIRECTORY: &str = "templates";
const WORKFLOW_TEMPLATE_DIRECTORY: &str = "workflow-templates";
const BUILTIN_BUNDLE_DIRECTORY: &str = "shell-preview";
const BUILTIN_MANIFEST_PATH: &str = "manifest.ncl";
const BUILTIN_WORKFLOW_PATH: &str = "workflow.ncl";
const CONFIG_FILE_LABEL: &str = "config.ncl";
const MAX_INITIALIZATION_FILES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltInFile {
    pub relative_path: &'static str,
    pub bytes: &'static [u8],
}

pub const BUILTIN_WORKFLOW_TEMPLATE_FILES: &[BuiltInFile] = &[
    BuiltInFile {
        relative_path: concat!("shell-preview/", "manifest.ncl"),
        bytes: include_bytes!(concat!("../../builtins/workflow-templates/shell-preview/", "manifest.ncl")),
    },
    BuiltInFile {
        relative_path: concat!("shell-preview/", "workflow.ncl"),
        bytes: include_bytes!(concat!("../../builtins/workflow-templates/shell-preview/", "workflow.ncl")),
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializationReport {
    pub created: Vec<std::path::PathBuf>,
}

#[derive(Debug)]
pub enum InitializationError {
    ExistingTarget {
        relative_path: std::path::PathBuf,
    },
    Io {
        relative_path: std::path::PathBuf,
        created: Vec<std::path::PathBuf>,
    },
    TooManyFiles,
}

impl std::fmt::Display for InitializationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExistingTarget { relative_path } => {
                write!(formatter, "refusing to overwrite existing initialization target: {}", relative_path.display())
            }
            Self::Io { relative_path, created } => write!(
                formatter,
                "initialization failed at {} after creating {} files",
                relative_path.display(),
                created.len()
            ),
            Self::TooManyFiles => formatter.write_str("initialization file count exceeds its limit"),
        }
    }
}

impl std::error::Error for InitializationError {}

struct PlannedFile<'a> {
    relative_path: std::path::PathBuf,
    bytes: &'a [u8],
}

// r[impl lattice.workflow_templates.init]
pub fn initialize(config_path: &std::path::Path) -> Result<InitializationReport, InitializationError> {
    let root = config_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let config_name = config_path
        .file_name()
        .map_or_else(|| std::path::PathBuf::from(CONFIG_FILE_LABEL), std::path::PathBuf::from);
    let planned = initialization_plan(config_name)?;
    preflight(root, &planned)?;
    apply_plan(root, planned)
}

fn apply_plan(
    root: &std::path::Path,
    planned: Vec<PlannedFile<'_>>,
) -> Result<InitializationReport, InitializationError> {
    let mut created = Vec::with_capacity(planned.len());
    for file in planned {
        let target = root.join(&file.relative_path);
        if let Some(parent) = target.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            return Err(InitializationError::Io {
                relative_path: file.relative_path,
                created,
            });
        }
        if write_new(&target, file.bytes).is_err() {
            return Err(InitializationError::Io {
                relative_path: file.relative_path,
                created,
            });
        }
        created.push(file.relative_path);
    }
    debug_assert!(!created.is_empty());
    debug_assert!(created.len() <= MAX_INITIALIZATION_FILES);
    Ok(InitializationReport { created })
}

fn initialization_plan(config_name: std::path::PathBuf) -> Result<Vec<PlannedFile<'static>>, InitializationError> {
    let file_count = crate::config::DEFAULT_NODE_TEMPLATE_FILES
        .len()
        .saturating_add(BUILTIN_WORKFLOW_TEMPLATE_FILES.len())
        .saturating_add(1);
    if file_count > MAX_INITIALIZATION_FILES {
        return Err(InitializationError::TooManyFiles);
    }
    let mut files = Vec::with_capacity(file_count);
    files.push(PlannedFile {
        relative_path: config_name,
        bytes: crate::config::default_config_ncl().as_bytes(),
    });
    for template in crate::config::DEFAULT_NODE_TEMPLATE_FILES {
        files.push(PlannedFile {
            relative_path: std::path::Path::new(NODE_TEMPLATE_DIRECTORY).join(template.file_name),
            bytes: template.contents.as_bytes(),
        });
    }
    for builtin in BUILTIN_WORKFLOW_TEMPLATE_FILES {
        let relative = std::path::Path::new(WORKFLOW_TEMPLATE_DIRECTORY).join(builtin.relative_path);
        debug_assert!(relative.ends_with(BUILTIN_MANIFEST_PATH) || relative.ends_with(BUILTIN_WORKFLOW_PATH));
        debug_assert!(relative.components().any(|component| component.as_os_str() == BUILTIN_BUNDLE_DIRECTORY));
        files.push(PlannedFile {
            relative_path: relative,
            bytes: builtin.bytes,
        });
    }
    debug_assert_eq!(files.len(), file_count);
    Ok(files)
}

fn preflight(root: &std::path::Path, files: &[PlannedFile<'_>]) -> Result<(), InitializationError> {
    debug_assert!(files.len() <= MAX_INITIALIZATION_FILES);
    debug_assert!(files.iter().all(|file| !file.relative_path.is_absolute()));
    for file in files {
        let target = root.join(&file.relative_path);
        match std::fs::symlink_metadata(&target) {
            Ok(_) => {
                return Err(InitializationError::ExistingTarget {
                    relative_path: file.relative_path.clone(),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(InitializationError::Io {
                    relative_path: file.relative_path.clone(),
                    created: Vec::new(),
                });
            }
        }
    }
    Ok(())
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::InitializationError;
    use super::PlannedFile;
    use super::apply_plan;
    use super::preflight;
    const ORIGINAL_ROLE_BYTES: &[u8] = b"operator-role";
    const ORIGINAL_ASSET_BYTES: &[u8] = b"operator-asset";
    const NEW_BYTES: &[u8] = b"new";
    const EXPECTED_PARTIAL_CREATED_FILES: usize = 1;

    // r[verify lattice.workflow_templates.init.scenario.existing]
    #[test]
    fn preflight_preserves_existing_role_and_asset_bytes() {
        let root = tempfile::tempdir().expect("initialization fixture root creates");
        let role = std::path::PathBuf::from("workflow-templates/example/roles/operator.ncl");
        let asset = std::path::PathBuf::from("workflow-templates/example/assets/guide.md");
        for (path, bytes) in [(&role, ORIGINAL_ROLE_BYTES), (&asset, ORIGINAL_ASSET_BYTES)] {
            let target = root.path().join(path);
            std::fs::create_dir_all(target.parent().expect("fixture target has parent"))
                .expect("fixture parent creates");
            std::fs::write(target, bytes).expect("fixture bytes write");
        }
        let planned = vec![
            PlannedFile {
                relative_path: role.clone(),
                bytes: NEW_BYTES,
            },
            PlannedFile {
                relative_path: asset.clone(),
                bytes: NEW_BYTES,
            },
        ];

        assert!(matches!(preflight(root.path(), &planned), Err(InitializationError::ExistingTarget { .. })));
        assert_eq!(std::fs::read(root.path().join(role)).expect("role reads"), ORIGINAL_ROLE_BYTES);
        assert_eq!(std::fs::read(root.path().join(asset)).expect("asset reads"), ORIGINAL_ASSET_BYTES);
    }

    // r[verify lattice.workflow_templates.init.scenario.existing]
    #[test]
    fn create_new_race_reports_partial_result_without_replacing_bytes() {
        let root = tempfile::tempdir().expect("initialization race root creates");
        let first = std::path::PathBuf::from("config.ncl");
        let raced = std::path::PathBuf::from("workflow-templates/example/manifest.ncl");
        let planned = vec![
            PlannedFile {
                relative_path: first.clone(),
                bytes: NEW_BYTES,
            },
            PlannedFile {
                relative_path: raced.clone(),
                bytes: NEW_BYTES,
            },
        ];
        preflight(root.path(), &planned).expect("race plan initially passes preflight");
        let raced_target = root.path().join(&raced);
        std::fs::create_dir_all(raced_target.parent().expect("raced target has parent"))
            .expect("raced target parent creates");
        std::fs::write(&raced_target, ORIGINAL_ROLE_BYTES).expect("racing writer creates target");

        let error = apply_plan(root.path(), planned).expect_err("raced target fails create-only application");
        match error {
            InitializationError::Io { relative_path, created } => {
                assert_eq!(relative_path, raced);
                assert_eq!(created, vec![first]);
                assert_eq!(created.len(), EXPECTED_PARTIAL_CREATED_FILES);
            }
            other => panic!("expected partial initialization I/O error, got {other:?}"),
        }
        assert_eq!(std::fs::read(raced_target).expect("raced bytes read"), ORIGINAL_ROLE_BYTES);
    }
}
