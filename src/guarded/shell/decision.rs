#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeObservation {
    label: String,
    sequence: u64,
    value: serde_json::Value,
}

// r[impl lattice.guarded_agent_templates.rounds]
pub fn decide_from_merged_json(
    kind: super::Kind,
    minimum_delta: u64,
    bytes: &[u8],
) -> Result<super::core::ProgressDecision, Error> {
    if bytes.len() > MAX_SUMMARY_BYTES {
        return Err(Error::LimitExceeded);
    }
    let observations: Vec<MergeObservation> = serde_json::from_slice(bytes).map_err(|_| Error::MalformedObservation)?;
    if observations.len() != MERGED_OBSERVATION_COUNT {
        return Err(Error::MalformedObservation);
    }
    debug_assert_eq!(observations.len(), MERGED_OBSERVATION_COUNT);
    let mut labels = std::collections::BTreeMap::new();
    for observation in observations {
        if labels.insert(observation.label, (observation.sequence, observation.value)).is_some() {
            return Err(Error::MalformedObservation);
        }
    }
    debug_assert_eq!(labels.len(), MERGED_OBSERVATION_COUNT);
    let before = merged_string(&labels, "before")?;
    let after = merged_string(&labels, "after")?;
    let is_validation_passed = merged_exit(&labels, "validation")?;
    let policy = super::core::ProgressPolicy::decreasing(minimum_delta).map_err(|_| Error::MalformedObservation)?;
    match kind {
        super::Kind::Cairn => decide_cairn_merged(policy, MergedPair { before, after }, is_validation_passed),
        super::Kind::Octet => decide_octet_merged(policy, MergedPair { before, after }, is_validation_passed),
    }
}

struct MergedPair<'a> {
    before: &'a str,
    after: &'a str,
}

fn decide_cairn_merged(
    policy: super::core::ProgressPolicy,
    merged: MergedPair<'_>,
    is_validation_passed: bool,
) -> Result<super::core::ProgressDecision, Error> {
    let before: super::core::CairnSnapshot =
        serde_json::from_str(merged.before).map_err(|_| Error::MalformedObservation)?;
    let after: super::core::CairnSnapshot =
        serde_json::from_str(merged.after).map_err(|_| Error::MalformedObservation)?;
    Ok(super::core::decide_cairn_progress(policy, &before, &after, is_validation_passed))
}

fn decide_octet_merged(
    policy: super::core::ProgressPolicy,
    merged: MergedPair<'_>,
    is_validation_passed: bool,
) -> Result<super::core::ProgressDecision, Error> {
    let before: super::core::OctetSnapshot =
        serde_json::from_str(merged.before).map_err(|_| Error::MalformedObservation)?;
    let after: super::core::OctetSnapshot =
        serde_json::from_str(merged.after).map_err(|_| Error::MalformedObservation)?;
    Ok(super::core::decide_octet_progress(policy, &before, &after, is_validation_passed))
}

fn merged_string<'a>(
    labels: &'a std::collections::BTreeMap<String, (u64, serde_json::Value)>,
    label: &str,
) -> Result<&'a str, Error> {
    labels
        .get(label)
        .and_then(|(_, value)| value.as_str())
        .map(str::trim)
        .ok_or(Error::MalformedObservation)
}

fn merged_exit(
    labels: &std::collections::BTreeMap<String, (u64, serde_json::Value)>,
    label: &str,
) -> Result<bool, Error> {
    labels
        .get(label)
        .and_then(|(_, value)| value.as_i64())
        .map(|value| value == 0)
        .ok_or(Error::MalformedObservation)
}

pub fn read_stdin_bounded() -> Result<Vec<u8>, Error> {
    let read_limit_bytes = u64::try_from(MAX_SUMMARY_BYTES.saturating_add(1)).map_err(|_| Error::LimitExceeded)?;
    let mut bytes = Vec::with_capacity(MAX_SUMMARY_BYTES);
    std::io::stdin()
        .take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::MalformedObservation)?;
    if bytes.len() > MAX_SUMMARY_BYTES {
        return Err(Error::LimitExceeded);
    }
    Ok(bytes)
}
