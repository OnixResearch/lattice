use super::*;

/// Executes a replay script against a target model.
#[must_use]
pub fn run_replay<T: ReplayTarget>(script: &ReplayScript, target: &mut T) -> ReplayReport {
    let mut issues = validate_script(script);

    if issues.is_empty() {
        for (event_index, event) in script.events.iter().enumerate() {
            let event_step = event_index.saturating_add(1);
            let mut event_issues = validate_event_against_target(event, target, event_step);
            event_issues.extend(target.validate_event(event));
            if event_issues.is_empty() {
                target.apply_event(event);
            } else {
                issues.extend(event_issues);
            }
        }
    }

    let state = target.snapshot_state();
    issues.extend(validate_assertions(&script.assertions, &state));
    let state_hash = state_snapshot_hash(&state);
    debug_assert!(!state_hash.is_empty());
    let snapshots = snapshot_hashes(script, target, &state_hash);
    debug_assert!(snapshots.iter().any(|snapshot| snapshot.label == "state"));
    issues.extend(validate_snapshots(&script.snapshots, &snapshots));

    ReplayReport {
        state,
        state_hash,
        snapshots,
        issues,
    }
}

/// Computes the BLAKE3 hash of snapshot bytes.
#[must_use]
pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn validate_script(script: &ReplayScript) -> Vec<ReplayIssue> {
    let issue_bound = script.events.len().saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(issues.capacity() >= issue_bound);
    debug_assert!(script.metadata.name.trim().len() <= script.metadata.name.len());
    if script.metadata.name.trim().is_empty() {
        issues.push(ReplayIssue::new(
            ReplayIssueCode::MissingMetadata,
            FIELD_METADATA_NAME,
            "replay metadata name is required",
        ));
    }
    for event in &script.events {
        match event {
            ReplayEvent::Unsupported { kind } => issues.push(ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_EVENT_KIND,
                format!("unsupported replay event kind `{kind}`"),
            )),
            ReplayEvent::Text { value, .. } if value.is_empty() => issues.push(ReplayIssue::new(
                ReplayIssueCode::EmptyText,
                FIELD_EVENT_TEXT,
                "text replay event must not be empty",
            )),
            _ => {}
        }
    }
    issues
}

fn validate_event_against_target<T: ReplayTarget>(
    event: &ReplayEvent,
    target: &T,
    event_step: usize,
) -> Vec<ReplayIssue> {
    match event {
        ReplayEvent::Key {
            target: event_target, ..
        }
        | ReplayEvent::Text {
            target: event_target, ..
        } => validate_focused_target(event_target, target, event_step),
        ReplayEvent::Mouse {
            target: event_target,
            x,
            y,
        } => {
            let mut issues = validate_focused_target(event_target, target, event_step);
            let (width, height) = target.bounds();
            if *x >= width || *y >= height {
                issues.push(ReplayIssue::new(
                    ReplayIssueCode::OutOfBoundsMouse,
                    event_field(event_step, FIELD_MOUSE_BOUNDS),
                    format!("step {event_step}: mouse coordinates ({x}, {y}) exceed bounds {width}x{height}"),
                ));
            }
            issues
        }
        ReplayEvent::Command { action_id } => {
            let surface = target.focused_target().unwrap_or_else(|| "unfocused".to_string());
            if !target.action_known(action_id) {
                vec![ReplayIssue::new(
                    ReplayIssueCode::UnknownCommand,
                    event_field(event_step, FIELD_ACTION),
                    format!("step {event_step}: action `{action_id}` is unknown for surface `{surface}`"),
                )]
            } else if target.action_enabled(action_id) {
                Vec::new()
            } else {
                vec![ReplayIssue::new(
                    ReplayIssueCode::DisabledAction,
                    event_field(event_step, FIELD_ACTION),
                    format!(
                        "step {event_step}: action `{action_id}` is disabled for surface `{surface}` (disabled state)"
                    ),
                )]
            }
        }
        ReplayEvent::Wait { condition } => validate_wait_condition(condition, target, event_step),
        ReplayEvent::Resize { .. } | ReplayEvent::Unsupported { .. } => Vec::new(),
    }
}

