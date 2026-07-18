use super::*;

/// Panel-ready knowledge workflow data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgePanels {
    /// Outline rows `(document title, heading title, level)`.
    pub outline: Vec<(String, String, u8)>,
    /// Capture target labels that are writable.
    pub capture_targets: Vec<String>,
    /// Tag labels mapped to document IDs.
    pub tags: std::collections::BTreeMap<String, Vec<String>>,
    /// Backlink target IDs mapped to source IDs.
    pub backlinks: std::collections::BTreeMap<String, Vec<String>>,
    /// Agenda item labels.
    pub agenda: Vec<String>,
    /// Markdown preview bodies by document ID.
    pub markdown: std::collections::BTreeMap<String, String>,
    /// Search records by document ID.
    pub search: std::collections::BTreeMap<String, String>,
    /// Graph adjacency by source ID.
    pub graph: std::collections::BTreeMap<String, Vec<String>>,
}

impl KnowledgePanels {
    /// Returns the canonical panel IDs used by the workflow composition.
    #[must_use]
    pub fn panel_ids() -> Vec<&'static str> {
        vec![
            PANEL_OUTLINE,
            PANEL_CAPTURE,
            PANEL_TAGS,
            PANEL_BACKLINKS,
            PANEL_AGENDA,
            PANEL_MARKDOWN,
            PANEL_SEARCH,
            PANEL_GRAPH,
        ]
    }
}

/// Knowledge workflow build result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWorkflowBuild {
    /// Panel data safe to render.
    pub panels: KnowledgePanels,
    /// Deterministic diagnostics.
    pub issues: Vec<KnowledgeIssue>,
}

