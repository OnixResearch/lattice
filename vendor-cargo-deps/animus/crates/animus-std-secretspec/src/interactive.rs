//! Interactive masked secret entry with path-only model delivery.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

const PRIVATE_FILE_MODE: u32 = 0o600;
const SECRET_FILE_PREFIX: &str = "secret-";

/// Session class requesting interactive secret entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretEntrySessionClass {
    /// Approved interactive root session.
    RootInteractive,
    /// Headless session.
    Headless,
    /// Child session.
    Child,
}

/// Authority facts supplied by the terminal and secret-host shells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretEntryAuthority {
    /// Requesting session class.
    pub session_class: SecretEntrySessionClass,
    /// Approved terminal supports masked input.
    pub masked_terminal_admitted: bool,
    /// Secret file host is admitted for this runtime.
    pub secret_host_admitted: bool,
    /// Separate reviewed authority explicitly admits a non-root session.
    pub separate_authority_admitted: bool,
}

/// Secret-entry request with no secret value field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretEntryRequest<'a> {
    /// Stable safe request identity used in the private file name.
    pub request_identity: &'a str,
    /// Safe prompt label shown by the terminal shell.
    pub prompt_label: &'a str,
    /// Maximum accepted secret byte count.
    pub max_secret_bytes: u32,
}

/// Path-only model-facing result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretPathResult {
    path: PathBuf,
}

impl SecretPathResult {
    /// Returns the admitted temporary secret-file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

/// Approved terminal capability that guarantees masked input.
pub trait MaskedSecretInput {
    /// Reads one secret without echoing or recording it.
    ///
    /// # Errors
    ///
    /// Returns a typed terminal failure without the secret value.
    fn read_masked(&mut self, prompt_label: &str) -> Result<Zeroizing<Vec<u8>>, SecretEntryError>;
}

/// Interactive secret-entry failure without secret bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretEntryError {
    /// Root or explicit separate session authority is absent.
    AuthorityDenied,
    /// An approved masked terminal is absent.
    MaskedTerminalUnavailable,
    /// Request identity or prompt metadata is unsafe.
    InvalidRequest,
    /// Secret input was empty or exceeded its bound.
    SecretBounds,
    /// Scratch root is absent, a symlink, or not a directory.
    ScratchRootRejected,
    /// Target exists, escapes, is a symlink, or cannot be created privately.
    TargetRejected,
    /// Secret bytes could not be written.
    WriteFailed,
    /// A result or transcript contains secret bytes.
    SecretLeak,
    /// Owner-only file modes are unavailable on this platform.
    UnsupportedPlatform,
}

/// Exact cleanup failure for one tracked path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretCleanupFailure {
    path: PathBuf,
}

impl SecretCleanupFailure {
    /// Returns the exact tracked path whose removal failed.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

/// Session-local interactive secret-file host.
pub struct InteractiveSecretFileHost {
    scratch_root: PathBuf,
    tracked_paths: Vec<PathBuf>,
}

impl InteractiveSecretFileHost {
    /// Admits an existing non-symlink scratch directory.
    ///
    /// # Errors
    ///
    /// Returns [`SecretEntryError::ScratchRootRejected`] for an unsafe root.
    pub fn new(scratch_root: &Path) -> Result<Self, SecretEntryError> {
        let metadata = std::fs::symlink_metadata(scratch_root)
            .map_err(|_| SecretEntryError::ScratchRootRejected)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(SecretEntryError::ScratchRootRejected);
        }
        let scratch_root = std::fs::canonicalize(scratch_root)
            .map_err(|_| SecretEntryError::ScratchRootRejected)?;
        Ok(Self {
            scratch_root,
            tracked_paths: Vec::new(),
        })
    }

