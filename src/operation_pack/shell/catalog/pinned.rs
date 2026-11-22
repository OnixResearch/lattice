//! Private staging directories pinned by capability handles from creation through rename.
//!
//! A staging directory is created mode 0700 through a handle on its parent, and every entry it
//! publishes is renamed relative to that handle and the target directory's handle. Before a
//! path-based reader (the strict pack loader or the workflow-template loader) and again before
//! each rename, the staging path and the published entry must still be the exact inodes that
//! were created, so a swapped directory is refused instead of published.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "filesystem observations return typed errors instead of terminating the process"
)]

use cap_std::fs::DirBuilderExt as _;
use cap_std::fs::MetadataExt as _;

const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const GROUP_AND_OTHER_BITS: u32 = 0o077;
const CURRENT_DIRECTORY: &str = ".";

/// Device and inode of one directory, observed without following a final symlink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Inode {
    device: u64,
    inode: u64,
}

/// The pinned directory and the new entry name a staged entry is published as.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Target<'target> {
    pub(crate) dir: &'target cap_std::fs::Dir,
    pub(crate) name: &'target str,
}

/// One private staging directory beside a target, held open for its whole lifetime.
#[derive(Debug)]
pub(crate) struct Staging {
    parent: cap_std::fs::Dir,
    name: String,
    path: std::path::PathBuf,
    dir: cap_std::fs::Dir,
    inode: Inode,
}

impl Staging {
    /// Create `<parent>/<name>` create-only with mode 0700 and pin it.
    pub(crate) fn create(parent_path: &std::path::Path, name: &str) -> Result<Self, super::super::super::Error> {
        let parent = cap_std::fs::Dir::open_ambient_dir(parent_path, cap_std::ambient_authority())
            .map_err(|_| super::super::super::Error::Io("staging parent authority"))?;
        let mut builder = cap_std::fs::DirBuilder::new();
        builder.mode(PRIVATE_DIRECTORY_MODE);
        parent
            .create_dir_with(name, &builder)
            .map_err(|_| super::super::super::Error::Conflict("staging directory exists"))?;
        let dir = parent.open_dir(name).map_err(|_| super::super::super::Error::Io("staging directory open"))?;
        let metadata =
            dir.dir_metadata().map_err(|_| super::super::super::Error::Io("staging directory observation"))?;
        let staging = Self {
            path: parent_path.join(name),
            name: name.to_string(),
            inode: Inode {
                device: metadata.dev(),
                inode: metadata.ino(),
            },
            parent,
            dir,
        };
        if metadata.mode() & GROUP_AND_OTHER_BITS != 0 {
            staging.remove()?;
            return Err(super::super::super::Error::Invalid("staging directory mode"));
        }
        staging.ensure_pinned()?;
        Ok(staging)
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// The pinned handle on the directory that holds the staging directory.
    pub(crate) fn parent(&self) -> &cap_std::fs::Dir {
        &self.parent
    }

    pub(crate) fn dir(&self) -> &cap_std::fs::Dir {
        &self.dir
    }

    /// The staging path, by name and through the parent handle, still names the pinned inode.
    pub(crate) fn ensure_pinned(&self) -> Result<(), super::super::super::Error> {
        let by_path = std::fs::symlink_metadata(&self.path)
            .map_err(|_| super::super::super::Error::Conflict("staging directory replaced"))?;
        let by_parent = self
            .parent
            .symlink_metadata(&self.name)
            .map_err(|_| super::super::super::Error::Conflict("staging directory replaced"))?;
        let path_inode = Inode {
            device: std::os::unix::fs::MetadataExt::dev(&by_path),
            inode: std::os::unix::fs::MetadataExt::ino(&by_path),
        };
        let parent_inode = Inode {
            device: by_parent.dev(),
            inode: by_parent.ino(),
        };
        if !by_path.is_dir() || path_inode != self.inode || parent_inode != self.inode {
            return Err(super::super::super::Error::Conflict("staging directory replaced"));
        }
        Ok(())
    }

    /// The inode of one directory entry inside the pinned staging directory.
    pub(crate) fn entry_inode(&self, entry: &str) -> Result<Inode, super::super::super::Error> {
        directory_inode(&self.dir, entry)
    }

    /// Rename the pinned entry into `target` create-only, relative to both handles.
    pub(crate) fn publish(
        &self,
        entry: &str,
        expected: Inode,
        target: Target<'_>,
    ) -> Result<(), super::super::super::Error> {
        self.ensure_pinned()?;
        if self.entry_inode(entry)? != expected {
            return Err(super::super::super::Error::Conflict("staged entry replaced"));
        }
        let is_absent = matches!(
            target.dir.symlink_metadata(target.name),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        );
        if !is_absent {
            return Err(super::super::super::Error::Conflict("publication target exists"));
        }
        self.dir
            .rename(entry, target.dir, target.name)
            .map_err(|_| super::super::super::Error::Io("staged entry publish"))?;
        sync(target.dir)
    }

    /// Remove the staging directory through its parent handle and sync the parent.
    pub(crate) fn remove(self) -> Result<(), super::super::super::Error> {
        self.parent
            .remove_dir_all(&self.name)
            .map_err(|_| super::super::super::Error::Io("staging cleanup"))?;
        sync(&self.parent)
    }
}

/// The inode of a directory entry, refusing a symlink or another file kind.
pub(crate) fn directory_inode(dir: &cap_std::fs::Dir, entry: &str) -> Result<Inode, super::super::super::Error> {
    let metadata = dir
        .symlink_metadata(entry)
        .map_err(|_| super::super::super::Error::Conflict("staged entry replaced"))?;
    if !metadata.is_dir() {
        return Err(super::super::super::Error::Conflict("staged entry replaced"));
    }
    Ok(Inode {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

pub(crate) fn sync(dir: &cap_std::fs::Dir) -> Result<(), super::super::super::Error> {
    dir.open(CURRENT_DIRECTORY)
        .and_then(|opened| opened.sync_all())
        .map_err(|_| super::super::super::Error::Io("directory sync"))
}

#[cfg(test)]
mod tests;
