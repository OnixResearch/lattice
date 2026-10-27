use super::base::{
    MIN_LINE_COUNT_USIZE, MIN_LINE_NUMBER, ZERO_COUNT, ZERO_COUNT_USIZE, hash_text, usize_to_u32,
};
use super::data::{
    ContextOversizedPolicy, ContextReference, ContextReferenceBudget, ContextReferenceError,
    ContextReferenceFragment, ContextReferenceKind, ContextReferencePolicy,
    ContextReferencePolicyDecision, ContextReferenceStatus, ContextResolutionPlan,
    ContextResolverKind, ContextTruncationState, ResolvedContextCandidate,
};
use super::host::ContextResolverSet;
/// Plans resolver dispatch without invoking host authority.
///
/// # Errors
///
/// Returns [`ContextReferenceError`] when policy denies the kind or the required
/// resolver service is absent.
pub fn plan_context_resolution(
    reference: &ContextReference,
    policy: ContextReferencePolicy,
    resolvers: &ContextResolverSet<'_>,
) -> Result<ContextResolutionPlan, ContextReferenceError> {
    let resolver_kind = resolver_kind_for(reference.kind);
    if !policy.allows(reference.kind) {
        return Err(ContextReferenceError::ResolverDenied);
    }
    if !resolver_available(reference.kind, resolvers) {
        return Err(ContextReferenceError::ResolverMissing);
    }
    Ok(ContextResolutionPlan {
        resolver_kind,
        decision: ContextReferencePolicyDecision::Allowed,
    })
}

/// Expands a reference through exactly one host-injected resolver.
///
/// # Errors
///
/// Returns [`ContextReferenceError`] when planning fails or the selected host
/// resolver denies, is unavailable, or returns an invalid candidate.
///
/// # Panics
///
/// Panics if callers bypass context-reference constructors and pass empty labels.
pub fn expand_context_reference(
    reference: &ContextReference,
    policy: ContextReferencePolicy,
    resolvers: &ContextResolverSet<'_>,
) -> Result<ResolvedContextCandidate, ContextReferenceError> {
    assert!(
        !reference.safe_target_label().is_empty(),
        "context reference labels must be validated"
    );
    assert!(
        !reference.normalized_target().is_empty(),
        "context reference targets must be validated"
    );
    let plan = plan_context_resolution(reference, policy, resolvers)?;
    match plan.resolver_kind {
        ContextResolverKind::File => resolvers
            .file
            .ok_or(ContextReferenceError::ResolverMissing)?
            .resolve_file(reference),
        ContextResolverKind::Directory => resolvers
            .directory
            .ok_or(ContextReferenceError::ResolverMissing)?
            .resolve_directory(reference),
        ContextResolverKind::Diff => resolvers
            .diff
            .ok_or(ContextReferenceError::ResolverMissing)?
            .resolve_diff(reference),
        ContextResolverKind::Image => resolvers
            .image
            .ok_or(ContextReferenceError::ResolverMissing)?
            .resolve_image(reference),
        ContextResolverKind::Url => resolvers
            .url
            .ok_or(ContextReferenceError::ResolverMissing)?
            .resolve_url(reference),
    }
}

