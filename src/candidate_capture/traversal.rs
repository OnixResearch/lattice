pub(super) fn snapshot(root: &std::path::Path) -> Result<std::collections::BTreeMap<String, Vec<u8>>, super::Error> {
    Snapshot {
        files: std::collections::BTreeMap::new(),
        total_bytes: 0,
        observed_entries: 0,
        directories: vec![(root.to_path_buf(), std::path::PathBuf::new())],
    }
    .collect()
}

struct Snapshot {
    files: std::collections::BTreeMap<String, Vec<u8>>,
    total_bytes: usize,
    observed_entries: usize,
    directories: Vec<(std::path::PathBuf, std::path::PathBuf)>,
}

impl Snapshot {
    fn collect(mut self) -> Result<std::collections::BTreeMap<String, Vec<u8>>, super::Error> {
        while let Some((directory, relative)) = self.directories.pop() {
            self.directory(&directory, &relative)?;
        }
        Ok(self.files)
    }

    fn directory(&mut self, directory: &std::path::Path, relative: &std::path::Path) -> Result<(), super::Error> {
        for entry in std::fs::read_dir(directory).map_err(super::Error::Io)? {
            self.entry(&entry.map_err(super::Error::Io)?, relative)?;
        }
        Ok(())
    }

    fn entry(&mut self, entry: &std::fs::DirEntry, relative: &std::path::Path) -> Result<(), super::Error> {
        let file_name = entry.file_name();
        let name = file_name.to_str().ok_or(super::Error::NonUtf8Path)?;
        if relative.as_os_str().is_empty() && matches!(name, ".git" | ".jj") {
            return Ok(());
        }
        self.count_entry()?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(super::Error::Io)?;
        Self::admit(&metadata)?;
        let relative_path = relative.join(name);
        if metadata.is_dir() {
            self.directories.push((path, relative_path));
            Ok(())
        } else {
            self.file(&path, &relative_path, &metadata)
        }
    }

    fn count_entry(&mut self) -> Result<(), super::Error> {
        self.observed_entries = self.observed_entries.checked_add(1).ok_or(super::Error::TooLarge)?;
        if self.observed_entries > crate::candidate::MAX_SNAPSHOT_FILES {
            return Err(super::Error::TooLarge);
        }
        Ok(())
    }

    fn admit(metadata: &std::fs::Metadata) -> Result<(), super::Error> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.is_file() && metadata.nlink() > 1 {
                return Err(super::Error::HardLinkRejected);
            }
        }
        if metadata.file_type().is_symlink() {
            return Err(super::Error::SymlinkRejected);
        }
        Ok(())
    }

    fn file(
        &mut self,
        path: &std::path::Path,
        relative: &std::path::Path,
        metadata: &std::fs::Metadata,
    ) -> Result<(), super::Error> {
        debug_assert!(self.files.len() <= self.observed_entries);
        debug_assert!(self.total_bytes <= crate::candidate::MAX_SNAPSHOT_BYTES);
        if !metadata.is_file() {
            return Err(super::Error::SpecialFileRejected);
        }
        if self.files.len() >= crate::candidate::MAX_SNAPSHOT_FILES {
            return Err(super::Error::TooLarge);
        }
        let bytes = super::read_file(path, metadata, self.total_bytes)?;
        self.total_bytes = self.total_bytes.checked_add(bytes.len()).ok_or(super::Error::TooLarge)?;
        if self.total_bytes > crate::candidate::MAX_SNAPSHOT_BYTES {
            return Err(super::Error::TooLarge);
        }
        self.files.insert(super::path_key(relative)?, bytes);
        Ok(())
    }
}
