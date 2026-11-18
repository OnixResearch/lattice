//! Imperative shell for the immutable Nix-store executables of the first local
//! construction profile: operator selection, filesystem metadata and artifact bytes.

const NAMESPACE: &str = "/nix/store";

/// The first supported profile uses root-owned, read-only Nix-store entries.
/// A writable multi-user store namespace is allowed only when its sticky bit
/// protects those entries from replacement; mutable/rootless paths are refused.
#[cfg(unix)]
pub(crate) fn immutable_executable(path: &std::path::Path) -> Result<String, &'static str> {
    let relative = path.strip_prefix(NAMESPACE).map_err(|_| "construction executable must be in Nix store")?;
    let mut components = relative.components();
    let entry = components.next().ok_or("construction store entry missing")?;
    let name = entry.as_os_str().to_str().ok_or("construction store entry is invalid")?;
    let is_nested_entry = matches!(entry, std::path::Component::Normal(_)) && components.next().is_some();
    if !is_nested_entry || !is_hashed_name(name) || canonical(path)? != path {
        return Err("construction executable must be a canonical Nix-store path");
    }
    debug_assert!(relative.components().count() >= 2, "an entry and a member below it were selected");
    require_protected_namespace()?;
    let mut prefix = std::path::PathBuf::from(NAMESPACE);
    for component in relative.components() {
        prefix.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&prefix).map_err(|_| "construction artifact unavailable")?;
        if !is_immutable_entry(&metadata) {
            return Err("construction executable path is mutable");
        }
    }
    debug_assert_eq!(prefix, path, "every component of the canonical path was inspected");
    crate::operation::identity::executor_file(path).map_err(|_| "construction executable is invalid")
}

#[cfg(not(unix))]
pub(crate) fn immutable_executable(_path: &std::path::Path) -> Result<String, &'static str> {
    Err("construction profile requires immutable Nix-store executables")
}

#[cfg(unix)]
fn canonical(path: &std::path::Path) -> Result<std::path::PathBuf, &'static str> {
    std::fs::canonicalize(path).map_err(|_| "construction artifact unavailable")
}

/// A store entry name is a 32-character base-32 hash, a dash and a non-empty name.
fn is_hashed_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 34
        && bytes.get(32) == Some(&b'-')
        && bytes.get(..32).is_some_and(|hash| hash.iter().all(u8::is_ascii_alphanumeric))
}

#[cfg(unix)]
fn require_protected_namespace() -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;
    let nix = std::fs::symlink_metadata("/nix").map_err(|_| "construction store unavailable")?;
    if !is_root_owned_directory(&nix) || nix.permissions().mode() & 0o022 != 0 {
        return Err("construction store root is mutable");
    }
    let namespace = std::fs::symlink_metadata(NAMESPACE).map_err(|_| "construction store unavailable")?;
    if !is_root_owned_directory(&namespace) || !protects_entries(namespace.permissions().mode()) {
        return Err("construction store lacks protected root-owned entries");
    }
    Ok(())
}

#[cfg(unix)]
fn is_root_owned_directory(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    metadata.is_dir() && !metadata.file_type().is_symlink() && metadata.uid() == 0
}

/// Entries are protected when the namespace is not writable, or is sticky.
const fn protects_entries(mode: u32) -> bool {
    mode & 0o222 == 0 || mode & 0o1000 != 0
}

#[cfg(unix)]
fn is_immutable_entry(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::PermissionsExt as _;
    !metadata.file_type().is_symlink() && metadata.uid() == 0 && metadata.permissions().mode() & 0o222 == 0
}

/// The selected shell must be the reviewed packaged backend; an operator selection
/// can only restate it.
#[cfg(target_os = "linux")]
pub(super) fn reviewed_shell() -> Result<std::path::PathBuf, &'static str> {
    const DIFFERS: &str = "construction shell backend differs from reviewed implementation";
    let reviewed = option_env!("LATTICE_CONSTRUCTION_SHELL_DEFAULT");
    let selected = crate::config::read_path_env("LATTICE_CONSTRUCTION_SHELL")
        .map_err(|()| DIFFERS)?
        .or_else(|| reviewed.map(std::path::PathBuf::from))
        .ok_or("construction shell backend is not selected")?;
    let reviewed = reviewed.ok_or("reviewed construction shell backend is not configured")?;
    if selected != std::path::Path::new(reviewed) {
        return Err(DIFFERS);
    }
    Ok(selected)
}

/// Identity of the running executable, bound to the immutable path it runs from.
#[cfg(target_os = "linux")]
pub(super) fn running_artifact() -> Result<String, &'static str> {
    let running = std::fs::File::open("/proc/self/exe").map_err(|_| "running executable unavailable")?;
    let path = std::fs::read_link("/proc/self/exe").map_err(|_| "running executable unavailable")?;
    let path_artifact = immutable_executable(&path)?;
    let artifact =
        crate::operation::identity::executor_open_file(&running).map_err(|_| "running executable is invalid")?;
    if artifact != path_artifact {
        return Err("running executable changed at selected path");
    }
    Ok(artifact)
}

#[cfg(test)]
mod tests {
    #[test]
    fn hashed_entry_names_require_hash_dash_and_name() {
        assert!(super::is_hashed_name("0123456789abcdefghijklmnopqrstuv-bash"));
        assert!(!super::is_hashed_name("0123456789abcdefghijklmnopqrstuv-"));
        assert!(!super::is_hashed_name("0123456789abcdefghijklmnopqrstuvwbash"));
        assert!(!super::is_hashed_name("0123456789abcdefghijklmnopqrstu.-bash"));
    }

    #[test]
    fn writable_namespace_protects_entries_only_when_sticky() {
        assert!(super::protects_entries(0o555));
        assert!(super::protects_entries(0o1775));
        assert!(!super::protects_entries(0o775));
    }
}
