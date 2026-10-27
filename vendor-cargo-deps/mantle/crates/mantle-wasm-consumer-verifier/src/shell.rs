//! Standard-library file verifier over one explicit capability root.
//!
//! The shell receives one opened root directory and the declared member
//! logical paths of a bundle. It rejects absolute escapes, parent traversal,
//! symlinks, non-regular files, oversized members, and drifted bytes. It
//! performs no registry, store, network, or ambient path search.

use std::io::Read;
use std::path::Path;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::fs::Dir;
use cap_std::fs::OpenOptionsExt;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::StoreObject;

use crate::contract::ConsumerLayer;
use crate::contract::ConsumerMemberKind;
use crate::contract::ConsumerSourceFile;
#[path = "source_tree.rs"]
mod source_tree;
use crate::contract::ConsumerMemberObservation;
use crate::contract::ConsumerMemberStatus;
use crate::contract::ConsumerVerificationReport;
use crate::contract::ConsumerVerificationStatus;
use crate::facade::declared_member_objects;
use crate::facade::verify_consumer_bundle;

/// Maximum bytes remeasured for one member.
pub const MAX_MEMBER_BYTES: u64 = 1_073_741_824;

/// Maximum combined bytes remeasured for one bundle.
pub const MAX_TOTAL_REMEASURED_BYTES: u64 = 4_294_967_296;

/// Blocker code when remeasured bytes drift from declared identities.
pub const BLOCKER_MEMBER_DRIFT: &str = "consumer-member-bytes-drift";

/// Blocker code when the combined remeasured size exceeds the bound.
pub const BLOCKER_TOTAL_BOUND: &str = "consumer-total-byte-bound";

/// Errors surfaced by the capability-root shell.
#[derive(Debug)]
pub enum ShellError {
    /// Root is not a directory.
    RootNotDirectory,
    /// A member path is empty, escapes the root, or traverses parents.
    InvalidMemberPath,
    /// One logical path is declared with conflicting member identities.
    ConflictingMemberIdentity,
}

/// Capability root for consumer member resolution.
pub struct ConsumerRoot {
    root: Dir,
}

/// One required member and its declared facts.
struct RequiredMember {
    role: String,
    logical_path: String,
    digest_blake3: String,
    size_bytes: u64,
}

/// One measured logical path.
enum Measurement {
    Missing,
    LengthMismatch,
    Identity {
        digest: String,
        kind: ConsumerMemberKind,
        source_files: Vec<ConsumerSourceFile>,
    },
}

