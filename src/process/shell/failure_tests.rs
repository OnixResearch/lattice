use bounded_exec::Operation;
use bounded_exec::RunError;
use bounded_exec::Worker;

use super::map_execution_error;
use crate::agent_node::ProcessError;
use crate::agent_node::ProcessFailurePhase as Phase;
use crate::agent_node::ProcessTermination as Termination;

fn io(operation: Operation) -> RunError {
    RunError::Io {
        operation,
        error: std::io::Error::other("private path /secret/token"),
    }
}

#[test]
// r[verify lattice.candidate_worker.process_observation.scenario.cleanup]
fn runner_error_matrix_preserves_phase_and_observed_cleanup() {
    for error in [
        RunError::EmptyProgram,
        RunError::ProgramNotAbsolute,
        RunError::InvalidLimits(bounded_exec::LimitError::TimeoutZero),
        RunError::EmptyCurrentDirectory,
        RunError::CurrentDirectoryNotAbsolute,
        RunError::DuplicateEnvironmentName,
        RunError::InputLimitExceeded,
    ] {
        assert!(matches!(map_execution_error(error), ProcessError::InvalidRequest(_)));
    }
    assert!(matches!(map_execution_error(io(Operation::Spawn)), ProcessError::Spawn(_)));
    let cases = [
        (io(Operation::Wait), Phase::Execution, Termination::Observed),
        (io(Operation::Kill), Phase::Teardown, Termination::Unknown),
        (io(Operation::Reap), Phase::Teardown, Termination::Unknown),
        (io(Operation::ReadStdout), Phase::Capture, Termination::Observed),
        (io(Operation::ReadStderr), Phase::Capture, Termination::Observed),
        (io(Operation::WriteStdin), Phase::Capture, Termination::Observed),
        (RunError::MissingPipe(Worker::Stdout), Phase::Capture, Termination::Observed),
        (RunError::CaptureCountOverflow(Worker::Stderr), Phase::Capture, Termination::Observed),
        (RunError::WorkerDisconnected(Worker::Stdout), Phase::Capture, Termination::Observed),
        (RunError::WorkerPanicked(Worker::Stderr), Phase::Capture, Termination::Observed),
        (RunError::WorkerTeardownTimedOut(Worker::Stdout), Phase::Teardown, Termination::Observed),
        (RunError::ChildTeardownTimedOut, Phase::Teardown, Termination::Unknown),
        (RunError::ProcessIdentifierOverflow, Phase::Teardown, Termination::Unknown),
    ];
    for (error, phase, termination) in cases {
        let mapped = map_execution_error(error);
        assert_eq!(mapped, ProcessError::Execution { phase, termination });
        let diagnostic = mapped.to_string();
        assert!(!diagnostic.contains("/secret"));
        assert!(diagnostic.len() < 256);
        assert!(!diagnostic.contains("spawn failed"));
    }
}
