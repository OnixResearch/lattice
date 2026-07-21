use super::{
    BODY_FIELD, Catalog, Collection, ContentDraft, Context, DIAGNOSTIC_SLOTS_PER_DESCRIPTOR,
    Descriptor, Diagnostic, Issue, LoadedContent, MAX_DESCRIPTORS, Request, Summary, Text,
    TrustStatus, validation,
};

/// Builds deterministic resource summaries from already validated descriptors.
///
/// # Panics
///
/// Panics when callers pass more descriptors than the descriptor item bound.
#[must_use]
pub fn advertise(descriptors: &[Descriptor]) -> Catalog {
    let descriptor_count = descriptors.len();
    assert!(
        descriptor_count <= MAX_DESCRIPTORS,
        "descriptor inputs stay within the resource item bound"
    );
    let diagnostic_slots_count = descriptor_count.saturating_mul(DIAGNOSTIC_SLOTS_PER_DESCRIPTOR);
    assert!(
        diagnostic_slots_count >= descriptor_count,
        "diagnostic capacity covers at least one issue per descriptor"
    );
    let mut summaries = Collection::with_capacity(descriptor_count);
    let mut diagnostics = Collection::with_capacity(diagnostic_slots_count);

    for descriptor in descriptors {
        if duplicate_name(descriptor.name(), descriptors) {
            push_diagnostic(descriptor, Issue::DuplicateName, &mut diagnostics);
            continue;
        }
        if denied_by_trust(descriptor) {
            push_diagnostic(descriptor, Issue::UntrustedDescriptor, &mut diagnostics);
            continue;
        }
        summaries.push(descriptor.summary());
    }

    sort_summaries(&mut summaries);
    sort_diagnostics(&mut diagnostics);
    Catalog {
        summaries,
        diagnostics,
    }
}

/// Builds progressive disclosure context without loading unselected bodies.
///
/// # Panics
///
/// Panics when callers pass more descriptors or loaded bodies than the descriptor
/// item bound.
#[must_use]
pub fn build_context(request: Request<'_>) -> Context {
    assert!(
        request.descriptors.len() <= MAX_DESCRIPTORS,
        "descriptor inputs stay within the resource item bound"
    );
    assert!(
        request.loaded_contents.len() <= MAX_DESCRIPTORS,
        "loaded body inputs stay within the resource item bound"
    );
    let catalog = advertise(request.descriptors);
    let selected_content = request.selected_name.and_then(|selected_name| {
        select_loaded_content(request.descriptors, request.loaded_contents, selected_name)
    });

    Context {
        catalog,
        selected_content,
    }
}

fn select_loaded_content(
    descriptors: &[Descriptor],
    bodies: &[ContentDraft<'_>],
    selected_name: &str,
) -> Option<LoadedContent> {
    assert!(
        descriptors.len() <= MAX_DESCRIPTORS,
        "descriptor inputs stay within the resource item bound"
    );
    assert!(
        bodies.len() <= MAX_DESCRIPTORS,
        "loaded body inputs stay within the resource item bound"
    );
    let descriptor = descriptors
        .iter()
        .find(|item| item.name() == selected_name)?;
    assert_eq!(
        descriptor.name(),
        selected_name,
        "selected descriptor matches the requested resource name"
    );
    if duplicate_name(descriptor.name(), descriptors) || denied_by_trust(descriptor) {
        return None;
    }
    let body = bodies.iter().find(|candidate| {
        candidate.name == descriptor.name()
            && candidate.content_identity == descriptor.content_identity().as_str()
    })?;
    assert_eq!(
        body.name,
        descriptor.name(),
        "loaded body matches selected resource"
    );
    assert_eq!(
        body.content_identity,
        descriptor.content_identity().as_str(),
        "loaded body identity matches selected descriptor"
    );
    let validated_body = validation::required_text(BODY_FIELD, Some(body.body)).ok()?;
    assert!(!validated_body.is_empty(), "loaded body is non-empty");
    Some(LoadedContent {
        name: descriptor.name.clone(),
        kind: descriptor.kind,
        content_identity: descriptor.content_identity.clone(),
        body: validated_body,
    })
}

fn duplicate_name(name: &str, descriptors: &[Descriptor]) -> bool {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.name() == name)
        .count()
        > 1
}

fn denied_by_trust(descriptor: &Descriptor) -> bool {
    descriptor.trust_status() == TrustStatus::Untrusted
}

fn push_diagnostic(
    descriptor: &Descriptor,
    issue: Issue,
    diagnostics: &mut Collection<Diagnostic>,
) {
    if diagnostics.iter().any(|diagnostic| {
        diagnostic.issue == issue
            && diagnostic.name == descriptor.name()
            && diagnostic.kind == descriptor.kind()
            && diagnostic.scope == descriptor.scope()
    }) {
        return;
    }
    diagnostics.push(Diagnostic {
        issue,
        name: Text::from(descriptor.name()),
        kind: descriptor.kind(),
        scope: descriptor.scope(),
    });
}

fn sort_summaries(summaries: &mut [Summary]) {
    summaries.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.kind.cmp(&right.kind))
            .then(left.scope.cmp(&right.scope))
    });
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.kind.cmp(&right.kind))
            .then(left.scope.cmp(&right.scope))
    });
}
