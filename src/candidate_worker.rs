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
use crate::candidate::CandidateReceipt;
use crate::candidate::CandidateReceiptError;
use crate::candidate::produced_receipt_from_snapshots;
use crate::candidate_capture::CandidateCaptureError;
use crate::candidate_capture::WorkspaceBoundary;
use crate::candidate_capture::capture_workspace;
use crate::process::core::RuntimeConfig;
use crate::process::shell::run_bounded_clear_environment_with_cancellation;

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
    let candidate = capture_workspace(root)?;
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
        Err(error) => return Err(CandidateWorkerError::Receipt(error)),
    };
    Ok(CandidateWorkerOutcome::Produced {
        receipt,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

#[derive(Debug, Error)]
pub enum CandidateWorkerError {
    #[error("workspace capture failed: {0}")]
    Capture(#[from] CandidateCaptureError),
    #[error("candidate receipt derivation failed: {0}")]
    Receipt(#[from] CandidateReceiptError),
    #[error("workspace root is not valid UTF-8")]
    NonUtf8Root,
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
}
