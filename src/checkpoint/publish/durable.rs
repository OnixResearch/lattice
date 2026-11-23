//! Durable file writes: a synced temporary file renamed into place, then a synced directory.

use std::io::Write as _;
use std::path::Path;

use crate::checkpoint::Refusal;

/// Write `name` in `directory` through [`place_atomic`], then [`sync_directory`].
pub(in crate::checkpoint) fn write_atomic(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), Refusal> {
    place_atomic(directory, name, bytes)?;
    sync_directory(directory)
}

/// Put `bytes` at `name` in `directory` through a fsynced temporary file renamed into place. A
/// failure removes the temporary file, so nothing partial is left behind and `name` holds either
/// its complete content or nothing new. The rename is durable only after [`sync_directory`].
pub(in crate::checkpoint) fn place_atomic(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), Refusal> {
    std::fs::create_dir_all(directory).map_err(|_| Refusal::Io)?;
    let temporary = directory.join(format!(".{name}.{}.tmp", std::process::id()));
    let written = std::fs::File::create(&temporary)
        .and_then(|mut file| file.write_all(bytes).and_then(|()| file.sync_all()))
        .and_then(|()| std::fs::rename(&temporary, directory.join(name)));
    if written.is_err() {
        let _removed = std::fs::remove_file(&temporary);
        return Err(Refusal::Io);
    }
    Ok(())
}

/// Fsync `directory`, making the renames into it durable.
pub(in crate::checkpoint) fn sync_directory(directory: &Path) -> Result<(), Refusal> {
    std::fs::File::open(directory).and_then(|directory| directory.sync_all()).map_err(|_| Refusal::Io)
}