impl ConsumerRoot {
    /// Open an existing directory as the only member-resolution root.
    pub fn open(root: &Path) -> Result<Self, ShellError> {
        let root =
            Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|_| ShellError::RootNotDirectory)?;
        Ok(Self { root })
    }

    /// Verify a bundle structurally, then remeasure every required member.
    ///
    /// Structural rejection short-circuits before any filesystem access. A
    /// byte layer that cannot resolve required bytes reports `blocked`;
    /// drifted bytes report the drift blocker and `rejected`.
    pub fn verify_bundle(&self, bundle: MaterializationBundle) -> Result<ConsumerVerificationReport, ShellError> {
        let mut structural_outcome = verify_consumer_bundle(bundle.clone());
        if structural_outcome.status != ConsumerVerificationStatus::Verified {
            return Ok(structural_outcome);
        }
        let required = required_members(&bundle)?;
        self.run_byte_layer(&required, &mut structural_outcome);
        Ok(structural_outcome)
    }

    fn run_byte_layer(&self, required: &[RequiredMember], structural_outcome: &mut ConsumerVerificationReport) {
        let mut members = Vec::with_capacity(required.len());
        let mut missing = 0_u32;
        let mut mismatched = 0_u32;
        let mut total_bytes = 0_u64;
        // Measure each unique logical path once, then judge every member
        // that declares it against its own expected identity.
        let mut measured: Vec<(&str, Measurement)> = Vec::with_capacity(required.len());
        for member in required {
            if measured.iter().any(|(path, _)| *path == member.logical_path.as_str()) {
                continue;
            }
            let Some(next_total) =
                total_bytes.checked_add(member.size_bytes).filter(|total| *total <= MAX_TOTAL_REMEASURED_BYTES)
            else {
                structural_outcome.status = ConsumerVerificationStatus::Rejected;
                structural_outcome.blockers.push(String::from(BLOCKER_TOTAL_BOUND));
                return;
            };
            total_bytes = next_total;
            let measurement = self.measure_path(member);
            measured.push((member.logical_path.as_str(), measurement));
        }
        for member in required {
            let shared_measurement = match measured.iter().find(|(path, _)| *path == member.logical_path.as_str()) {
                Some((_, measurement)) => measurement,
                None => &Measurement::Missing,
            };
            let status = match shared_measurement {
                Measurement::Missing => ConsumerMemberStatus::Missing,
                Measurement::LengthMismatch => ConsumerMemberStatus::Mismatched,
                Measurement::Identity {
                    kind: ConsumerMemberKind::SourceTree,
                    ..
                } if member.role != "source-closure" => ConsumerMemberStatus::Missing,
                Measurement::Identity { digest, .. } => {
                    if *digest == member.digest_blake3 {
                        ConsumerMemberStatus::Matched
                    } else {
                        ConsumerMemberStatus::Mismatched
                    }
                }
            };
            match status {
                ConsumerMemberStatus::Matched => {}
                ConsumerMemberStatus::Missing => missing += 1,
                ConsumerMemberStatus::Mismatched => mismatched += 1,
            }
            debug_assert!(matches!(
                status,
                ConsumerMemberStatus::Matched | ConsumerMemberStatus::Missing | ConsumerMemberStatus::Mismatched
            ));
            let doubled = match required.len().checked_mul(2) {
                Some(bound) => bound,
                None => required.len(),
            };
            assert!(members.len() < doubled, "member observation bound exceeded");
            members.push(ConsumerMemberObservation {
                role: member.role.clone(),
                digest_blake3: member.digest_blake3.clone(),
                size_bytes: member.size_bytes,
                kind: match shared_measurement {
                    Measurement::Identity { kind, .. } => Some(*kind),
                    _ => None,
                },
                source_files: match shared_measurement {
                    Measurement::Identity { source_files, .. } => source_files.clone(),
                    _ => Vec::new(),
                },
                status: Some(status),
            });
        }
        structural_outcome.members = members;
        structural_outcome.layers_completed.push(ConsumerLayer::Bytes);
        structural_outcome.status = if total_bytes > MAX_TOTAL_REMEASURED_BYTES {
            structural_outcome.blockers.push(String::from(BLOCKER_TOTAL_BOUND));
            ConsumerVerificationStatus::Rejected
        } else if missing > 0 {
            ConsumerVerificationStatus::Blocked
        } else if mismatched > 0 {
            structural_outcome.blockers.push(String::from(BLOCKER_MEMBER_DRIFT));
            ConsumerVerificationStatus::Rejected
        } else {
            debug_assert!(structural_outcome.blockers.is_empty());
            debug_assert!(missing == 0 && mismatched == 0);
            ConsumerVerificationStatus::Verified
        };
    }

    fn measure_path(&self, member: &RequiredMember) -> Measurement {
        let relative = match relative_member_path(&member.logical_path) {
            Ok(relative) => relative,
            Err(_) => return Measurement::Missing,
        };
        let mut parts = relative.rsplitn(2, '/');
        let name = parts.next().expect("nonempty member path");
        let parent_path = parts.next().unwrap_or("");
        let mut parent = match self.root.try_clone() {
            Ok(root) => root,
            Err(_) => return Measurement::Missing,
        };
        for part in parent_path.split('/').filter(|part| !part.is_empty()) {
            parent = match parent.open_dir_nofollow(part) {
                Ok(directory) => directory,
                Err(_) => return Measurement::Missing,
            };
        }
        if member.role == "source-closure"
            && let Ok(directory) = parent.open_dir_nofollow(name)
        {
            return source_tree::measure(directory, member.size_bytes);
        }
        if member.size_bytes > MAX_MEMBER_BYTES {
            return Measurement::LengthMismatch;
        }
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No).custom_flags(0o4000);
        let file = match parent.open_with(name, &options) {
            Ok(file) => file,
            Err(_) => return Measurement::Missing,
        };
        let metadata = match file.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => return Measurement::Missing,
        };
        if metadata.len() != member.size_bytes {
            return Measurement::LengthMismatch;
        }
        let mut hasher = blake3::Hasher::new();
        let mut reader = file.take(member.size_bytes.saturating_add(1));
        let mut buffer = [0_u8; 65_536];
        let mut read = 0_u64;
        loop {
            let count = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => count,
                Err(_) => return Measurement::Missing,
            };
            read += count as u64;
            if read > member.size_bytes {
                return Measurement::LengthMismatch;
            }
            hasher.update(&buffer[..count]);
        }
        if read != member.size_bytes {
            return Measurement::LengthMismatch;
        }
        Measurement::Identity {
            digest: hasher.finalize().to_hex().to_string(),
            kind: ConsumerMemberKind::File,
            source_files: Vec::new(),
        }
    }
}

/// Collect every declared store object as a required byte member.
///
/// Roles may share one logical path; the byte layer measures each unique
/// path once and judges every declaring member against its own identity.
/// Collect every declared store object as a required byte member.
///
/// Roles may share one logical path when they declare the same identity;
/// a path declared with conflicting digests is member substitution and is
/// rejected before any filesystem access.
fn required_members(bundle: &MaterializationBundle) -> Result<Vec<RequiredMember>, ShellError> {
    let declared = declared_member_objects(bundle);
    let mut members = Vec::with_capacity(declared.len());
    for (role, object) in &declared {
        let StoreObject {
            logical_path,
            digest_blake3,
            size_bytes,
        } = object;
        relative_member_path(logical_path)?;
        debug_assert!(!(*role).is_empty());
        let digest_hex = digest_blake3.to_string();
        let has_conflicting_identity = members.iter().any(|member: &RequiredMember| {
            member.logical_path == logical_path.as_str()
                && (member.digest_blake3 != digest_hex || member.size_bytes != *size_bytes)
        });
        if has_conflicting_identity {
            return Err(ShellError::ConflictingMemberIdentity);
        }
        members.push(RequiredMember {
            role: String::from(*role),
            logical_path: logical_path.clone(),
            digest_blake3: digest_hex,
            size_bytes: *size_bytes,
        });
    }
    debug_assert_eq!(members.len(), declared.len());
    Ok(members)
}

/// Map a member logical store path to its root-relative consumer path.
///
/// The consumer root mirrors the declared member tree: the leading `/` of
/// the logical path becomes the root-relative layout. Parent, current, and
/// empty components are rejected before any filesystem access.
fn relative_member_path(logical_path: &str) -> Result<&str, ShellError> {
    let Some(relative) = logical_path.strip_prefix('/') else {
        return Err(ShellError::InvalidMemberPath);
    };
    if relative.is_empty() {
        return Err(ShellError::InvalidMemberPath);
    }
    for component in relative.split('/') {
        if component.is_empty() || component == ".." || component == "." {
            return Err(ShellError::InvalidMemberPath);
        }
    }
    debug_assert!(!relative.starts_with('/') && !relative.ends_with('/'));
    Ok(relative)
}
