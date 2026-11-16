use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;

const DIRECTORY_MODE: u32 = 0o750;
const FILE_MODE: u32 = 0o640;
const EVIDENCE_FILES: usize = 2;

pub(super) fn write(
    root: &std::path::Path,
    payload: &super::content::Payload,
    observation: &serde_json::Value,
) -> Result<(), String> {
    check_bounds(payload)?;
    let native = serde_json::to_vec_pretty(observation).map_err(|error| error.to_string())?;
    let tree = Tree {
        root,
        payload,
        native: &native,
        directories: directories(&payload.snapshot),
    };
    tree.publish()
}

fn check_bounds(payload: &super::content::Payload) -> Result<(), String> {
    let snapshot = &payload.snapshot;
    let bytes = snapshot.values().try_fold(0usize, |total, bytes| total.checked_add(bytes.len()));
    if snapshot.len() > crate::candidate::MAX_SNAPSHOT_FILES
        || bytes.is_none_or(|bytes| bytes > crate::candidate::MAX_SNAPSHOT_BYTES)
    {
        return Err("candidate snapshot exceeds bounds".into());
    }
    if snapshot.contains_key("host-receipt.json") || snapshot.contains_key("native-observation.json") {
        return Err("candidate snapshot collides with reserved export evidence".into());
    }
    Ok(())
}

fn directories(
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> std::collections::BTreeSet<std::path::PathBuf> {
    let mut directories = std::collections::BTreeSet::new();
    for file in snapshot.keys() {
        let mut directory = std::path::Path::new(file).parent();
        while let Some(path) = directory.filter(|path| !path.as_os_str().is_empty()) {
            directories.insert(path.to_path_buf());
            directory = path.parent();
        }
    }
    directories
}

struct Tree<'a> {
    root: &'a std::path::Path,
    payload: &'a super::content::Payload,
    native: &'a [u8],
    directories: std::collections::BTreeSet<std::path::PathBuf>,
}

impl Tree<'_> {
    fn publish(&self) -> Result<(), String> {
        if self.root.exists() {
            return self.verify();
        }
        std::fs::create_dir(self.root).map_err(|error| error.to_string())?;
        set_mode(self.root, DIRECTORY_MODE)?;
        self.write_members()?;
        self.synchronize()?;
        self.verify()?;
        sync(self.root.parent().ok_or("candidate output has no parent")?)
    }

    fn write_members(&self) -> Result<(), String> {
        for (relative, bytes) in &self.payload.snapshot {
            self.write_member(relative, bytes)?;
        }
        write_new(&self.root.join("host-receipt.json"), &self.payload.receipt, FILE_MODE)?;
        write_new(&self.root.join("native-observation.json"), self.native, FILE_MODE)
    }

    fn write_member(&self, relative: &str, bytes: &[u8]) -> Result<(), String> {
        let path = std::path::Path::new(relative);
        ensure_parents(self.root, path)?;
        let source_mode = self.payload.modes.get(relative).copied().unwrap_or(0);
        write_new(&self.root.join(path), bytes, FILE_MODE | (source_mode & 0o111))
    }

    fn synchronize(&self) -> Result<(), String> {
        // Prefix order puts each parent before its descendants; sync in reverse.
        for relative in self.directories.iter().rev() {
            sync(&self.root.join(relative))?;
        }
        sync(self.root)
    }

    fn verify(&self) -> Result<(), String> {
        check_root(self.root)?;
        let mut actual_files = self.verify_directory(self.root)?;
        for relative in &self.directories {
            let members = self.verify_directory(&self.root.join(relative))?;
            actual_files = actual_files.checked_add(members).ok_or("candidate output member count overflow")?;
        }
        let expected_files = self
            .payload
            .snapshot
            .len()
            .checked_add(EVIDENCE_FILES)
            .ok_or("candidate output member count overflow")?;
        if actual_files != expected_files {
            return Err("candidate output conflicts with exact replay".into());
        }
        Ok(())
    }

    fn verify_directory(&self, path: &std::path::Path) -> Result<usize, String> {
        let mut entries = std::fs::read_dir(path).map_err(|error| error.to_string())?;
        entries.try_fold(0usize, |count, entry| {
            let entry = entry.map_err(|error| error.to_string())?;
            let files = self.verify_entry(&entry.path())?;
            count.checked_add(files).ok_or_else(|| "candidate output member count overflow".into())
        })
    }

    fn verify_entry(&self, path: &std::path::Path) -> Result<usize, String> {
        let metadata = entry_metadata(path)?;
        let relative = path.strip_prefix(self.root).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            self.check_directory(relative, &metadata)?;
            Ok(0)
        } else {
            self.check_file(path, relative, &metadata)?;
            Ok(1)
        }
    }

    fn check_directory(&self, relative: &std::path::Path, metadata: &std::fs::Metadata) -> Result<(), String> {
        if !self.directories.contains(relative) || metadata.mode() & 0o777 != DIRECTORY_MODE {
            return Err("candidate output directory mode conflicts with replay".into());
        }
        Ok(())
    }

    fn check_file(
        &self,
        path: &std::path::Path,
        relative: &std::path::Path,
        metadata: &std::fs::Metadata,
    ) -> Result<(), String> {
        if !metadata.is_file() || metadata.nlink() != 1 {
            return Err("candidate output contains a special or hard-linked entry".into());
        }
        let (expected, mode) = self.expected(relative).ok_or("candidate output conflicts with exact replay")?;
        let actual = super::super::artifacts::read_bounded(path, expected.len())?;
        if actual != expected {
            return Err("candidate output ownership or content conflicts with replay".into());
        }
        if metadata.mode() & 0o777 != mode {
            return Err("candidate output mode conflicts with replay".into());
        }
        Ok(())
    }

    fn expected(&self, relative: &std::path::Path) -> Option<(&[u8], u32)> {
        let name = relative.to_str()?;
        if name == "host-receipt.json" {
            return Some((&self.payload.receipt, FILE_MODE));
        }
        if name == "native-observation.json" {
            return Some((self.native, FILE_MODE));
        }
        let bytes = self.payload.snapshot.get(name)?;
        let source_mode = self.payload.modes.get(name).copied().unwrap_or(0);
        Some((bytes, FILE_MODE | (source_mode & 0o111)))
    }
}

fn check_root(root: &std::path::Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || !super::admission::is_owned(&metadata) || metadata.mode() & 0o777 != DIRECTORY_MODE {
        return Err("candidate output root has unsafe type, ownership or mode".into());
    }
    Ok(())
}

fn entry_metadata(path: &std::path::Path) -> Result<std::fs::Metadata, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !super::admission::is_owned(&metadata) {
        return Err("candidate output contains an unsafe or foreign entry".into());
    }
    Ok(metadata)
}

fn ensure_parents(root: &std::path::Path, file: &std::path::Path) -> Result<(), String> {
    let parent = file.parent().ok_or("candidate path has no parent")?;
    let mut directory = root.to_path_buf();
    for component in parent.components() {
        directory.push(component);
        if !directory.exists() {
            std::fs::create_dir(&directory).map_err(|error| error.to_string())?;
            set_mode(&directory, DIRECTORY_MODE)?;
        }
    }
    Ok(())
}

fn set_mode(path: &std::path::Path, mode: u32) -> Result<(), String> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|error| error.to_string())
}

fn write_new(path: &std::path::Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true).mode(mode);
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|error| error.to_string())?;
    set_mode(path, mode)?;
    sync(path)
}

fn sync(path: &std::path::Path) -> Result<(), String> {
    std::fs::File::open(path).and_then(|file| file.sync_all()).map_err(|error| error.to_string())
}