    /// Reads through an approved masked terminal and returns only a private path.
    ///
    /// # Errors
    ///
    /// Fails before prompting when authority or request metadata is invalid. File and input
    /// failures return bounded classifications without secret bytes.
    // r[impl animus.secret.path]
    // r[impl animus.secret.file]
    // r[impl animus.secret.scope]
    pub fn enter_secret(
        &mut self,
        request: SecretEntryRequest<'_>,
        authority: SecretEntryAuthority,
        terminal: &mut impl MaskedSecretInput,
    ) -> Result<SecretPathResult, SecretEntryError> {
        admit_authority(authority)?;
        validate_request(request)?;
        let path = self
            .scratch_root
            .join(format!("{SECRET_FILE_PREFIX}{}", request.request_identity));
        if path.parent() != Some(self.scratch_root.as_path()) || path.exists() {
            return Err(SecretEntryError::TargetRejected);
        }
        let secret = terminal.read_masked(request.prompt_label)?;
        let secret_length =
            u32::try_from(secret.len()).map_err(|_| SecretEntryError::SecretBounds)?;
        if secret_length == 0 || secret_length > request.max_secret_bytes {
            return Err(SecretEntryError::SecretBounds);
        }
        let mut file = open_private_new(path.as_path())?;
        if file.write_all(secret.as_slice()).is_err() || file.sync_data().is_err() {
            drop(file);
            let _cleanup_result = std::fs::remove_file(path.as_path());
            return Err(SecretEntryError::WriteFailed);
        }
        drop(file);
        self.tracked_paths.push(path.clone());
        Ok(SecretPathResult { path })
    }

    /// Removes one tracked file after its caller consumes it.
    ///
    /// # Errors
    ///
    /// Returns [`SecretCleanupFailure`] with the exact tracked path when removal fails.
    pub fn consume(&mut self, result: &SecretPathResult) -> Result<(), SecretCleanupFailure> {
        let Some(index) = self
            .tracked_paths
            .iter()
            .position(|path| path == result.path())
        else {
            return Err(SecretCleanupFailure {
                path: result.path.clone(),
            });
        };
        std::fs::remove_file(result.path()).map_err(|_| SecretCleanupFailure {
            path: result.path.clone(),
        })?;
        self.tracked_paths.remove(index);
        Ok(())
    }

    /// Removes every remaining tracked file during runtime close.
    ///
    /// # Errors
    ///
    /// Returns [`SecretCleanupFailure`] with the first exact failed path. All failed paths remain
    /// tracked so a later close can retry them.
    pub fn close(&mut self) -> Result<(), SecretCleanupFailure> {
        let mut retained = Vec::new();
        for path in self.tracked_paths.drain(..) {
            if std::fs::remove_file(path.as_path()).is_err() {
                retained.push(path);
            }
        }
        self.tracked_paths = retained;
        self.tracked_paths.first().map_or(Ok(()), |path| {
            Err(SecretCleanupFailure { path: path.clone() })
        })
    }

    /// Returns the current tracked file count.
    #[must_use]
    pub fn tracked_count(&self) -> usize {
        self.tracked_paths.len()
    }
}

/// Rejects any model result or transcript that contains the exact secret bytes.
///
/// # Errors
///
/// Returns [`SecretEntryError::SecretLeak`] when secret bytes appear.
pub fn validate_path_only_record(record: &[u8], secret: &[u8]) -> Result<(), SecretEntryError> {
    if secret.is_empty() || record.windows(secret.len()).any(|window| window == secret) {
        Err(SecretEntryError::SecretLeak)
    } else {
        Ok(())
    }
}

fn admit_authority(authority: SecretEntryAuthority) -> Result<(), SecretEntryError> {
    if !authority.masked_terminal_admitted {
        return Err(SecretEntryError::MaskedTerminalUnavailable);
    }
    if !authority.secret_host_admitted {
        return Err(SecretEntryError::AuthorityDenied);
    }
    if authority.session_class != SecretEntrySessionClass::RootInteractive
        && !authority.separate_authority_admitted
    {
        return Err(SecretEntryError::AuthorityDenied);
    }
    Ok(())
}

fn validate_request(request: SecretEntryRequest<'_>) -> Result<(), SecretEntryError> {
    if request.request_identity.is_empty()
        || request.prompt_label.is_empty()
        || request.max_secret_bytes == 0
        || request
            .request_identity
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
        || request.prompt_label.chars().any(char::is_control)
    {
        Err(SecretEntryError::InvalidRequest)
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn open_private_new(path: &Path) -> Result<File, SecretEntryError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE_MODE)
        .open(path)
        .map_err(|_| SecretEntryError::TargetRejected)
}

#[cfg(not(unix))]
fn open_private_new(_path: &Path) -> Result<File, SecretEntryError> {
    Err(SecretEntryError::UnsupportedPlatform)
}
