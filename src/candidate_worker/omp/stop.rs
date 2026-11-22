//! How a failed worker process stopped, read from the launcher's own
//! blocker texts (`candidate_worker::execution` and `completion`), so a
//! caller never matches free-form strings itself.

/// How a failed worker process ended. Termination was observed on every
/// path; none of them is a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    /// The process exited on its own with a non-zero status, or without a
    /// status after a signal (`None`).
    Exited { status_code: Option<i32> },
    /// The host ended the process at its admitted deadline.
    Deadline,
    /// The host ended the process at its admitted output limit.
    OutputLimit,
    /// The process exited, but changed a path outside its write boundary.
    WriteBoundary,
}

/// The termination of a `Failed` outcome; `None` for any other outcome or
/// a blocker the launcher never writes.
#[must_use]
pub fn of(outcome: &super::super::Outcome) -> Option<Termination> {
    let super::super::Outcome::Failed { blocker, .. } = outcome else {
        return None;
    };
    match blocker.as_str() {
        super::super::execution::DEADLINE_BLOCKER => Some(Termination::Deadline),
        super::super::execution::OUTPUT_LIMIT_BLOCKER => Some(Termination::OutputLimit),
        super::super::completion::WRITE_BOUNDARY_BLOCKER => Some(Termination::WriteBoundary),
        other => exit_status(other.strip_prefix(super::super::execution::EXIT_BLOCKER_PREFIX)?),
    }
}

/// The status of a `{status:?}` suffix: `None` or `Some(code)`.
fn exit_status(debug: &str) -> Option<Termination> {
    if debug == "None" {
        return Some(Termination::Exited { status_code: None });
    }
    let code = debug.strip_prefix("Some(")?.strip_suffix(')')?.parse().ok()?;
    Some(Termination::Exited {
        status_code: Some(code),
    })
}

#[cfg(test)]
mod tests {
    fn failed(blocker: &str) -> super::super::super::Outcome {
        super::super::super::Outcome::Failed {
            blocker: blocker.to_owned(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn every_launcher_failure_text_has_its_termination_and_nothing_else_does() {
        use super::Termination;
        let exited = format!("{}{:?}", super::super::super::execution::EXIT_BLOCKER_PREFIX, Some(3));
        let signalled = format!("{}{:?}", super::super::super::execution::EXIT_BLOCKER_PREFIX, None::<i32>);
        let cases = [
            (exited.as_str(), Some(Termination::Exited { status_code: Some(3) })),
            (signalled.as_str(), Some(Termination::Exited { status_code: None })),
            (super::super::super::execution::DEADLINE_BLOCKER, Some(Termination::Deadline)),
            (super::super::super::execution::OUTPUT_LIMIT_BLOCKER, Some(Termination::OutputLimit)),
            (super::super::super::completion::WRITE_BOUNDARY_BLOCKER, Some(Termination::WriteBoundary)),
            ("worker exited with status Some(x)", None),
            ("anything else", None),
        ];
        for (blocker, expected) in cases {
            assert_eq!(super::of(&failed(blocker)), expected, "{blocker}");
        }
        let blocked = super::super::super::Outcome::Blocked {
            blocker: super::super::super::execution::DEADLINE_BLOCKER.to_owned(),
        };
        assert_eq!(super::of(&blocked), None, "only a failed outcome has a termination");
    }
}
