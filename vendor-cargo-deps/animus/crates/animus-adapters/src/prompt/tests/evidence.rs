use super::super::*;
use super::support::*;

#[test]
fn provenance_accepts_budgeted_safe_metadata() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_FILE)?;
    let candidate =
        ResolvedContextCandidate::new(reference, ContextResolverKind::File, SHORT_CONTENT);
    let budget = ContextReferenceBudget::new(
        MAX_VISIBLE_BYTES,
        MAX_VISIBLE_LINES,
        ContextOversizedPolicy::Truncate,
    )?;
    let fragment = apply_context_budget(candidate, budget)?;
    let draft = ContextReferenceProvenanceDraft::from_fragment(
        &fragment,
        ContextReferencePolicyDecision::Allowed,
    );

    let provenance = ContextReferenceProvenance::new(draft)?;

    assert!(provenance.review_summary().contains("safe_receipt"));
    assert!(provenance.review_summary().contains("payload_hash_only"));
    Ok(())
}

#[test]
fn provenance_rejects_raw_content_credentials_and_paths() {
    let hash = super::super::base::hash_text(SHORT_CONTENT);
    let raw_content = ContextReferenceProvenanceDraft::new(
        ContextReferenceKind::File,
        ContextReferenceStatus::Included,
        "tool_output: raw file body",
        ContextResolverKind::File,
    )
    .with_content_hash(&hash);
    assert!(matches!(
        ContextReferenceProvenance::new(raw_content),
        Err(ContextReferenceError::SafeMetadata(
            crate::safe::receipt::Error::UnsafeMetadata { .. }
        ))
    ));

    let credential = ContextReferenceProvenanceDraft::new(
        ContextReferenceKind::Url,
        ContextReferenceStatus::Denied,
        "token=secret",
        ContextResolverKind::Url,
    );
    assert!(matches!(
        ContextReferenceProvenance::new(credential),
        Err(ContextReferenceError::SafeMetadata(
            crate::safe::receipt::Error::UnsafeMetadata { .. }
        ))
    ));

    let path = ContextReferenceProvenanceDraft::new(
        ContextReferenceKind::File,
        ContextReferenceStatus::Included,
        "/home/user/secret.txt",
        ContextResolverKind::File,
    );
    assert!(matches!(
        ContextReferenceProvenance::new(path),
        Err(ContextReferenceError::SafeMetadata(
            crate::safe::receipt::Error::UnsafeMetadata { .. }
        ))
    ));
}

#[test]
fn provenance_rejects_oversized_labels_and_unbounded_output() {
    let overlong_label = ContextReferenceProvenanceDraft::new(
        ContextReferenceKind::File,
        ContextReferenceStatus::Included,
        OVERLONG_SAFE_LABEL,
        ContextResolverKind::File,
    );
    assert!(matches!(
        ContextReferenceProvenance::new(overlong_label),
        Err(ContextReferenceError::SafeMetadata(
            crate::safe::receipt::Error::UnsafeMetadata { .. }
        ))
    ));

    let unbounded = ContextReferenceProvenanceDraft::new(
        ContextReferenceKind::File,
        ContextReferenceStatus::Included,
        "file:src/lib.rs",
        ContextResolverKind::File,
    )
    .with_counts(ContextReferenceCounts::new(
        MAX_VISIBLE_BYTES,
        MAX_VISIBLE_LINES,
        super::super::base::MIN_LINE_NUMBER,
        super::super::base::MIN_LINE_NUMBER,
    ));
    assert!(matches!(
        ContextReferenceProvenance::new(unbounded),
        Err(ContextReferenceError::SafeMetadata(
            crate::safe::receipt::Error::UnsafeMetadata {
                class: crate::safe::receipt::LeakClass::UnboundedOutput,
                ..
            }
        ))
    ));
}
