/// Returns invariant identities that were cataloged but never observed.
#[must_use]
pub fn unobserved_oracle_ids(
    summaries: &[SimulationOracleSummary],
) -> std::collections::BTreeSet<String> {
    summaries
        .iter()
        .filter(|summary| summary.observations == 0)
        .map(|summary| summary.invariant_id.as_str().to_string())
        .collect()
}

fn summarize_category(
    category: SimulationOracleCategory,
    facts: &[&SimulationOracleObservation],
) -> SimulationOracleStatus {
    debug_assert!(facts.len() <= MAX_ORACLES.saturating_mul(4));
    debug_assert!(facts.iter().all(|fact| fact.category == category));
    if facts.is_empty() {
        return match category {
            SimulationOracleCategory::Unreachable => SimulationOracleStatus::UnreachablePreserved,
            _ => SimulationOracleStatus::CatalogedButUnobserved,
        };
    }
    match category {
        SimulationOracleCategory::Always => {
            if facts.iter().all(|fact| fact.condition) {
                SimulationOracleStatus::AlwaysPassed
            } else {
                SimulationOracleStatus::AlwaysFailed
            }
        }
        SimulationOracleCategory::Sometimes => {
            if facts.iter().any(|fact| fact.condition) {
                SimulationOracleStatus::SometimesObserved
            } else {
                SimulationOracleStatus::SometimesUnobservedSuccess
            }
        }
        SimulationOracleCategory::Reachable => {
            if facts.iter().any(|fact| fact.condition) {
                SimulationOracleStatus::Reached
            } else {
                SimulationOracleStatus::Unreached
            }
        }
        SimulationOracleCategory::Unreachable => {
            if facts.iter().any(|fact| fact.condition) {
                SimulationOracleStatus::UnreachableViolated
            } else {
                SimulationOracleStatus::UnreachablePreserved
            }
        }
    }
}

fn valid_detail(detail: &str) -> bool {
    if detail.is_empty() || detail.len() > MAX_ORACLE_DETAIL_BYTES {
        return false;
    }
    let lower = detail.to_ascii_lowercase();
    detail.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'+' | b'=')
    }) && !lower.contains("password")
        && !lower.contains("credential")
        && !lower.contains("secret")
        && !lower.contains("clipboard")
        && !lower.contains("process-output")
        && !lower.contains("debug(")
}

fn oracle_issue(
    code: SimulationOracleDiagnosticCode,
    invariant_id: Option<&str>,
) -> SimulationOracleDiagnostic {
    SimulationOracleDiagnostic {
        code,
        invariant_id: invariant_id.map(|value| value.chars().take(96).collect()),
    }
}
