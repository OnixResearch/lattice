use super::*;

const DOC: &str = "doc";
const OTHER: &str = "other";

#[test]
fn valid_workspace_builds_panel_models() {
    let input = valid_workspace();

    let build = build_knowledge_workflow(&input);

    assert!(build.is_valid());
    assert_eq!(KnowledgePanels::panel_ids().len(), 8);
    assert_eq!(build.panels.outline[0].1, "Heading");
    assert_eq!(build.panels.capture_targets, vec!["Inbox".to_string()]);
    assert_eq!(build.panels.graph.get(DOC), Some(&vec![DocumentId::new(OTHER)]));
}

#[test]
fn malformed_workspace_reports_deterministic_issues() {
    let mut input = valid_workspace();
    input.documents.push(KnowledgeDocument::new(DOC, "Duplicate", ""));
    input.headings.push(KnowledgeHeading::new("missing", "", 0));
    input.tags.push(KnowledgeTag::new(DOC, "bad"));
    input.backlinks.push(KnowledgeBacklink::new(DOC, "missing"));
    let malformed = RawKnowledgeAgendaItem::new(DOC, "Bad", "2026-99").with_end("2025-01-01");
    assert!(matches!(malformed.validate(), Err(KnowledgeAgendaDateError::InvalidDate { field: "start", .. })));
    input.agenda.push(KnowledgeAgendaItem::new(
        "missing",
        "Bad",
        crate::caldate::CalDate::parse("2026-06-11").unwrap(),
    ));
    input.captures.push(KnowledgeCaptureTarget::new("readonly", "Read-only", false));
    input.search.clear();

    let build = build_knowledge_workflow(&input);

    assert!(!build.is_valid());
    assert!(!build.issues.is_empty());
    assert_issue(&build, KnowledgeIssueCode::DuplicateDocument);
    assert_issue(&build, KnowledgeIssueCode::MalformedHeading);
    assert_issue(&build, KnowledgeIssueCode::InvalidTag);
    assert_issue(&build, KnowledgeIssueCode::MissingLinkTarget);
    assert_issue(&build, KnowledgeIssueCode::BadDateRange);
    assert_issue(&build, KnowledgeIssueCode::FailedCaptureTarget);
    assert_issue(&build, KnowledgeIssueCode::EmptySearch);
}

#[test]
fn empty_workspace_fails_safely() {
    let build = build_knowledge_workflow(&KnowledgeWorkspaceInput::default());

    assert_issue(&build, KnowledgeIssueCode::EmptyDocuments);
    assert_issue(&build, KnowledgeIssueCode::EmptySearch);
    assert!(build.panels.outline.is_empty());
}

fn valid_workspace() -> KnowledgeWorkspaceInput {
    KnowledgeWorkspaceInput {
        documents: vec![
            KnowledgeDocument::new(DOC, "Doc", "Body"),
            KnowledgeDocument::new(OTHER, "Other", "Other body"),
        ],
        headings: vec![KnowledgeHeading::new(DOC, "Heading", 1)],
        tags: vec![KnowledgeTag::new(DOC, "#tag")],
        backlinks: vec![KnowledgeBacklink::new(OTHER, DOC)],
        agenda: vec![KnowledgeAgendaItem::new(
            DOC,
            "Review",
            crate::caldate::CalDate::parse("2026-06-11").unwrap(),
        )],
        captures: vec![KnowledgeCaptureTarget::new("inbox", "Inbox", true)],
        search: vec![KnowledgeSearchRecord::new(DOC, "Doc Heading Body")],
        graph: vec![KnowledgeGraphEdge::new(DOC, OTHER)],
    }
}

fn assert_issue(build: &KnowledgeWorkflowBuild, code: KnowledgeIssueCode) {
    assert!(build.issues.iter().any(|issue| issue.code == code));
}
