use super::super::*;
use super::support::*;

#[test]
fn parser_accepts_supported_reference_shapes() -> Result<(), ContextReferenceError> {
    let file = parse_context_reference(SAMPLE_FILE)?;
    assert_eq!(file.kind(), ContextReferenceKind::File);
    assert_eq!(file.normalized_target(), "src/lib.rs");
    assert_eq!(file.range().map(|range| range.start_line()), Some(2));
    assert_eq!(file.range().and_then(|range| range.end_line()), Some(4));
    assert_eq!(file.render_mode(), ContextRenderMode::Text);

    let directory = parse_context_reference(SAMPLE_DIRECTORY)?;
    assert_eq!(directory.kind(), ContextReferenceKind::Directory);
    assert_eq!(directory.render_mode(), ContextRenderMode::DirectoryListing);

    let diff = parse_context_reference(SAMPLE_DIFF)?;
    assert_eq!(diff.kind(), ContextReferenceKind::Diff);
    assert_eq!(diff.render_mode(), ContextRenderMode::Diff);

    let image = parse_context_reference(SAMPLE_IMAGE)?;
    assert_eq!(image.kind(), ContextReferenceKind::Image);
    assert_eq!(image.render_mode(), ContextRenderMode::Image);

    let url = parse_context_reference(SAMPLE_URL)?;
    assert_eq!(url.kind(), ContextReferenceKind::Url);
    assert!(url.safe_target_label().starts_with(super::super::base::URL_LABEL_PREFIX));
    assert!(!url.safe_target_label().contains("example.test"));
    Ok(())
}

#[test]
fn parser_rejects_malformed_and_unsafe_references() {
    assert!(matches!(parse_context_reference("src/lib.rs"), Err(ContextReferenceError::MissingSigil)));
    assert!(matches!(
        parse_context_reference(MALFORMED_RANGE),
        Err(ContextReferenceError::MalformedRange { .. })
    ));
    assert!(matches!(
        parse_context_reference(UNSUPPORTED_REFERENCE),
        Err(ContextReferenceError::UnsupportedKind { .. })
    ));
    assert!(matches!(parse_context_reference(ESCAPING_REFERENCE), Err(ContextReferenceError::PathEscape { .. })));
    assert!(matches!(
        parse_context_reference("@https://token@example.test/private"),
        Err(ContextReferenceError::UnsafeTarget { .. })
    ));
}
