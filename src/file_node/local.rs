//! Descriptor-confined host filesystem operations.

use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootedOperationPath {
    root: std::path::PathBuf,
    relative: std::path::PathBuf,
}

// r[impl n8nclone.workflow_execution.file_nodes.local_capabilities]
pub(super) fn execute(plan: &super::FileOperationPlan) -> Result<super::FileOperationResult, super::FileError> {
    let rooted_path = select_rooted_operation_path(&plan.path, &plan.allowed_roots)?;
    let root = open_root_capability(&rooted_path.root, &plan.path)?;
    execute_relative_operation(&root, &rooted_path.relative, plan)
}

fn select_rooted_operation_path(path: &str, allowed_roots: &[String]) -> Result<RootedOperationPath, super::FileError> {
    let candidate = std::path::Path::new(path);
    let mut selected: Option<(usize, RootedOperationPath)> = None;

    for root_label in allowed_roots {
        let root = std::path::Path::new(root_label);
        let Ok(relative) = candidate.strip_prefix(root) else {
            continue;
        };
        if !has_safe_relative_components(relative) {
            continue;
        }

        let root_depth = root.components().count();
        let should_replace_selected = selected.as_ref().is_none_or(|(selected_depth, _)| root_depth > *selected_depth);
        if should_replace_selected {
            selected = Some((root_depth, RootedOperationPath {
                root: root.to_path_buf(),
                relative: relative.to_path_buf(),
            }));
        }
    }

    let rooted_path = selected
        .map(|(_, rooted_path)| rooted_path)
        .ok_or_else(|| super::FileError::UnsafePath(path.to_string()))?;
    debug_assert!(!rooted_path.relative.as_os_str().is_empty());
    debug_assert!(has_safe_relative_components(&rooted_path.relative));
    Ok(rooted_path)
}

fn has_safe_relative_components(path: &std::path::Path) -> bool {
    let mut has_normal_component = false;
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) => has_normal_component = true,
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return false;
            }
        }
    }
    has_normal_component
}

fn open_root_capability(root: &std::path::Path, operation_path: &str) -> Result<cap_std::fs::Dir, super::FileError> {
    cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|error| super::io_error(operation_path, error))
}

fn execute_relative_operation(
    root: &cap_std::fs::Dir,
    relative_path: &std::path::Path,
    plan: &super::FileOperationPlan,
) -> Result<super::FileOperationResult, super::FileError> {
    debug_assert!(!relative_path.as_os_str().is_empty());
    debug_assert!(has_safe_relative_components(relative_path));
    match plan.kind {
        super::FileOperationKind::Read => {
            let mut file = root.open(relative_path).map_err(|error| super::io_error(&plan.path, error))?;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|error| super::io_error(&plan.path, error))?;
            let bytes = content.len();
            Ok(super::FileOperationResult::Read { content, bytes })
        }
        super::FileOperationKind::Write => {
            let mut options = cap_std::fs::OpenOptions::new();
            options.write(true);
            if plan.overwrite {
                options.create(true).truncate(true);
            } else {
                options.create_new(true);
            }
            let mut file =
                root.open_with(relative_path, &options).map_err(|error| super::io_error(&plan.path, error))?;
            let content = plan.content.as_deref().unwrap_or_default();
            file.write_all(content.as_bytes()).map_err(|error| super::io_error(&plan.path, error))?;
            Ok(super::FileOperationResult::Written { bytes: content.len() })
        }
        super::FileOperationKind::Append => {
            let mut options = cap_std::fs::OpenOptions::new();
            options.create(true).append(true);
            let mut file =
                root.open_with(relative_path, &options).map_err(|error| super::io_error(&plan.path, error))?;
            let content = plan.content.as_deref().unwrap_or_default();
            file.write_all(content.as_bytes()).map_err(|error| super::io_error(&plan.path, error))?;
            Ok(super::FileOperationResult::Written { bytes: content.len() })
        }
    }
}

#[cfg(test)]
mod tests {
    const BROAD_ROOT: &str = "/workspace";
    const NARROW_ROOT: &str = "/workspace/state";
    const TARGET_PATH: &str = "/workspace/state/report.md";
    const EXPECTED_RELATIVE_PATH: &str = "report.md";
    const SLOT_DIRECTORY_NAME: &str = "slot";
    const MOVED_DIRECTORY_NAME: &str = "slot-original";
    const FILE_NAME: &str = "data.txt";
    const ORIGINAL_CONTENT: &str = "original";
    const OUTSIDE_CONTENT: &str = "outside";
    const APPEND_CONTENT: &str = "-attack";

    #[test]
    fn selects_most_specific_allowed_root() {
        let roots = vec![BROAD_ROOT.to_string(), NARROW_ROOT.to_string()];

        let selected = super::select_rooted_operation_path(TARGET_PATH, &roots).expect("select rooted operation path");

        assert_eq!(selected, super::RootedOperationPath {
            root: NARROW_ROOT.into(),
            relative: EXPECTED_RELATIVE_PATH.into(),
        });
    }

    // r[verify n8nclone.workflow_execution.file_nodes.local_capabilities]
    #[test]
    fn rejects_allowed_root_itself_as_file_target() {
        let roots = vec![NARROW_ROOT.to_string()];

        let result = super::select_rooted_operation_path(NARROW_ROOT, &roots);

        assert_eq!(result, Err(super::super::FileError::UnsafePath(NARROW_ROOT.to_string())));
    }

    // r[verify n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.path-swap]
    #[cfg(unix)]
    #[test]
    fn opened_root_rejects_replaced_intermediate_path() {
        let allowed = tempfile::tempdir().expect("allowed temporary directory");
        let outside = tempfile::tempdir().expect("outside temporary directory");
        let slot = allowed.path().join(SLOT_DIRECTORY_NAME);
        std::fs::create_dir(&slot).expect("create slot directory");
        let original_path = slot.join(FILE_NAME);
        std::fs::write(&original_path, ORIGINAL_CONTENT).expect("write original fixture");
        let outside_path = outside.path().join(FILE_NAME);
        std::fs::write(&outside_path, OUTSIDE_CONTENT).expect("write outside fixture");
        let target_path = original_path.to_string_lossy().into_owned();
        let plan = super::super::FileOperationPlan {
            kind: super::super::FileOperationKind::Append,
            path: target_path.clone(),
            content: Some(APPEND_CONTENT.to_string()),
            overwrite: false,
            allowed_roots: vec![allowed.path().to_string_lossy().into_owned()],
        };
        let rooted_path = super::select_rooted_operation_path(&target_path, &plan.allowed_roots)
            .expect("select rooted operation path before replacement");
        let root = super::open_root_capability(&rooted_path.root, &target_path).expect("open allowed root");
        let moved_slot = allowed.path().join(MOVED_DIRECTORY_NAME);
        std::fs::rename(&slot, &moved_slot).expect("move original slot");
        std::os::unix::fs::symlink(outside.path(), &slot).expect("replace slot with escaping symlink");

        let result = super::execute_relative_operation(&root, &rooted_path.relative, &plan);

        assert!(matches!(result, Err(super::super::FileError::PermissionDenied(_))), "result: {result:?}");
        assert_eq!(std::fs::read_to_string(outside_path).expect("read outside fixture"), OUTSIDE_CONTENT);
        assert_eq!(
            std::fs::read_to_string(moved_slot.join(FILE_NAME)).expect("read moved original fixture"),
            ORIGINAL_CONTENT
        );
    }
}
