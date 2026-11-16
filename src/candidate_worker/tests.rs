use super::*;
use crate::candidate_capture::admit_workspace_boundary;

pub(super) fn boundary(root: &std::path::Path) -> crate::candidate_capture::WorkspaceBoundary {
    admit_workspace_boundary(root, &["src".to_string()], &["src".to_string()], false, false).expect("boundary admits")
}

pub(super) fn host_tool(name: &str) -> String {
    which::which(name).expect("host test tool").to_string_lossy().into_owned()
}

pub(super) fn request(root: &std::path::Path, command: &str) -> Request {
    Request {
        work_id: "work-1".to_string(),
        attempt_id: "attempt-1".to_string(),
        fence: "7".to_string(),
        workspace_id: "workspace-1".to_string(),
        program: "sh".to_string(),
        args: vec!["-c".to_string(), command.to_string()],
        boundary: boundary(root),
        require_isolation: false,
        sandbox_program: None,
        broker: None,
        timeout_secs: 5,
        output_limit_bytes: 4_096,
    }
}

mod admission;
mod cancellation;
mod execution;
mod interrupted;
mod isolation;
mod publication;
mod result;
