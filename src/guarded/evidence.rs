//! Redaction-safe guarded progress summaries for orchestration status and replay.

const MERGE_NODE_LABEL: &str = "Collect progress facts";
const CAIRN_DISCOVERY_NODE_LABEL: &str = "Discover active changes";
const OCTET_BASELINE_NODE_LABEL: &str = "Probe baseline findings";
const VALIDATION_NODE_LABEL: &str = "Validate repository";
const STDOUT_PORT_LABEL: &str = "stdout";
const MERGED_PORT_LABEL: &str = "merged";
const SELECTED_FINGERPRINT_DOMAIN: &[u8] = b"lattice-guarded-selected-change:v1";
const ABSENT_FACT: &str = "none";
const UNKNOWN_FACT: &str = "unknown";

const MERGE_OUTPUT: OutputSelector = OutputSelector {
    node_label: MERGE_NODE_LABEL,
    port_label: MERGED_PORT_LABEL,
};
const CAIRN_DISCOVERY_OUTPUT: OutputSelector = OutputSelector {
    node_label: CAIRN_DISCOVERY_NODE_LABEL,
    port_label: STDOUT_PORT_LABEL,
};
const OCTET_BASELINE_OUTPUT: OutputSelector = OutputSelector {
    node_label: OCTET_BASELINE_NODE_LABEL,
    port_label: STDOUT_PORT_LABEL,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Summary {
    pub bundle: String,
    pub selected_work_fingerprint: Option<String>,
    pub validation: String,
    pub before: Option<u64>,
    pub after: Option<u64>,
    pub decision: String,
    pub restoration: String,
    pub next_action: String,
}

impl Summary {
    pub fn render(&self) -> String {
        format!(
            "guarded bundle={} selected={} validation={} before={} after={} decision={} restoration={} next={}",
            self.bundle,
            self.selected_work_fingerprint.as_deref().unwrap_or(ABSENT_FACT),
            self.validation,
            optional_metric(self.before),
            optional_metric(self.after),
            self.decision,
            self.restoration,
            self.next_action,
        )
    }
}

struct MergedSource<'a> {
    definition_id: &'a str,
    merged: &'a str,
}

#[derive(Clone, Copy)]
struct OutputSelector {
    node_label: &'static str,
    port_label: &'static str,
}

// r[impl lattice.guarded_agent_templates.evidence]
pub fn summarize(definition_id: &str, details: &crate::store::RunDetailRecord) -> Option<Summary> {
    let kind = match definition_id {
        super::authority::CAIRN_BUNDLE_ID => super::Kind::Cairn,
        super::authority::OCTET_BUNDLE_ID => super::Kind::Octet,
        _ => return None,
    };
    if let Some(merged) = node_output(details, MERGE_OUTPUT) {
        return merged_summary(kind, MergedSource { definition_id, merged });
    }
    Some(partial_summary(kind, definition_id, details))
}

fn merged_summary(kind: super::Kind, source: MergedSource<'_>) -> Option<Summary> {
    let records: Vec<MergeObservation> = serde_json::from_str(source.merged).ok()?;
    let before = merge_value(&records, "before")?;
    let after = merge_value(&records, "after")?;
    let is_validation_passed = merge_value_json(&records, "validation")?.as_i64() == Some(0);
    debug_assert!(!before.is_empty());
    debug_assert!(!after.is_empty());
    let decision =
        super::shell::decide_from_merged_json(kind, super::core::DEFAULT_MINIMUM_DELTA, source.merged.as_bytes())
            .ok()?;
    match kind {
        super::Kind::Cairn => {
            let before: super::core::CairnSnapshot = serde_json::from_str(before).ok()?;
            let after: super::core::CairnSnapshot = serde_json::from_str(after).ok()?;
            Some(cairn_summary(source.definition_id, &before, &after, is_validation_passed, &decision))
        }
        super::Kind::Octet => {
            let before: super::core::OctetSnapshot = serde_json::from_str(before).ok()?;
            let after: super::core::OctetSnapshot = serde_json::from_str(after).ok()?;
            Some(octet_summary(source.definition_id, &before, &after, is_validation_passed, &decision))
        }
    }
}

fn cairn_summary(
    definition_id: &str,
    before: &super::core::CairnSnapshot,
    after: &super::core::CairnSnapshot,
    is_validation_passed: bool,
    decision: &super::core::ProgressDecision,
) -> Summary {
    Summary {
        bundle: definition_id.to_owned(),
        selected_work_fingerprint: before.selected.as_deref().map(selected_fingerprint),
        validation: validation_label(is_validation_passed).to_owned(),
        before: Some(before.active_count),
        after: Some(after.active_count),
        decision: decision_label(decision),
        restoration: "not_required".to_owned(),
        next_action: next_action(decision).to_owned(),
    }
}

fn octet_summary(
    definition_id: &str,
    before: &super::core::OctetSnapshot,
    after: &super::core::OctetSnapshot,
    is_validation_passed: bool,
    decision: &super::core::ProgressDecision,
) -> Summary {
    Summary {
        bundle: definition_id.to_owned(),
        selected_work_fingerprint: None,
        validation: validation_label(is_validation_passed).to_owned(),
        before: before.findings.parse().ok(),
        after: after.findings.parse().ok(),
        decision: decision_label(decision),
        restoration: restoration_label(before, after).to_owned(),
        next_action: next_action(decision).to_owned(),
    }
}

