//! Candidate-producing command worker over the existing bounded process route.
//!
//! The worker is an imperative shell. It captures the admitted workspace before
//! and after a clear-environment bounded process and derives the candidate
//! receipt from the actual file maps. It does not provide OS isolation or
//! publication authority.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use thiserror::Error;

use crate::agent_node::ProcessError;
use crate::agent_node::ProcessOutput;
use crate::agent_node::ProcessRequest;
use crate::candidate::CandidateDisposition;
use crate::candidate::CandidateReceipt;
use crate::candidate::CandidateReceiptError;
use crate::candidate::produced_receipt_from_snapshots;
use crate::candidate_capture::CandidateCaptureError;
use crate::candidate_capture::WorkspaceBoundary;
use crate::candidate_capture::capture_workspace;
use crate::process::core::RuntimeConfig;
use crate::process::shell::run_bounded_clear_environment_with_cancellation;
use crate::pure_inet_profile::blake3_identity;
use crate::store::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateWorkerRequest {
    pub work_id: String,
    pub attempt_id: String,
    pub fence: String,
    pub workspace_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub boundary: WorkspaceBoundary,
    pub timeout_secs: u64,
    pub output_limit_bytes: usize,
}

#[derive(Debug)]
pub enum CandidateWorkerOutcome {
    Produced {
        receipt: CandidateReceipt,
        changed_files: BTreeMap<String, Vec<u8>>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Blocked {
        blocker: String,
    },
    Failed {
        blocker: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Cancelled,
    Unknown {
        blocker: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

/// Run one worker and derive a candidate receipt from the actual workspace.
pub fn run_candidate_worker(
    request: &CandidateWorkerRequest,
    config: &RuntimeConfig,
    cancellation: &AtomicBool,
) -> Result<CandidateWorkerOutcome, CandidateWorkerError> {
    if request.timeout_secs == 0 || request.output_limit_bytes == 0 {
        return Ok(CandidateWorkerOutcome::Blocked {
            blocker: "worker deadline and output limit must be positive".to_string(),
        });
    }
    let root = request.boundary.root.as_path();
    if !root.is_dir() {
        return Ok(CandidateWorkerOutcome::Blocked {
            blocker: "admitted workspace root is unavailable".to_string(),
        });
    }
    let cwd = root.to_str().ok_or(CandidateWorkerError::NonUtf8Root)?.to_string();
    let base = capture_workspace(root)?;
    let process = ProcessRequest {
        program: request.program.clone(),
        args: request.args.clone(),
        cwd: Some(cwd),
        env: BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: request.timeout_secs,
        output_limit_bytes: request.output_limit_bytes,
    };
    let output = match run_bounded_clear_environment_with_cancellation(&process, config, Some(cancellation)) {
        Ok(output) => output,
        Err(ProcessError::Cancelled) => return Ok(CandidateWorkerOutcome::Cancelled),
        Err(ProcessError::Timeout) => {
            return Ok(CandidateWorkerOutcome::Failed {
                blocker: "worker exceeded its admitted deadline".to_string(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }
        Err(ProcessError::OutputLimitExceeded { .. }) => {
            return Ok(CandidateWorkerOutcome::Failed {
                blocker: "worker exceeded its admitted output limit".to_string(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }
        Err(ProcessError::InvalidRequest(_) | ProcessError::Spawn(_)) => {
            return Ok(CandidateWorkerOutcome::Blocked {
                blocker: "worker invocation was not admitted".to_string(),
            });
        }
    };
    finish_from_process_output(request, root, base, output)
}

/// Run one worker and durably publish a produced receipt.
pub fn run_and_publish_candidate_worker(
    request: &CandidateWorkerRequest,
    config: &RuntimeConfig,
    cancellation: &AtomicBool,
    store: &Store,
) -> Result<CandidateWorkerOutcome, CandidateWorkerError> {
    let outcome = run_candidate_worker(request, config, cancellation)?;
    let (receipt, changed_files, stdout, stderr) = match outcome {
        CandidateWorkerOutcome::Produced {
            receipt,
            changed_files,
            stdout,
            stderr,
        } => (receipt, changed_files, stdout, stderr),
        other => return Ok(other),
    };
    if let Err(error) = store.save_candidate_receipt(&receipt) {
        return Ok(CandidateWorkerOutcome::Unknown {
            blocker: format!("candidate receipt publication failed: {error}"),
            stdout,
            stderr,
        });
    }
    for (path, bytes) in &changed_files {
        if let Err(error) = store.save_candidate_artifact(&receipt.attempt_id, path, bytes) {
            return Ok(CandidateWorkerOutcome::Unknown {
                blocker: format!("candidate artifact publication failed: {error}"),
                stdout,
                stderr,
            });
        }
    }
    Ok(CandidateWorkerOutcome::Produced {
        receipt,
        changed_files,
        stdout,
        stderr,
    })
}

fn finish_from_process_output(
    request: &CandidateWorkerRequest,
    root: &Path,
    base: BTreeMap<String, Vec<u8>>,
    output: ProcessOutput,
) -> Result<CandidateWorkerOutcome, CandidateWorkerError> {
    if output.status_code != Some(0) {
        return Ok(CandidateWorkerOutcome::Failed {
            blocker: format!("worker exited with status {:?}", output.status_code),
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    let candidate = match capture_workspace(root) {
        Ok(candidate) => candidate,
        Err(error) => {
            return Ok(CandidateWorkerOutcome::Unknown {
                blocker: format!("candidate capture failed: {error}"),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
    };
    let receipt = match produced_receipt_from_snapshots(
        request.work_id.clone(),
        request.attempt_id.clone(),
        request.fence.clone(),
        request.workspace_id.clone(),
        &base,
        &candidate,
    ) {
        Ok(receipt) => receipt,
        Err(CandidateReceiptError::NoCandidateChanges) => {
            return Ok(CandidateWorkerOutcome::Failed {
                blocker: "worker exited zero without a candidate change".to_string(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        Err(error) => {
            return Ok(CandidateWorkerOutcome::Unknown {
                blocker: format!("candidate receipt failed: {error}"),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
    };
    let changed_files = receipt
        .changed_paths
        .iter()
        .filter_map(|path| candidate.get(path).map(|bytes| (path.clone(), bytes.clone())))
        .collect::<BTreeMap<_, _>>();
    Ok(CandidateWorkerOutcome::Produced {
        receipt,
        changed_files,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CandidateAttemptResult {
    pub schema: String,
    pub disposition: CandidateDisposition,
    pub receipt: Option<CandidateReceipt>,
    pub stdout_blake3: String,
    pub stderr_blake3: String,
    pub blocker: Option<String>,
}

pub const CANDIDATE_ATTEMPT_RESULT_SCHEMA: &str = "lattice.candidate-attempt-result.v1";

impl CandidateAttemptResult {
    /// Project a worker outcome without retaining raw output bytes.
    pub fn from_outcome(outcome: &CandidateWorkerOutcome) -> Self {
        let (disposition, receipt, stdout, stderr, blocker) = match outcome {
            CandidateWorkerOutcome::Produced {
                receipt,
                stdout,
                stderr,
                ..
            } => (CandidateDisposition::Produced, Some(receipt.clone()), stdout, stderr, None),
            CandidateWorkerOutcome::Blocked { blocker } => {
                (CandidateDisposition::Blocked, None, &Vec::new(), &Vec::new(), Some(blocker.clone()))
            }
            CandidateWorkerOutcome::Failed {
                blocker,
                stdout,
                stderr,
            } => (CandidateDisposition::Failed, None, stdout, stderr, Some(blocker.clone())),
            CandidateWorkerOutcome::Cancelled => {
                (CandidateDisposition::Cancelled, None, &Vec::new(), &Vec::new(), None)
            }
            CandidateWorkerOutcome::Unknown {
                blocker,
                stdout,
                stderr,
            } => (CandidateDisposition::Unknown, None, stdout, stderr, Some(blocker.clone())),
        };
        Self {
            schema: CANDIDATE_ATTEMPT_RESULT_SCHEMA.to_string(),
            disposition,
            receipt,
            stdout_blake3: blake3_identity(stdout),
            stderr_blake3: blake3_identity(stderr),
            blocker,
        }
    }

    pub fn validate(&self) -> Result<(), CandidateWorkerError> {
        if self.schema != CANDIDATE_ATTEMPT_RESULT_SCHEMA {
            return Err(CandidateWorkerError::Result("unsupported candidate result schema".to_string()));
        }
        if !is_blake3(&self.stdout_blake3) || !is_blake3(&self.stderr_blake3) {
            return Err(CandidateWorkerError::Result("candidate output identity is malformed".to_string()));
        }
        match self.disposition {
            CandidateDisposition::Produced => {
                let receipt = self
                    .receipt
                    .as_ref()
                    .ok_or_else(|| CandidateWorkerError::Result("produced result is missing a receipt".to_string()))?;
                receipt.validate()?;
                if self.blocker.is_some() {
                    return Err(CandidateWorkerError::Result("produced result carries a blocker".to_string()));
                }
            }
            CandidateDisposition::Blocked | CandidateDisposition::Failed | CandidateDisposition::Unknown => {
                if self.receipt.is_some() {
                    return Err(CandidateWorkerError::Result("non-produced result carries a receipt".to_string()));
                }
                if self.blocker.is_none() {
                    return Err(CandidateWorkerError::Result("non-produced result is missing a blocker".to_string()));
                }
            }
            CandidateDisposition::Cancelled => {
                if self.receipt.is_some() {
                    return Err(CandidateWorkerError::Result("cancelled result carries a receipt".to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, CandidateWorkerError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| CandidateWorkerError::Result(error.to_string()))
    }

    pub fn from_json(text: &str) -> Result<Self, CandidateWorkerError> {
        let result: Self =
            serde_json::from_str(text).map_err(|error| CandidateWorkerError::Result(error.to_string()))?;
        result.validate()?;
        Ok(result)
    }
}

fn is_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Error)]
pub enum CandidateWorkerError {
    #[error("workspace capture failed: {0}")]
    Capture(#[from] CandidateCaptureError),
    #[error("candidate receipt derivation failed: {0}")]
    Receipt(#[from] CandidateReceiptError),
    #[error("workspace root is not valid UTF-8")]
    NonUtf8Root,
    #[error("candidate attempt result is invalid: {0}")]
    Result(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::CandidateDisposition;
    use crate::candidate_capture::admit_workspace_boundary;

    fn boundary(root: &Path) -> WorkspaceBoundary {
        admit_workspace_boundary(root, &["src".to_string()], &["src".to_string()], false, false)
            .expect("boundary admits")
    }

    fn request(root: &Path, command: &str) -> CandidateWorkerRequest {
        CandidateWorkerRequest {
            work_id: "work-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            fence: "7".to_string(),
            workspace_id: "workspace-1".to_string(),
            program: "sh".to_string(),
            args: vec!["-c".to_string(), command.to_string()],
            boundary: boundary(root),
            timeout_secs: 5,
            output_limit_bytes: 4_096,
        }
    }

    #[test]
    fn worker_produces_host_derived_candidate_receipt() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"old").expect("base source");

        let outcome = run_candidate_worker(
            &request(root, "printf new > src/main.rs"),
            &RuntimeConfig::default(),
            &AtomicBool::new(false),
        )
        .expect("worker runs");

        let CandidateWorkerOutcome::Produced { receipt, .. } = outcome else {
            panic!("expected produced candidate, got {outcome:?}");
        };
        assert_eq!(receipt.disposition, CandidateDisposition::Produced);
        assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
        assert!(receipt.is_usable_candidate());
    }

    #[test]
    fn exit_zero_without_change_is_not_a_candidate() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let outcome = run_candidate_worker(&request(root, "true"), &RuntimeConfig::default(), &AtomicBool::new(false))
            .expect("worker runs");

        assert!(matches!(outcome, CandidateWorkerOutcome::Failed { .. }));
    }

    #[test]
    fn non_zero_exit_is_a_failure() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let outcome =
            run_candidate_worker(&request(root, "exit 3"), &RuntimeConfig::default(), &AtomicBool::new(false))
                .expect("worker runs");

        assert!(matches!(outcome, CandidateWorkerOutcome::Failed { .. }));
    }

    #[test]
    fn output_overflow_and_timeout_are_failures() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let mut overflow = request(root, "printf 1234567890");
        overflow.output_limit_bytes = 8;
        let outcome =
            run_candidate_worker(&overflow, &RuntimeConfig::default(), &AtomicBool::new(false)).expect("worker runs");
        assert!(matches!(outcome, CandidateWorkerOutcome::Failed { .. }));

        let mut timed = request(root, "sleep 2");
        timed.timeout_secs = 1;
        let outcome =
            run_candidate_worker(&timed, &RuntimeConfig::default(), &AtomicBool::new(false)).expect("worker runs");
        assert!(matches!(outcome, CandidateWorkerOutcome::Failed { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn interrupted_candidate_capture_is_unknown() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let outcome = run_candidate_worker(
            &request(root, "/run/current-system/sw/bin/chmod 000 src/main.rs"),
            &RuntimeConfig::default(),
            &AtomicBool::new(false),
        )
        .expect("worker runs");

        assert!(matches!(outcome, CandidateWorkerOutcome::Unknown { .. }), "{outcome:?}");
        std::fs::set_permissions(root.join("src/main.rs"), std::fs::Permissions::from_mode(0o644))
            .expect("restore source");
    }

    #[test]
    fn produced_candidate_is_published_to_store() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path().join("workspace");
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
        let store = Store::new(directory.path().join("store.redb")).expect("store opens");

        let outcome = run_and_publish_candidate_worker(
            &request(&root, "printf new > src/main.rs"),
            &RuntimeConfig::default(),
            &AtomicBool::new(false),
            &store,
        )
        .expect("worker runs");

        assert!(matches!(outcome, CandidateWorkerOutcome::Produced { .. }), "{outcome:?}");
        let receipt = store.load_candidate_receipt("attempt-1").expect("receipt loads").expect("receipt exists");
        assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
        assert_eq!(
            store.load_candidate_artifact("attempt-1", "src/main.rs").expect("artifact loads"),
            Some(b"new".to_vec())
        );
        assert_eq!(store.list_candidate_artifacts("attempt-1").expect("artifacts list").len(), 1);
    }

    #[test]
    fn attempt_result_hashes_outputs_and_round_trips() {
        let receipt = CandidateReceipt {
            schema: crate::candidate::CANDIDATE_RECEIPT_SCHEMA.to_string(),
            work_id: "work-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            fence: "7".to_string(),
            workspace_id: "workspace-1".to_string(),
            base_identity: "a".repeat(64),
            candidate_identity: Some("b".repeat(64)),
            diff_identity: Some("c".repeat(64)),
            changed_paths: vec!["src/main.rs".to_string()],
            disposition: CandidateDisposition::Produced,
            blockers: Vec::new(),
        };
        let outcome = CandidateWorkerOutcome::Produced {
            receipt: receipt.clone(),
            changed_files: BTreeMap::new(),
            stdout: b"stdout".to_vec(),
            stderr: b"stderr".to_vec(),
        };
        let result = CandidateAttemptResult::from_outcome(&outcome);
        assert_eq!(result.stdout_blake3, blake3_identity(b"stdout"));
        assert_eq!(result.stderr_blake3, blake3_identity(b"stderr"));
        let json = result.to_json().expect("result serializes");
        let parsed = CandidateAttemptResult::from_json(&json).expect("result parses");
        assert_eq!(parsed, result);

        let mut invalid = CandidateAttemptResult::from_outcome(&CandidateWorkerOutcome::Blocked {
            blocker: "blocked".to_string(),
        });
        invalid.receipt = Some(receipt);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn invalid_limits_are_blocked_before_execution() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"old").expect("source");

        let mut no_deadline = request(root, "printf new > src/main.rs");
        no_deadline.timeout_secs = 0;
        let outcome = run_candidate_worker(&no_deadline, &RuntimeConfig::default(), &AtomicBool::new(false))
            .expect("worker runs");
        assert!(matches!(outcome, CandidateWorkerOutcome::Blocked { .. }));

        let mut no_output = request(root, "printf new > src/main.rs");
        no_output.output_limit_bytes = 0;
        let outcome =
            run_candidate_worker(&no_output, &RuntimeConfig::default(), &AtomicBool::new(false)).expect("worker runs");
        assert!(matches!(outcome, CandidateWorkerOutcome::Blocked { .. }));
        assert_eq!(std::fs::read(root.join("src/main.rs")).expect("source remains"), b"old");
    }

    #[test]
    fn worker_environment_is_clear() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let outcome = run_candidate_worker(
            &request(root, "printf %s \"$HOME\" > src/home"),
            &RuntimeConfig::default(),
            &AtomicBool::new(false),
        )
        .expect("worker runs");

        assert!(matches!(outcome, CandidateWorkerOutcome::Produced { .. }), "{outcome:?}");
        assert_eq!(std::fs::read(root.join("src/home")).expect("environment probe"), b"");
    }

    #[test]
    fn in_flight_cancellation_stops_the_worker_before_late_write() {
        use std::sync::Arc;
        use std::sync::atomic::Ordering;
        use std::time::Duration;

        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"same").expect("source");

        let cancellation = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancellation);
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            signal.store(true, Ordering::Release);
        });
        let outcome = run_candidate_worker(
            &request(root, "/run/current-system/sw/bin/sleep 1; printf leaked > src/leak"),
            &RuntimeConfig::default(),
            cancellation.as_ref(),
        )
        .expect("worker runs");
        canceller.join().expect("canceller joins");

        assert!(matches!(outcome, CandidateWorkerOutcome::Cancelled), "{outcome:?}");
        assert!(!root.join("src/leak").exists());
    }
}
