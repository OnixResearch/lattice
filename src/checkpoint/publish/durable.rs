//! Durable file writes: a synced temporary file renamed into place, then a synced directory.

use std::io::Write as _;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

use crate::checkpoint::Refusal;
use crate::checkpoint::core::TEMPORARY_PREFIX;
use crate::checkpoint::core::TEMPORARY_SUFFIX;
use crate::checkpoint::core::is_temporary_name;

/// Distinguishes this process's temporary files; the process id distinguishes processes.
static TEMPORARY_SEQUENCE: AtomicU32 = AtomicU32::new(0);

fn temporary_name() -> String {
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{TEMPORARY_PREFIX}{:08x}{sequence:08x}{TEMPORARY_SUFFIX}", std::process::id())
}

/// Remove the temporary files a crashed write left directly in `directory`, if it exists.
pub(in crate::checkpoint) fn remove_temporaries(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let is_leftover = entry.file_name().to_str().is_some_and(is_temporary_name)
            && entry.file_type().is_ok_and(|kind| kind.is_file());
        if is_leftover {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

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
    let temporary = directory.join(temporary_name());
    let mut file =
        std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|_| Refusal::Io)?;
    let written = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
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