fn partial_summary(kind: super::Kind, definition_id: &str, details: &crate::store::RunDetailRecord) -> Summary {
    let validation = validation_state(details);
    match kind {
        super::Kind::Cairn => partial_cairn_summary(definition_id, details, validation),
        super::Kind::Octet => partial_octet_summary(definition_id, details, validation),
    }
}

fn validation_state(details: &crate::store::RunDetailRecord) -> String {
    details
        .nodes
        .iter()
        .find(|node| node.node_label == VALIDATION_NODE_LABEL)
        .map(|node| match node.status {
            crate::store::RunNodeDetailStatus::Completed => "passed",
            crate::store::RunNodeDetailStatus::Failed => "failed",
            crate::store::RunNodeDetailStatus::Pending
            | crate::store::RunNodeDetailStatus::Running
            | crate::store::RunNodeDetailStatus::Skipped => UNKNOWN_FACT,
        })
        .unwrap_or(UNKNOWN_FACT)
        .to_owned()
}

fn partial_cairn_summary(definition_id: &str, details: &crate::store::RunDetailRecord, validation: String) -> Summary {
    debug_assert_eq!(definition_id, super::authority::CAIRN_BUNDLE_ID);
    debug_assert!(matches!(validation.as_str(), "passed" | "failed" | UNKNOWN_FACT));
    let snapshot = node_output(details, CAIRN_DISCOVERY_OUTPUT)
        .and_then(|value| serde_json::from_str::<super::core::CairnSnapshot>(value).ok());
    let is_complete = snapshot.as_ref().is_some_and(|value| value.active_count == 0);
    Summary {
        bundle: definition_id.to_owned(),
        selected_work_fingerprint: snapshot
            .as_ref()
            .and_then(|value| value.selected.as_deref())
            .map(selected_fingerprint),
        validation,
        before: snapshot.as_ref().map(|value| value.active_count),
        after: is_complete.then_some(0),
        decision: if is_complete {
            "complete"
        } else {
            "blocked:incomplete_round"
        }
        .to_owned(),
        restoration: "not_required".to_owned(),
        next_action: if is_complete { "stop" } else { "inspect_run" }.to_owned(),
    }
}

fn partial_octet_summary(definition_id: &str, details: &crate::store::RunDetailRecord, validation: String) -> Summary {
    debug_assert_eq!(definition_id, super::authority::OCTET_BUNDLE_ID);
    debug_assert!(matches!(validation.as_str(), "passed" | "failed" | UNKNOWN_FACT));
    let snapshot = node_output(details, OCTET_BASELINE_OUTPUT)
        .and_then(|value| serde_json::from_str::<super::core::OctetSnapshot>(value).ok());
    Summary {
        bundle: definition_id.to_owned(),
        selected_work_fingerprint: None,
        validation,
        before: snapshot.as_ref().and_then(|value| value.findings.parse().ok()),
        after: None,
        decision: "blocked:incomplete_round".to_owned(),
        restoration: snapshot
            .as_ref()
            .map_or(UNKNOWN_FACT, |value| {
                if value.restoration_confirmed {
                    "confirmed"
                } else {
                    "blocked"
                }
            })
            .to_owned(),
        next_action: "inspect_run".to_owned(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct MergeObservation {
    label: String,
    value: serde_json::Value,
}

fn merge_value<'a>(records: &'a [MergeObservation], label: &str) -> Option<&'a str> {
    merge_value_json(records, label)?.as_str().map(str::trim)
}

fn merge_value_json<'a>(records: &'a [MergeObservation], label: &str) -> Option<&'a serde_json::Value> {
    let mut matches = records.iter().filter(|record| record.label == label);
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(&first.value)
}

fn node_output(details: &crate::store::RunDetailRecord, selector: OutputSelector) -> Option<&str> {
    details
        .nodes
        .iter()
        .find(|node| node.node_label == selector.node_label)
        .and_then(|node| node.outputs.iter().find(|output| output.port_label == selector.port_label))
        .map(|output| output.text.as_str())
}

fn selected_fingerprint(selected: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SELECTED_FINGERPRINT_DOMAIN);
    hasher.update(selected.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn validation_label(is_passed: bool) -> &'static str {
    if is_passed { "passed" } else { "failed" }
}

fn restoration_label(before: &super::core::OctetSnapshot, after: &super::core::OctetSnapshot) -> &'static str {
    let facts = [
        before.restoration_confirmed,
        after.restoration_confirmed,
        before.policy_identity_before == before.policy_identity_after,
        after.policy_identity_before == after.policy_identity_after,
        before.policy_identity_before == after.policy_identity_before,
    ];
    if facts.into_iter().all(std::convert::identity) {
        "confirmed"
    } else {
        "blocked"
    }
}

fn decision_label(decision: &super::core::ProgressDecision) -> String {
    match decision {
        super::core::ProgressDecision::Improved { .. } => "improved".to_owned(),
        super::core::ProgressDecision::Complete { .. } => "complete".to_owned(),
        super::core::ProgressDecision::Blocked { reason } => format!("blocked:{}", reason.as_str()),
    }
}

fn next_action(decision: &super::core::ProgressDecision) -> &'static str {
    match decision {
        super::core::ProgressDecision::Improved { .. } => "review_successor",
        super::core::ProgressDecision::Complete { .. } => "stop",
        super::core::ProgressDecision::Blocked { .. } => "inspect_run",
    }
}

fn optional_metric(value: Option<u64>) -> String {
    value.map_or_else(|| ABSENT_FACT.to_owned(), |value| value.to_string())
}