fn validate_focused_target<T: ReplayTarget>(event_target: &str, target: &T, event_step: usize) -> Vec<ReplayIssue> {
    if !target.focus_targets().contains(event_target) {
        return vec![ReplayIssue::new(
            ReplayIssueCode::StaleFocus,
            event_field(event_step, FIELD_EVENT_TARGET),
            format!("step {event_step}: target `{event_target}` is not focusable"),
        )];
    }
    if target.focused_target().as_deref() == Some(event_target) {
        Vec::new()
    } else {
        vec![ReplayIssue::new(
            ReplayIssueCode::StaleFocus,
            event_field(event_step, FIELD_EVENT_TARGET),
            format!("step {event_step}: target `{event_target}` does not own focus"),
        )]
    }
}

fn validate_wait_condition<T: ReplayTarget>(
    condition: &ReplayWaitCondition,
    target: &T,
    event_step: usize,
) -> Vec<ReplayIssue> {
    if condition.evaluation_budget == 0 {
        return vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            "wait condition exhausted an empty evaluation budget",
        )];
    }
    match &condition.predicate {
        ReplayWaitPredicate::StateEquals { key, expected } => validate_state_equals(target, event_step, key, expected),
        ReplayWaitPredicate::StateContains { key, needle } => validate_state_contains(target, event_step, key, needle),
        ReplayWaitPredicate::SnapshotContains { label, needle } => {
            validate_snapshot_contains(target, event_step, label, needle)
        }
        ReplayWaitPredicate::FocusTarget { target: focus_target } => {
            validate_wait_focus_target(target, event_step, focus_target)
        }
        ReplayWaitPredicate::DiagnosticPresent { code } => validate_diagnostic_present(target, event_step, code),
        ReplayWaitPredicate::Unsupported { predicate } => vec![ReplayIssue::new(
            ReplayIssueCode::UnsupportedWaitPredicate,
            event_field(event_step, FIELD_WAIT),
            format!("unsupported wait predicate `{predicate}`"),
        )],
    }
}

fn validate_state_equals<T: ReplayTarget>(
    target: &T,
    event_step: usize,
    key: &str,
    expected: &str,
) -> Vec<ReplayIssue> {
    let state = target.snapshot_state();
    let Some(actual) = state.get(key) else {
        return vec![missing_wait_target(event_step, key)];
    };
    if actual == expected {
        Vec::new()
    } else {
        vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            format!("wait expected `{key}` to equal `{expected}` but found `{actual}`"),
        )]
    }
}

fn validate_state_contains<T: ReplayTarget>(
    target: &T,
    event_step: usize,
    key: &str,
    needle: &str,
) -> Vec<ReplayIssue> {
    let state = target.snapshot_state();
    let Some(actual) = state.get(key) else {
        return vec![missing_wait_target(event_step, key)];
    };
    if actual.contains(needle) {
        Vec::new()
    } else {
        vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            format!("wait expected `{key}` to contain `{needle}` but found `{actual}`"),
        )]
    }
}

fn validate_snapshot_contains<T: ReplayTarget>(
    target: &T,
    event_step: usize,
    label: &str,
    needle: &str,
) -> Vec<ReplayIssue> {
    let Some(bytes) = target.snapshot_bytes(label) else {
        return vec![missing_wait_target(event_step, label)];
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            format!("wait snapshot `{label}` was not valid UTF-8"),
        )];
    };
    if text.contains(needle) {
        Vec::new()
    } else {
        vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            format!("wait expected snapshot `{label}` to contain `{needle}`"),
        )]
    }
}