/// Applies deterministic visible-context bounds to a resolved candidate.
///
/// # Errors
///
/// Returns [`ContextReferenceError`] when content counts overflow the public DTO
/// bounds.
///
/// # Panics
///
/// Panics if callers bypass budget constructors and pass zero visible bounds.
pub fn apply_context_budget(
    candidate: ResolvedContextCandidate,
    budget: ContextReferenceBudget,
) -> Result<ContextReferenceFragment, ContextReferenceError> {
    assert!(
        budget.max_visible_bytes > ZERO_COUNT,
        "visible byte budget must be validated"
    );
    assert!(
        budget.max_visible_lines > ZERO_COUNT,
        "visible line budget must be validated"
    );
    let byte_count = usize_to_u32(candidate.content.len())?;
    let original_line_count = usize_to_u32(count_lines(candidate.content.as_str()))?;
    let content_hash = hash_text(candidate.content.as_str());
    let is_over_budget =
        byte_count > budget.max_visible_bytes || original_line_count > budget.max_visible_lines;
    if !is_over_budget {
        return Ok(ContextReferenceFragment {
            reference: candidate.reference,
            resolver_kind: candidate.resolver_kind,
            status: ContextReferenceStatus::Included,
            truncation: ContextTruncationState::None,
            visible_text: Some(candidate.content),
            byte_count,
            line_count: original_line_count,
            visible_byte_count: byte_count,
            visible_line_count: original_line_count,
            content_hash,
        });
    }
    match budget.oversized_policy {
        ContextOversizedPolicy::Reject => Ok(ContextReferenceFragment {
            reference: candidate.reference,
            resolver_kind: candidate.resolver_kind,
            status: ContextReferenceStatus::Rejected,
            truncation: ContextTruncationState::Rejected,
            visible_text: None,
            byte_count,
            line_count: original_line_count,
            visible_byte_count: ZERO_COUNT,
            visible_line_count: ZERO_COUNT,
            content_hash,
        }),
        ContextOversizedPolicy::Truncate => {
            let visible = truncate_content(
                candidate.content.as_str(),
                TruncationLimit::new(budget.max_visible_bytes, budget.max_visible_lines),
            )?;
            let visible_byte_count = usize_to_u32(visible.len())?;
            let visible_line_count = usize_to_u32(count_lines(visible.as_str()))?;
            Ok(ContextReferenceFragment {
                reference: candidate.reference,
                resolver_kind: candidate.resolver_kind,
                status: ContextReferenceStatus::Truncated,
                truncation: ContextTruncationState::Truncated,
                visible_text: Some(visible),
                byte_count,
                line_count: original_line_count,
                visible_byte_count,
                visible_line_count,
                content_hash,
            })
        }
    }
}
fn resolver_kind_for(kind: ContextReferenceKind) -> ContextResolverKind {
    match kind {
        ContextReferenceKind::File => ContextResolverKind::File,
        ContextReferenceKind::Directory => ContextResolverKind::Directory,
        ContextReferenceKind::Diff => ContextResolverKind::Diff,
        ContextReferenceKind::Image => ContextResolverKind::Image,
        ContextReferenceKind::Url => ContextResolverKind::Url,
    }
}

fn resolver_available(kind: ContextReferenceKind, resolvers: &ContextResolverSet<'_>) -> bool {
    match kind {
        ContextReferenceKind::File => resolvers.file.is_some(),
        ContextReferenceKind::Directory => resolvers.directory.is_some(),
        ContextReferenceKind::Diff => resolvers.diff.is_some(),
        ContextReferenceKind::Image => resolvers.image.is_some(),
        ContextReferenceKind::Url => resolvers.url.is_some(),
    }
}

#[derive(Clone, Copy)]
struct TruncationLimit {
    max_bytes: u32,
    max_lines: u32,
}

impl TruncationLimit {
    const fn new(max_bytes: u32, max_lines: u32) -> Self {
        Self {
            max_bytes,
            max_lines,
        }
    }
}

fn truncate_content(
    text: &str,
    limit: TruncationLimit,
) -> Result<alloc::string::String, ContextReferenceError> {
    assert!(limit.max_bytes > ZERO_COUNT, "byte limit must be positive");
    assert!(limit.max_lines > ZERO_COUNT, "line limit must be positive");
    let max_bytes =
        usize::try_from(limit.max_bytes).map_err(|_error| ContextReferenceError::CountOverflow)?;
    let mut visible = alloc::string::String::new();
    let mut current_line = MIN_LINE_NUMBER;
    for character in text.chars() {
        if character == '\n' && current_line >= limit.max_lines {
            break;
        }
        if visible.len().saturating_add(character.len_utf8()) > max_bytes {
            break;
        }
        visible.push(character);
        if character == '\n' {
            current_line = current_line.saturating_add(MIN_LINE_NUMBER);
        }
    }
    Ok(visible)
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return ZERO_COUNT_USIZE;
    }
    let newline_count = text.chars().filter(|character| *character == '\n').count();
    if text.ends_with('\n') {
        newline_count
    } else {
        newline_count.saturating_add(MIN_LINE_COUNT_USIZE)
    }
}
