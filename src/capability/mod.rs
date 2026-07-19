//! Shared capability-backed filesystem admission and bounded reads.

use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RootedPath {
    root: std::path::PathBuf,
    relative: std::path::PathBuf,
}

impl RootedPath {
    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub(crate) fn relative(&self) -> &std::path::Path {
        &self.relative
    }
}

// r[impl n8nclone.workflow_execution.capability_reads]
pub(crate) fn select_rooted_path(path: &str, allowed_roots: &[String]) -> Option<RootedPath> {
    let candidate = std::path::Path::new(path);
    let mut selected: Option<(usize, RootedPath)> = None;

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
            selected = Some((root_depth, RootedPath {
                root: root.to_path_buf(),
                relative: relative.to_path_buf(),
            }));
        }
    }

    let rooted_path = selected.map(|(_, rooted_path)| rooted_path)?;
    debug_assert!(!rooted_path.relative.as_os_str().is_empty());
    debug_assert!(has_safe_relative_components(&rooted_path.relative));
    Some(rooted_path)
}

pub(crate) fn rooted_relative_path(root: &std::path::Path, target: &std::path::Path) -> Option<RootedPath> {
    let relative = if target.is_absolute() {
        target.strip_prefix(root).ok()?
    } else {
        target
    };
    if !has_safe_relative_components(relative) {
        return None;
    }
    let rooted_path = RootedPath {
        root: root.to_path_buf(),
        relative: relative.to_path_buf(),
    };
    debug_assert!(!rooted_path.relative.as_os_str().is_empty());
    debug_assert!(has_safe_relative_components(&rooted_path.relative));
    Some(rooted_path)
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

pub(crate) fn open_root(rooted_path: &RootedPath) -> std::io::Result<cap_std::fs::Dir> {
    open_root_path(rooted_path.root())
}

pub(crate) fn open_root_path(root: &std::path::Path) -> std::io::Result<cap_std::fs::Dir> {
    cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
}

pub(crate) fn open_relative_read(
    root: &cap_std::fs::Dir,
    relative_path: &std::path::Path,
) -> std::io::Result<cap_std::fs::File> {
    debug_assert!(!relative_path.as_os_str().is_empty());
    debug_assert!(has_safe_relative_components(relative_path));
    root.open(relative_path)
}

pub(crate) fn open_rooted_read(rooted_path: &RootedPath) -> std::io::Result<cap_std::fs::File> {
    let root = open_root(rooted_path)?;
    open_relative_read(&root, rooted_path.relative())
}

pub(crate) fn read_bounded(file: cap_std::fs::File, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let read_limit_bytes = max_bytes.checked_add(1).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "bounded read limit cannot represent overflow sentinel")
    })?;
    let read_limit_u64_bytes = u64::try_from(read_limit_bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut bytes = Vec::new();
    file.take(read_limit_u64_bytes).read_to_end(&mut bytes)?;
    debug_assert!(bytes.len() <= read_limit_bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    const BROAD_ROOT: &str = "/workspace";
    const NARROW_ROOT: &str = "/workspace/state";
    const TARGET_PATH: &str = "/workspace/state/report.md";
    const EXPECTED_RELATIVE_PATH: &str = "report.md";
    const TRAVERSAL_PATH: &str = "/workspace/state/../outside.md";
    const SLOT_DIRECTORY_NAME: &str = "slot";
    const MOVED_DIRECTORY_NAME: &str = "slot-original";
    const FILE_NAME: &str = "data.txt";
    const ORIGINAL_CONTENT: &str = "original";
    const OUTSIDE_CONTENT: &str = "outside";
    const SMALL_BYTE_LIMIT: usize = 3;

    #[test]
    fn selects_most_specific_allowed_root() {
        let roots = vec![BROAD_ROOT.to_string(), NARROW_ROOT.to_string()];

        let selected = super::select_rooted_path(TARGET_PATH, &roots).expect("select rooted path");

        assert_eq!(selected, super::RootedPath {
            root: NARROW_ROOT.into(),
            relative: EXPECTED_RELATIVE_PATH.into(),
        });
    }

    #[test]
    fn rejects_missing_root_empty_target_and_traversal() {
        assert_eq!(super::select_rooted_path(TARGET_PATH, &[]), None);
        assert_eq!(super::select_rooted_path(NARROW_ROOT, &[NARROW_ROOT.to_string()]), None);
        assert_eq!(super::select_rooted_path(TRAVERSAL_PATH, &[NARROW_ROOT.to_string()]), None);
    }

    #[test]
    fn bounded_read_stops_after_overflow_sentinel() {
        let root = tempfile::tempdir().expect("temporary root");
        std::fs::write(root.path().join(FILE_NAME), ORIGINAL_CONTENT).expect("write fixture");
        let rooted_path =
            super::rooted_relative_path(root.path(), std::path::Path::new(FILE_NAME)).expect("admit fixture path");
        let file = super::open_rooted_read(&rooted_path).expect("open fixture");

        let bytes = super::read_bounded(file, SMALL_BYTE_LIMIT).expect("read bounded fixture");

        assert_eq!(bytes.len(), SMALL_BYTE_LIMIT + 1);
        assert_eq!(bytes.as_slice(), &ORIGINAL_CONTENT.as_bytes()[..SMALL_BYTE_LIMIT + 1]);
    }

    #[test]
    fn bounded_read_rejects_unrepresentable_sentinel() {
        let root = tempfile::tempdir().expect("temporary root");
        std::fs::write(root.path().join(FILE_NAME), ORIGINAL_CONTENT).expect("write fixture");
        let rooted_path =
            super::rooted_relative_path(root.path(), std::path::Path::new(FILE_NAME)).expect("admit fixture path");
        let file = super::open_rooted_read(&rooted_path).expect("open fixture");

        let error = super::read_bounded(file, usize::MAX).expect_err("unrepresentable sentinel rejected");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    // r[verify n8nclone.workflow_execution.capability_reads.scenario.path-swap]
    #[cfg(unix)]
    #[test]
    fn opened_root_rejects_replaced_intermediate_path() {
        let allowed = tempfile::tempdir().expect("allowed temporary directory");
        let outside = tempfile::tempdir().expect("outside temporary directory");
        let slot = allowed.path().join(SLOT_DIRECTORY_NAME);
        std::fs::create_dir(&slot).expect("create slot directory");
        std::fs::write(slot.join(FILE_NAME), ORIGINAL_CONTENT).expect("write original fixture");
        std::fs::write(outside.path().join(FILE_NAME), OUTSIDE_CONTENT).expect("write outside fixture");
        let rooted_path = super::rooted_relative_path(
            allowed.path(),
            std::path::Path::new(SLOT_DIRECTORY_NAME).join(FILE_NAME).as_path(),
        )
        .expect("admit target before replacement");
        let root = super::open_root(&rooted_path).expect("open allowed root");
        std::fs::rename(&slot, allowed.path().join(MOVED_DIRECTORY_NAME)).expect("move original slot");
        std::os::unix::fs::symlink(outside.path(), &slot).expect("replace slot with escaping symlink");

        let result = super::open_relative_read(&root, rooted_path.relative());

        assert!(matches!(result, Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied));
    }
}