fn validate_wait_focus_target<T: ReplayTarget>(target: &T, event_step: usize, focus_target: &str) -> Vec<ReplayIssue> {
    if !target.focus_targets().contains(focus_target) {
        return vec![missing_wait_target(event_step, focus_target)];
    }
    if target.focused_target().as_deref() == Some(focus_target) {
        Vec::new()
    } else {
        vec![ReplayIssue::new(
            ReplayIssueCode::WaitConditionUnmet,
            event_field(event_step, FIELD_WAIT),
            format!("wait expected focus target `{focus_target}` to own focus"),
        )]
    }
}

fn validate_diagnostic_present<T: ReplayTarget>(target: &T, event_step: usize, code: &str) -> Vec<ReplayIssue> {
    if target.diagnostic_events().contains(code) {
        Vec::new()
    } else {
        vec![missing_wait_target(event_step, code)]
    }
}

fn missing_wait_target(event_step: usize, target: &str) -> ReplayIssue {
    ReplayIssue::new(
        ReplayIssueCode::MissingWaitTarget,
        event_field(event_step, FIELD_WAIT),
        format!("wait target `{target}` was not supplied"),
    )
}

fn validate_assertions(
    assertions: &[ReplayAssertion],
    state: &std::collections::BTreeMap<String, String>,
) -> Vec<ReplayIssue> {
    let mut issues = Vec::with_capacity(assertions.len());
    for assertion in assertions {
        if state.get(&assertion.key) != Some(&assertion.expected) {
            issues.push(ReplayIssue::new(
                ReplayIssueCode::AssertionMismatch,
                FIELD_ASSERTION,
                format!("expected `{}` to be `{}`", assertion.key, assertion.expected),
            ));
        }
    }
    issues
}

fn snapshot_hashes<T: ReplayTarget>(
    script: &ReplayScript,
    target: &T,
    state_hash: &str,
) -> Vec<ReplaySnapshotExpectation> {
    let snapshot_bound = script.snapshots.len().saturating_add(1);
    let mut snapshots = Vec::with_capacity(snapshot_bound);
    snapshots.push(ReplaySnapshotExpectation::new("state", state_hash));
    for expectation in &script.snapshots {
        if expectation.label == "state" {
            continue;
        }
        if let Some(bytes) = target.snapshot_bytes(&expectation.label) {
            snapshots.push(ReplaySnapshotExpectation::new(expectation.label.clone(), blake3_hex(&bytes)));
        }
    }
    snapshots
}

fn validate_snapshots(
    expected: &[ReplaySnapshotExpectation],
    actual: &[ReplaySnapshotExpectation],
) -> Vec<ReplayIssue> {
    let mut issues = Vec::with_capacity(expected.len());
    for expectation in expected {
        let actual_hash = actual
            .iter()
            .find(|snapshot| snapshot.label == expectation.label)
            .map(|snapshot| snapshot.blake3.as_str());
        if actual_hash != Some(expectation.blake3.as_str()) {
            issues.push(ReplayIssue::new(
                ReplayIssueCode::SnapshotMismatch,
                expectation.label.as_str(),
                format!("snapshot `{}` hash did not match", expectation.label),
            ));
        }
    }
    issues
}

/// Computes the deterministic BLAKE3 hash for state snapshot key/value pairs.
#[must_use]
pub fn state_snapshot_hash(state: &std::collections::BTreeMap<String, String>) -> String {
    let byte_capacity_bytes = state
        .iter()
        .map(|(key, value)| key.len().saturating_add(value.len()).saturating_add(STATE_ENTRY_SEPARATOR_BYTES))
        .sum();
    let mut bytes = Vec::with_capacity(byte_capacity_bytes);
    for (key, value) in state {
        bytes.extend_from_slice(key.as_bytes());
        bytes.push(b'=');
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    blake3_hex(&bytes)
}

fn event_field(event_step: usize, field: &str) -> String {
    format!("event[{event_step}].{field}")
}
