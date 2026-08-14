// r[impl lattice.guarded_agent_templates.cairn]
pub fn discover_cairn(root: &std::path::Path, requested: Option<&str>) -> Result<super::core::CairnSnapshot, Error> {
    let canonical_root = root.canonicalize().map_err(|_| Error::RootUnavailable)?;
    debug_assert!(canonical_root.is_absolute());
    let root_dir = cap_std::fs::Dir::open_ambient_dir(&canonical_root, cap_std::ambient_authority())
        .map_err(|_| Error::RootUnavailable)?;
    let changes_dir = root_dir.open_dir(CAIRN_CHANGES_DIRECTORY).map_err(|_| Error::RootUnavailable)?;
    let mut active_changes = Vec::with_capacity(super::core::MAX_ACTIVE_CHANGES);
    let entries = changes_dir.entries().map_err(|_| Error::RootUnavailable)?;
    for entry in entries {
        let entry = entry.map_err(|_| Error::RootUnavailable)?;
        let file_type = entry.file_type().map_err(|_| Error::RootUnavailable)?;
        if file_type.is_symlink() {
            return Err(Error::SymlinkRejected);
        }
        if !file_type.is_dir() {
            continue;
        }
        let id = entry.file_name().into_string().map_err(|_| Error::MalformedObservation)?;
        active_changes.push(id);
        if active_changes.len() > super::core::MAX_ACTIVE_CHANGES {
            return Err(Error::LimitExceeded);
        }
    }
    debug_assert!(active_changes.len() <= super::core::MAX_ACTIVE_CHANGES);
    super::core::make_cairn_snapshot(active_changes, requested).map_err(|_| Error::MalformedObservation)
}
