use super::super::*;
use super::support::*;

#[test]
fn planning_denies_url_without_invoking_resolver() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_URL)?;
    let resolver = CountingUrlResolver::new(FILE_CONTENT);
    let resolvers = ContextResolverSet::empty().with_url(&resolver);

    let result = expand_context_reference(
        &reference,
        ContextReferencePolicy::allow_local(),
        &resolvers,
    );

    assert_eq!(result, Err(ContextReferenceError::ResolverDenied));
    assert_eq!(resolver.calls.get(), super::super::base::ZERO_COUNT);
    Ok(())
}

#[test]
fn planning_rejects_missing_resolver_without_fallback() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_FILE)?;
    let result = plan_context_resolution(
        &reference,
        ContextReferencePolicy::allow_all(),
        &ContextResolverSet::empty(),
    );

    assert_eq!(result, Err(ContextReferenceError::ResolverMissing));
    Ok(())
}

#[test]
fn expansion_uses_injected_resolver_when_allowed() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_URL)?;
    let resolver = CountingUrlResolver::new(SHORT_CONTENT);
    let resolvers = ContextResolverSet::empty().with_url(&resolver);

    let candidate =
        expand_context_reference(&reference, ContextReferencePolicy::allow_all(), &resolvers)?;

    assert_eq!(candidate.content(), SHORT_CONTENT);
    assert_eq!(resolver.calls.get(), super::super::base::MIN_LINE_NUMBER);
    Ok(())
}

#[test]
fn budget_truncates_oversized_content_with_safe_identity() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_FILE)?;
    let candidate =
        ResolvedContextCandidate::new(reference, ContextResolverKind::File, FILE_CONTENT);
    let budget = ContextReferenceBudget::new(
        MAX_VISIBLE_BYTES,
        MAX_VISIBLE_LINES,
        ContextOversizedPolicy::Truncate,
    )?;

    let fragment = apply_context_budget(candidate, budget)?;

    assert_eq!(fragment.status(), ContextReferenceStatus::Truncated);
    assert_eq!(fragment.truncation(), ContextTruncationState::Truncated);
    assert_eq!(fragment.visible_text(), Some("alpha\nbeta"));
    assert_eq!(fragment.line_count(), 3);
    assert_eq!(fragment.visible_line_count(), 2);
    assert_ne!(
        fragment.content_hash().as_str(),
        super::super::base::hash_text(super::super::base::EMPTY_HASH_INPUT).as_str()
    );
    Ok(())
}

#[test]
fn budget_rejects_oversized_content_when_policy_requires() -> Result<(), ContextReferenceError> {
    let reference = parse_context_reference(SAMPLE_FILE)?;
    let candidate =
        ResolvedContextCandidate::new(reference, ContextResolverKind::File, FILE_CONTENT);
    let budget = ContextReferenceBudget::new(
        MAX_VISIBLE_BYTES,
        MAX_VISIBLE_LINES,
        ContextOversizedPolicy::Reject,
    )?;

    let fragment = apply_context_budget(candidate, budget)?;

    assert_eq!(fragment.status(), ContextReferenceStatus::Rejected);
    assert_eq!(fragment.visible_text(), None);
    assert_eq!(
        fragment.visible_byte_count(),
        super::super::base::ZERO_COUNT
    );
    Ok(())
}