impl KnowledgeWorkflowBuild {
    /// Returns true when the workspace has no diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Builds deterministic panel models from adapter input.
#[must_use]
pub fn build_knowledge_workflow(input: &KnowledgeWorkspaceInput) -> KnowledgeWorkflowBuild {
    let issues = validate_workspace(input);
    let panels = build_panels(input);
    KnowledgeWorkflowBuild { panels, issues }
}

fn validate_workspace(input: &KnowledgeWorkspaceInput) -> Vec<KnowledgeIssue> {
    let mut issues = Vec::new();
    let document_ids = document_id_set(&input.documents);
    debug_assert!(document_ids.len() <= input.documents.len());
    debug_assert!(issues.len() <= issues.capacity());
    if input.documents.is_empty() {
        issues.push(KnowledgeIssue::new(
            KnowledgeIssueCode::EmptyDocuments,
            "documents",
            "at least one document is required",
        ));
    }
    issues.extend(duplicate_document_issues(&input.documents));
    issues.extend(heading_issues(&input.headings, &document_ids));
    issues.extend(tag_issues(&input.tags, &document_ids));
    issues.extend(backlink_issues(&input.backlinks, &document_ids));
    issues.extend(agenda_issues(&input.agenda, &document_ids));
    issues.extend(capture_issues(&input.captures));
    if input.search.is_empty() {
        issues.push(KnowledgeIssue::new(
            KnowledgeIssueCode::EmptySearch,
            "search",
            "search records are required for the knowledge workflow",
        ));
    }
    debug_assert!(issues.len() <= issues.capacity());
    issues
}

fn document_id_set(documents: &[KnowledgeDocument]) -> std::collections::BTreeSet<String> {
    documents
        .iter()
        .map(|document| document.id.clone())
        .collect()
}

fn duplicate_document_issues(documents: &[KnowledgeDocument]) -> Vec<KnowledgeIssue> {
    let mut seen = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(documents.len());
    for document in documents {
        if !seen.insert(document.id.clone()) {
            issues.push(KnowledgeIssue::new(
                KnowledgeIssueCode::DuplicateDocument,
                document.id.clone(),
                "duplicate document ID",
            ));
        }
    }
    issues
}

fn heading_issues(
    headings: &[KnowledgeHeading],
    document_ids: &std::collections::BTreeSet<String>,
) -> Vec<KnowledgeIssue> {
    headings
        .iter()
        .filter(|heading| {
            heading.title.trim().is_empty()
                || heading.level == 0
                || !document_ids.contains(&heading.document_id)
        })
        .map(|heading| {
            KnowledgeIssue::new(
                KnowledgeIssueCode::MalformedHeading,
                heading.document_id.clone(),
                "heading must reference a document, have text, and use level >= 1",
            )
        })
        .collect()
}

fn tag_issues(
    tags: &[KnowledgeTag],
    document_ids: &std::collections::BTreeSet<String>,
) -> Vec<KnowledgeIssue> {
    tags.iter()
        .filter(|tag| {
            !tag.label.starts_with(TAG_PREFIX) || !document_ids.contains(&tag.document_id)
        })
        .map(|tag| {
            KnowledgeIssue::new(
                KnowledgeIssueCode::InvalidTag,
                tag.label.clone(),
                "tag must start with # and reference an existing document",
            )
        })
        .collect()
}

fn backlink_issues(
    backlinks: &[KnowledgeBacklink],
    document_ids: &std::collections::BTreeSet<String>,
) -> Vec<KnowledgeIssue> {
    backlinks
        .iter()
        .filter(|link| {
            !document_ids.contains(&link.source_id) || !document_ids.contains(&link.target_id)
        })
        .map(|link| {
            KnowledgeIssue::new(
                KnowledgeIssueCode::MissingLinkTarget,
                format!("{}->{}", link.source_id, link.target_id),
                "backlink source and target must exist",
            )
        })
        .collect()
}

fn agenda_issues(
    agenda: &[KnowledgeAgendaItem],
    document_ids: &std::collections::BTreeSet<String>,
) -> Vec<KnowledgeIssue> {
    agenda
        .iter()
        .filter(|item| {
            !document_ids.contains(&item.document_id)
                || !is_iso_date(&item.start)
                || item
                    .end
                    .as_ref()
                    .is_some_and(|end| !is_iso_date(end) || end < &item.start)
        })
        .map(|item| {
            KnowledgeIssue::new(
                KnowledgeIssueCode::BadDateRange,
                item.title.clone(),
                "agenda item must reference a document and use an ordered YYYY-MM-DD date range",
            )
        })
        .collect()
}

fn capture_issues(captures: &[KnowledgeCaptureTarget]) -> Vec<KnowledgeIssue> {
    captures
        .iter()
        .filter(|target| !target.writable)
        .map(|target| {
            KnowledgeIssue::new(
                KnowledgeIssueCode::FailedCaptureTarget,
                target.id.clone(),
                "capture target is not writable by the application adapter",
            )
        })
        .collect()
}

fn build_panels(input: &KnowledgeWorkspaceInput) -> KnowledgePanels {
    let titles = input
        .documents
        .iter()
        .map(|document| (document.id.clone(), document.title.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let panels = KnowledgePanels {
        outline: input
            .headings
            .iter()
            .filter_map(|heading| {
                let title = titles.get(&heading.document_id)?;
                Some((title.clone(), heading.title.clone(), heading.level))
            })
            .collect(),
        capture_targets: input
            .captures
            .iter()
            .filter(|target| target.writable)
            .map(|target| target.label.clone())
            .collect(),
        tags: group_pairs(
            input
                .tags
                .iter()
                .map(|tag| (tag.label.clone(), tag.document_id.clone())),
        ),
        backlinks: group_pairs(
            input
                .backlinks
                .iter()
                .map(|link| (link.target_id.clone(), link.source_id.clone())),
        ),
        agenda: input
            .agenda
            .iter()
            .map(|item| format!("{} {}", item.start, item.title))
            .collect(),
        markdown: input
            .documents
            .iter()
            .map(|document| (document.id.clone(), document.body.clone()))
            .collect(),
        search: input
            .search
            .iter()
            .map(|record| (record.document_id.clone(), record.text.clone()))
            .collect(),
        graph: group_pairs(
            input
                .graph
                .iter()
                .map(|edge| (edge.source_id.clone(), edge.target_id.clone())),
        ),
    };
    debug_assert!(panels.markdown.len() <= input.documents.len());
    debug_assert!(panels.search.len() <= input.search.len());
    panels
}

fn group_pairs<I>(pairs: I) -> std::collections::BTreeMap<String, Vec<String>>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut grouped: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (key, value) in pairs {
        grouped.entry(key).or_default().push(value);
    }
    grouped
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}
