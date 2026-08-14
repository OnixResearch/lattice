//! Bounded filesystem and process adapters for guarded rounds.

use std::io::Read;
use std::io::Write;

const CAIRN_CHANGES_DIRECTORY: &str = ".cairn/changes";
const OCTET_POLICY_FILE: &str = "dylint.toml";
const OCTET_BACKUP_FILE: &str = "policy-backup.toml";
const OCTET_TEMPORARY_POLICY_FILE: &str = ".lattice-guarded-octet-policy.tmp";
const OCTET_SUMMARY_FILE: &str = "summary.txt";
const OCTET_STDOUT_FILE: &str = "probe-stdout.log";
const OCTET_STDERR_FILE: &str = "probe-stderr.log";
const OCTET_FINDINGS_PREFIX: &str = "Findings:";
const MERGED_OBSERVATION_COUNT: usize = 3;
const NO_DISABLED_OCTET_POLICY: &[u8] = b"[octet]\ndisabled_lints = []\n";
const MAX_POLICY_BYTES: usize = 1_048_576;
const MAX_SUMMARY_BYTES: usize = 1_048_576;
const MAX_PROBE_OUTPUT_BYTES: usize = 16_777_216;
const MAX_PROBE_SLOTS: usize = 64;
const PROBE_SLOT_DIGITS: usize = 3;
const MINIMUM_TIMEOUT_SECS: u64 = 1;
const MAXIMUM_TIMEOUT_SECS: u64 = 3_600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    RootUnavailable,
    UnsafePath,
    SymlinkRejected,
    LimitExceeded,
    MalformedObservation,
    ProcessFailed,
    RestorationFailed,
    RecoveryBlocked,
    ArtifactConflict,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::RootUnavailable => "guarded repository root is unavailable",
            Self::UnsafePath => "guarded relative path is unsafe",
            Self::SymlinkRejected => "guarded path contains a rejected symbolic link",
            Self::LimitExceeded => "guarded adapter limit was exceeded",
            Self::MalformedObservation => "guarded observation is malformed",
            Self::ProcessFailed => "guarded bounded process failed",
            Self::RestorationFailed => "guarded policy restoration failed",
            Self::RecoveryBlocked => "guarded recovery requires operator inspection",
            Self::ArtifactConflict => "guarded artifact path already contains recovery state",
        })
    }
}

impl std::error::Error for Error {}

pub trait ProbeProcess {
    fn run(&self, request: &crate::agent_node::ProcessRequest) -> Result<crate::agent_node::ProcessOutput, Error>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoundedProbeProcess;

impl ProbeProcess for BoundedProbeProcess {
    fn run(&self, request: &crate::agent_node::ProcessRequest) -> Result<crate::agent_node::ProcessOutput, Error> {
        crate::process::shell::run_bounded(request, &crate::process::core::RuntimeConfig::default(), None)
            .map_err(|_| Error::ProcessFailed)
    }
}

include!("shell/cairn.rs");
include!("shell/octet.rs");
include!("shell/decision.rs");
