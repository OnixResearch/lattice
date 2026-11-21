//! Per-work fence records in one attempt state root.
//!
//! `works/<blake3(work_id)>/<fence>` names the one attempt admitted at that
//! fence of that work. A run records it under an exclusive lock before the
//! execution binding and intent, so a lower fence or a second attempt at one
//! fence is refused before any worker effect. Kiln remains the fence
//! authority; these records only stop this state root from executing a
//! superseded or crossed request. Recovery reads them and never writes them.

// r[impl lattice.implementation_attempt.consumer.scenario.refusal]

const DIRECTORY: &str = "works";
const FENCE_DIGITS: usize = 20;
const MAX_FENCES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Decision {
    /// No attempt is recorded at this fence; record this one.
    Record,
    /// This exact attempt is already recorded at this fence.
    Replay,
}

/// Decide one run from the highest recorded fence and the attempt recorded at
/// this request's fence.
pub(super) fn decide(
    highest: Option<u64>,
    recorded_at_fence: Option<&str>,
    fence: u64,
    attempt_id: &str,
) -> Result<Decision, super::refusal::Reason> {
    if highest.is_some_and(|recorded| recorded > fence) {
        return Err(super::refusal::Reason::StaleFence);
    }
    match recorded_at_fence {
        None => Ok(Decision::Record),
        Some(recorded) if recorded == attempt_id => Ok(Decision::Replay),
        Some(_) => Err(super::refusal::Reason::CrossedAttempt),
    }
}

fn work_directory(state: &std::path::Path, work_id: &str) -> std::path::PathBuf {
    state.join(DIRECTORY).join(blake3::hash(work_id.as_bytes()).to_hex().as_str())
}

fn fence_name(fence: u64) -> String {
    format!("{fence:0FENCE_DIGITS$}")
}

fn recorded_at(directory: &std::path::Path, fence: u64) -> Result<Option<String>, String> {
    let bytes = super::artifacts::read_optional(
        &directory.join(fence_name(fence)),
        crate::implementation_attempt::MAX_IDENTIFIER_BYTES,
    )?;
    bytes
        .map(|value| String::from_utf8(value).map_err(|_| "retained fence record is malformed".to_string()))
        .transpose()
}

fn highest_fence(directory: &std::path::Path) -> Result<Option<u64>, String> {
    let entries = std::fs::read_dir(directory).map_err(|_| "retained fence records are unreadable")?;
    let mut highest = None;
    let mut count: usize = 0;
    for entry in entries {
        count = count.saturating_add(1);
        if count > MAX_FENCES {
            return Err("retained fence records exceed their bound".into());
        }
        let name = entry.map_err(|_| "retained fence records are unreadable")?.file_name();
        let name = name.to_str().ok_or("retained fence record name is malformed")?;
        if name.len() != FENCE_DIGITS || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("retained fence record name is malformed".into());
        }
        let fence = name.parse::<u64>().map_err(|_| "retained fence record name is malformed")?;
        highest = highest.max(Some(fence));
    }
    Ok(highest)
}

/// Record this attempt at its fence, or refuse a stale or crossed request.
pub(super) fn admit(
    state: &std::path::Path,
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
) -> Result<(), String> {
    super::artifacts::private_directory(&state.join(DIRECTORY))?;
    let directory = work_directory(state, &envelope.work_id);
    super::artifacts::private_directory(&directory)?;
    let lock = std::fs::File::open(&directory).map_err(|_| "fence records cannot be opened")?;
    lock.lock().map_err(|_| "fence records cannot be locked")?;
    let recorded = recorded_at(&directory, envelope.fence)?;
    let decision = decide(highest_fence(&directory)?, recorded.as_deref(), envelope.fence, &envelope.attempt_id)
        .map_err(|reason| super::refusal::refuse(envelope, reason))?;
    if decision == Decision::Record
        && !super::artifacts::create_new(&directory.join(fence_name(envelope.fence)), envelope.attempt_id.as_bytes())?
    {
        return Err("fence record appeared while locked".into());
    }
    drop(lock);
    Ok(())
}

/// Read-only: refuse when another attempt is recorded at this fence.
pub(super) fn check(
    state: &std::path::Path,
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
) -> Result<(), String> {
    let recorded = recorded_at(&work_directory(state, &envelope.work_id), envelope.fence)?;
    if recorded.is_some_and(|attempt| attempt != envelope.attempt_id) {
        return Err(super::refusal::refuse(envelope, super::refusal::Reason::CrossedAttempt));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::refusal::Reason;
    use super::Decision;
    use super::decide;

    #[test]
    // r[verify lattice.implementation_attempt.consumer.scenario.refusal]
    fn fences_admit_first_and_exact_replay_but_refuse_stale_and_crossed() {
        assert_eq!(decide(None, None, 7, "a"), Ok(Decision::Record));
        assert_eq!(decide(Some(6), None, 7, "a"), Ok(Decision::Record));
        assert_eq!(decide(Some(7), Some("a"), 7, "a"), Ok(Decision::Replay));
        assert_eq!(decide(Some(8), None, 7, "a"), Err(Reason::StaleFence));
        assert_eq!(decide(Some(8), Some("a"), 7, "a"), Err(Reason::StaleFence));
        assert_eq!(decide(Some(7), Some("b"), 7, "a"), Err(Reason::CrossedAttempt));
    }

    #[test]
    fn records_are_bounded_named_and_checked_in_a_state_root() {
        let root = tempfile::tempdir().expect("state");
        let state = root.path().canonicalize().expect("canonical state");
        let mut envelope = super::super::tests::host_local().envelope;
        super::admit(&state, &envelope).expect("first record");
        super::admit(&state, &envelope).expect("exact replay");
        super::check(&state, &envelope).expect("recovery of the recorded attempt");
        let mut crossed = envelope.clone();
        crossed.attempt_id = "attempt-crossed".into();
        assert_eq!(super::admit(&state, &crossed), Err(Reason::CrossedAttempt.message().to_owned()));
        assert_eq!(super::check(&state, &crossed), Err(Reason::CrossedAttempt.message().to_owned()));
        envelope.fence += 1;
        envelope.attempt_id = "attempt-successor".into();
        super::admit(&state, &envelope).expect("successor fence");
        let mut stale = envelope.clone();
        stale.fence -= 1;
        stale.attempt_id = "attempt-late".into();
        assert_eq!(super::admit(&state, &stale), Err(Reason::StaleFence.message().to_owned()));
        let directory = super::work_directory(&state, &envelope.work_id);
        std::fs::write(directory.join("not-a-fence"), b"x").expect("foreign entry");
        assert!(super::admit(&state, &envelope).is_err(), "unexpected entries fail closed");
    }
}
