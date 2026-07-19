//! Descriptor-confined host filesystem operations.

use std::io::Read;
use std::io::Write;

// r[impl n8nclone.workflow_execution.file_nodes.local_capabilities]
// r[impl n8nclone.workflow_execution.capability_reads.scenario.file-effects]
pub(super) fn execute(plan: &super::FileOperationPlan) -> Result<super::FileOperationResult, super::FileError> {
    let rooted_path = crate::capability::select_rooted_path(&plan.path, &plan.allowed_roots)
        .ok_or_else(|| super::FileError::UnsafePath(plan.path.clone()))?;
    let root = crate::capability::open_root(&rooted_path).map_err(|error| super::io_error(&plan.path, error))?;
    execute_relative_operation(&root, rooted_path.relative(), plan)
}

fn execute_relative_operation(
    root: &cap_std::fs::Dir,
    relative_path: &std::path::Path,
    plan: &super::FileOperationPlan,
) -> Result<super::FileOperationResult, super::FileError> {
    match plan.kind {
        super::FileOperationKind::Read => {
            let mut file = crate::capability::open_relative_read(root, relative_path)
                .map_err(|error| super::io_error(&plan.path, error))?;
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
    const SLOT_DIRECTORY_NAME: &str = "slot";
    const MOVED_DIRECTORY_NAME: &str = "slot-original";
    const FILE_NAME: &str = "data.txt";
    const ORIGINAL_CONTENT: &str = "original";
    const OUTSIDE_CONTENT: &str = "outside";
    const APPEND_CONTENT: &str = "-attack";

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
        let rooted_path = crate::capability::select_rooted_path(&target_path, &plan.allowed_roots)
            .expect("select rooted operation path before replacement");
        let root = crate::capability::open_root(&rooted_path).expect("open allowed root");
        let moved_slot = allowed.path().join(MOVED_DIRECTORY_NAME);
        std::fs::rename(&slot, &moved_slot).expect("move original slot");
        std::os::unix::fs::symlink(outside.path(), &slot).expect("replace slot with escaping symlink");

        let result = super::execute_relative_operation(&root, rooted_path.relative(), &plan);

        assert!(matches!(result, Err(super::super::FileError::PermissionDenied(_))), "result: {result:?}");
        assert_eq!(std::fs::read_to_string(outside_path).expect("read outside fixture"), OUTSIDE_CONTENT);
        assert_eq!(
            std::fs::read_to_string(moved_slot.join(FILE_NAME)).expect("read moved original fixture"),
            ORIGINAL_CONTENT
        );
    }
}
