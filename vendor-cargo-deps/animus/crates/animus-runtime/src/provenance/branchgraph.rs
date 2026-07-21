/// Parent graph entry used for pure branch queries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceGraphEntry {
    /// Stable entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
}

/// Pure branch-query report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceBranchReport {
    /// Active head entry identity.
    pub active_head: Text,
    /// Active ancestry including the active head.
    pub active_ancestry: Collection<Text>,
    /// Entries not reachable from the active head.
    pub dead_end_entry_ids: Collection<Text>,
    /// Entries unique to each direct branch parent or head candidate.
    pub unique_branch_entries: BTreeMap<Text, Collection<Text>>,
}

/// Validates a provenance parent graph and returns branch/dead-end information.
///
/// # Errors
///
/// Returns [`ProvenanceError`] on duplicate ids, missing parents, or cycles.
pub fn validate_provenance_graph(
    entries: &[ProvenanceGraphEntry],
    active_head: &str,
) -> Result<ProvenanceBranchReport, ProvenanceError> {
    let graph = build_graph(entries)?;
    if !graph.contains_key(active_head) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::MissingParent,
            "active_head",
            active_head,
            "active head is not present in graph",
        ));
    }
    detect_cycles(&graph)?;
    let active_ancestry_set = ancestry_set(&graph, active_head)?;
    let mut active_ancestry: Collection<Text> = active_ancestry_set.iter().cloned().collect();
    active_ancestry.sort();
    let mut dead_end_entry_ids: Collection<Text> = graph
        .keys()
        .filter(|entry_id| !active_ancestry_set.contains(*entry_id))
        .cloned()
        .collect();
    dead_end_entry_ids.sort();
    let unique_branch_entries = unique_branches(&graph, active_head)?;
    Ok(ProvenanceBranchReport {
        active_head: active_head.to_owned(),
        active_ancestry,
        dead_end_entry_ids,
        unique_branch_entries,
    })
}

fn build_graph(
    entries: &[ProvenanceGraphEntry],
) -> Result<BTreeMap<Text, Collection<Text>>, ProvenanceError> {
    let mut graph = BTreeMap::new();
    for entry in entries {
        validate_safe_text("graph_id", entry.id.clone())?;
        if graph.contains_key(entry.id.as_str()) {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::DuplicateEntry,
                "graph_id",
                entry.id.as_str(),
                "entry id appeared more than once",
            ));
        }
        graph.insert(entry.id.clone(), entry.parents.clone());
    }
    for (entry_id, parents) in &graph {
        for parent in parents {
            if !graph.contains_key(parent.as_str()) {
                return Err(ProvenanceError::new(
                    ProvenanceErrorKind::MissingParent,
                    "parent_id",
                    entry_id.as_str(),
                    format!("missing parent {parent}"),
                ));
            }
        }
    }
    Ok(graph)
}

fn detect_cycles(graph: &BTreeMap<Text, Collection<Text>>) -> Result<(), ProvenanceError> {
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for entry_id in graph.keys() {
        visit_for_cycle(entry_id, graph, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit_for_cycle(
    entry_id: &str,
    graph: &BTreeMap<Text, Collection<Text>>,
    visiting: &mut BTreeSet<Text>,
    visited: &mut BTreeSet<Text>,
) -> Result<(), ProvenanceError> {
    if visited.contains(entry_id) {
        return Ok(());
    }
    if !visiting.insert(entry_id.to_owned()) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::Cycle,
            "graph_id",
            entry_id,
            "cycle detected in ancestry",
        ));
    }
    let parents = graph.get(entry_id).ok_or_else(|| {
        ProvenanceError::new(
            ProvenanceErrorKind::MissingParent,
            "graph_id",
            entry_id,
            "entry missing during cycle check",
        )
    })?;
    for parent in parents {
        visit_for_cycle(parent.as_str(), graph, visiting, visited)?;
    }
    visiting.remove(entry_id);
    visited.insert(entry_id.to_owned());
    Ok(())
}

fn ancestry_set(
    graph: &BTreeMap<Text, Collection<Text>>,
    entry_id: &str,
) -> Result<BTreeSet<Text>, ProvenanceError> {
    let mut ancestry = BTreeSet::new();
    collect_ancestry(graph, entry_id, &mut ancestry)?;
    Ok(ancestry)
}

fn collect_ancestry(
    graph: &BTreeMap<Text, Collection<Text>>,
    entry_id: &str,
    ancestry: &mut BTreeSet<Text>,
) -> Result<(), ProvenanceError> {
    if !ancestry.insert(entry_id.to_owned()) {
        return Ok(());
    }
    let parents = graph.get(entry_id).ok_or_else(|| {
        ProvenanceError::new(
            ProvenanceErrorKind::MissingParent,
            "graph_id",
            entry_id,
            "entry missing during ancestry collection",
        )
    })?;
    for parent in parents {
        collect_ancestry(graph, parent.as_str(), ancestry)?;
    }
    Ok(())
}

fn unique_branches(
    graph: &BTreeMap<Text, Collection<Text>>,
    active_head: &str,
) -> Result<BTreeMap<Text, Collection<Text>>, ProvenanceError> {
    let mut branch_heads = BTreeSet::new();
    branch_heads.insert(active_head.to_owned());
    for parents in graph.values() {
        for parent in parents {
            branch_heads.insert(parent.clone());
        }
    }
    let mut reports = BTreeMap::new();
    for branch_head in branch_heads {
        let branch_ancestry = ancestry_set(graph, branch_head.as_str())?;
        let mut other_ancestry = BTreeSet::new();
        for other_head in graph.keys() {
            if other_head != &branch_head {
                other_ancestry.extend(ancestry_set(graph, other_head.as_str())?);
            }
        }
        let mut unique: Collection<Text> = branch_ancestry
            .difference(&other_ancestry)
            .cloned()
            .collect();
        unique.sort();
        if !unique.is_empty() {
            reports.insert(branch_head, unique);
        }
    }
    Ok(reports)
}
