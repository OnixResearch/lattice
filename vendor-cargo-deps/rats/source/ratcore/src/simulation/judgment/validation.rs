/// Validates and summarizes pure invariant observations.
///
/// # Errors
///
/// Returns duplicate, unknown, category, detail, and guidance diagnostics.
pub fn summarize_oracles(
    registry: &SimulationIdentityRegistry,
    catalog: &[SimulationOracleCatalogEntry],
    observations: &[SimulationOracleObservation],
) -> Result<Vec<SimulationOracleSummary>, Vec<SimulationOracleDiagnostic>> {
    if catalog.len() > MAX_ORACLES {
        return Err(vec![oracle_issue(
            SimulationOracleDiagnosticCode::CatalogTooLarge,
            None,
        )]);
    }
    if observations.len() > MAX_ORACLES.saturating_mul(4) {
        return Err(vec![oracle_issue(
            SimulationOracleDiagnosticCode::ObservationHistoryTooLarge,
            None,
        )]);
    }
    debug_assert!(catalog.len() <= MAX_ORACLES);
    debug_assert!(observations.len() <= MAX_ORACLES.saturating_mul(4));
    let mut issues = OracleIssues::default();
    let entries = collect_catalog(registry, catalog, &mut issues);
    let mut grouped = collect_observations(&entries, observations, &mut issues);
    if !issues.0.is_empty() {
        return Err(issues.0);
    }
    Ok(build_summaries(entries, &mut grouped))
}

#[derive(Default)]
struct OracleIssues(Vec<SimulationOracleDiagnostic>);

impl OracleIssues {
    fn push(&mut self, issue: SimulationOracleDiagnostic) {
        if self.0.len() < MAX_ORACLES {
            self.0.push(issue);
        }
    }
}

fn collect_catalog<'a>(
    registry: &SimulationIdentityRegistry,
    catalog: &'a [SimulationOracleCatalogEntry],
    issues: &mut OracleIssues,
) -> std::collections::BTreeMap<&'a str, &'a SimulationOracleCatalogEntry> {
    debug_assert!(catalog.len() <= MAX_ORACLES);
    debug_assert!(issues.0.len() <= MAX_ORACLES);
    let mut entries = std::collections::BTreeMap::new();
    for entry in catalog {
        let id = entry.invariant_id.as_str();
        validate_catalog_entry(registry, id, entry, issues);
        if entries.len() < MAX_ORACLES && entries.insert(id, entry).is_some() {
            issues.push(oracle_issue(
                SimulationOracleDiagnosticCode::DuplicateInvariant,
                Some(id),
            ));
        }
    }
    entries
}

fn validate_catalog_entry(
    registry: &SimulationIdentityRegistry,
    id: &str,
    entry: &SimulationOracleCatalogEntry,
    issues: &mut OracleIssues,
) {
    if !registry.contains(SimulationIdentityKind::Invariant, id) {
        issues.push(oracle_issue(
            SimulationOracleDiagnosticCode::UnknownInvariant,
            Some(id),
        ));
    }
    if !valid_detail(&entry.detail) {
        issues.push(oracle_issue(
            SimulationOracleDiagnosticCode::InvalidDetail,
            Some(id),
        ));
    }
}

fn collect_observations<'a>(
    entries: &std::collections::BTreeMap<&str, &SimulationOracleCatalogEntry>,
    observations: &'a [SimulationOracleObservation],
    issues: &mut OracleIssues,
) -> std::collections::BTreeMap<&'a str, Vec<&'a SimulationOracleObservation>> {
    debug_assert!(observations.len() <= MAX_ORACLES.saturating_mul(4));
    debug_assert!(issues.0.len() <= MAX_ORACLES);
    let mut grouped = std::collections::BTreeMap::new();
    for observation in observations {
        let id = observation.invariant_id.as_str();
        let Some(entry) = entries.get(id) else {
            issues.push(oracle_issue(
                SimulationOracleDiagnosticCode::UnknownInvariant,
                Some(id),
            ));
            continue;
        };
        validate_observation(id, entry, observation, issues);
        let facts = grouped.entry(id).or_insert_with(Vec::new);
        if facts.len() < MAX_ORACLES.saturating_mul(4) {
            facts.push(observation);
        }
    }
    grouped
}

fn validate_observation(
    id: &str,
    entry: &SimulationOracleCatalogEntry,
    observation: &SimulationOracleObservation,
    issues: &mut OracleIssues,
) {
    debug_assert_eq!(observation.invariant_id.as_str(), id);
    debug_assert!(issues.0.len() <= MAX_ORACLES);
    if observation.category != entry.category {
        issues.push(oracle_issue(
            SimulationOracleDiagnosticCode::CategoryMismatch,
            Some(id),
        ));
    }
    if !valid_detail(&observation.detail) {
        issues.push(oracle_issue(
            SimulationOracleDiagnosticCode::InvalidDetail,
            Some(id),
        ));
    }
    if observation
        .guidance
        .is_some_and(|distance| !distance.is_finite() || distance < 0.0)
    {
        issues.push(oracle_issue(
            SimulationOracleDiagnosticCode::InvalidGuidance,
            Some(id),
        ));
    }
}

fn build_summaries(
    entries: std::collections::BTreeMap<&str, &SimulationOracleCatalogEntry>,
    grouped: &mut std::collections::BTreeMap<&str, Vec<&SimulationOracleObservation>>,
) -> Vec<SimulationOracleSummary> {
    let mut summaries = Vec::with_capacity(entries.len());
    for (id, entry) in entries {
        let facts = grouped.remove(id).unwrap_or_default();
        summaries.push(SimulationOracleSummary {
            invariant_id: entry.invariant_id.clone(),
            category: entry.category,
            status: summarize_category(entry.category, &facts),
            observations: facts.iter().fold(0_u32, |count, _| count.saturating_add(1)),
        });
    }
    summaries
}
