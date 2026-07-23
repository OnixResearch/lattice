pub(crate) struct ExpectedGeneratedDocument<'a> {
    pub(crate) path: &'a str,
    pub(crate) checked_in: &'a str,
}

pub(crate) struct ExpectedArtifact<'a> {
    pub(crate) path: &'a str,
    pub(crate) bytes: &'a [u8],
}

pub(crate) const EXPECTED_ARTIFACTS: &[ExpectedArtifact<'static>] = &[
    ExpectedArtifact {
        path: "docs/generated/snapshots/dialog-basic.subwayrat.typ",
        bytes: include_bytes!("../../../../docs/generated/snapshots/dialog-basic.subwayrat.typ"),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/dialog-basic.meteorite.html",
        bytes: include_bytes!("../../../../docs/generated/snapshots/dialog-basic.meteorite.html"),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/chrome-shell.subwayrat.typ",
        bytes: include_bytes!("../../../../docs/generated/snapshots/chrome-shell.subwayrat.typ"),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/chrome-shell.meteorite.html",
        bytes: include_bytes!("../../../../docs/generated/snapshots/chrome-shell.meteorite.html"),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/knowledge-workflow.subwayrat.typ",
        bytes: include_bytes!(
            "../../../../docs/generated/snapshots/knowledge-workflow.subwayrat.typ"
        ),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/knowledge-workflow.meteorite.html",
        bytes: include_bytes!(
            "../../../../docs/generated/snapshots/knowledge-workflow.meteorite.html"
        ),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/theme-tokens.subwayrat.typ",
        bytes: include_bytes!("../../../../docs/generated/snapshots/theme-tokens.subwayrat.typ"),
    },
    ExpectedArtifact {
        path: "docs/generated/snapshots/theme-tokens.meteorite.html",
        bytes: include_bytes!("../../../../docs/generated/snapshots/theme-tokens.meteorite.html"),
    },
];

pub(crate) fn assert_generated_document(expected: ExpectedGeneratedDocument<'_>) {
    let documents = super::super::built_in_generated_component_documents();
    let document = documents
        .iter()
        .find(|candidate| candidate.page.path == expected.path)
        .expect("generated component document exists");
    let validation_status =
        super::super::validate_generated_document(super::super::GeneratedDocumentCheck {
            subject: expected.path.to_string(),
            expected: &document.content,
            checked_in: expected.checked_in,
        });

    assert!(document.issues.is_empty());
    assert!(validation_status.is_valid());
}

pub(crate) fn assert_artifact_bytes(
    artifacts: &[super::super::SnapshotArtifact],
    expected: &ExpectedArtifact<'_>,
) {
    let artifact = artifacts
        .iter()
        .find(|entry| entry.path == expected.path)
        .expect("artifact path exists");

    assert_eq!(artifact.bytes.as_slice(), expected.bytes);
}
