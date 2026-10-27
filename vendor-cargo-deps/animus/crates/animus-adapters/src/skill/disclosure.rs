/// Borrowed full-body content explicitly supplied by a host load decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BodyDraft<'a> {
    /// Skill name selected by the host.
    pub name: &'a str,
    /// Lowercase BLAKE3 identity for this body.
    pub content_identity: &'a str,
    /// Full body content supplied after explicit host approval.
    pub body: &'a str,
}

/// Loaded skill body selected for prompt construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedBody {
    name: super::Text,
    content_identity: crate::artifact::Hash,
    body: super::Text,
}

impl LoadedBody {
    /// Returns the loaded skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the body content identity.
    #[must_use]
    pub fn content_identity(&self) -> &crate::artifact::Hash {
        &self.content_identity
    }

    /// Returns the full body content selected by the host.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }
}

/// Inputs for skill progressive disclosure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    /// Validated skill manifests supplied by the host.
    pub manifests: &'a [super::Descriptor],
    /// Full bodies made available by an explicit host load action.
    pub loaded_bodies: &'a [BodyDraft<'a>],
    /// Optional selected skill name to load into prompt context.
    pub selected_name: Option<&'a str>,
}

/// Progressive skill disclosure context for prompt construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    catalog: super::Catalog,
    selected_body: Option<LoadedBody>,
}

impl Context {
    /// Returns resource summaries plus safe diagnostics.
    #[must_use]
    pub const fn catalog(&self) -> &super::Catalog {
        &self.catalog
    }

    /// Returns the selected full body, when a trusted matching body was loaded.
    #[must_use]
    pub const fn selected_body(&self) -> Option<&LoadedBody> {
        self.selected_body.as_ref()
    }
}

/// Builds progressive disclosure context without loading unselected bodies.
///
/// # Panics
///
/// Panics when callers pass more manifests or loaded body drafts than the
/// descriptor item bound.
#[must_use]
pub fn build_context(request: Request<'_>) -> Context {
    assert!(
        request.manifests.len() <= super::MAX_ITEMS,
        "manifest inputs stay within the descriptor item bound"
    );
    assert!(
        request.loaded_bodies.len() <= super::MAX_ITEMS,
        "loaded body inputs stay within the descriptor item bound"
    );
    let catalog = super::advertise(request.manifests);
    let selected_body = request.selected_name.and_then(|selected_name| {
        select_loaded_body(request.manifests, request.loaded_bodies, selected_name)
    });

    Context {
        catalog,
        selected_body,
    }
}

fn select_loaded_body(
    manifests: &[super::Descriptor],
    bodies: &[BodyDraft<'_>],
    selected_name: &str,
) -> Option<LoadedBody> {
    assert!(
        manifests.len() <= super::MAX_ITEMS,
        "manifest inputs stay within the descriptor item bound"
    );
    assert!(
        bodies.len() <= super::MAX_ITEMS,
        "loaded body inputs stay within the descriptor item bound"
    );
    let manifest = manifests.iter().find(|item| item.name() == selected_name)?;
    assert_eq!(
        manifest.name(),
        selected_name,
        "selected descriptor matches the requested skill name"
    );
    if manifest.trust_status() != super::TrustStatus::Trusted {
        return None;
    }
    if super::resource::duplicate_name(manifest.name(), manifests) {
        return None;
    }
    let body = bodies.iter().find(|candidate| {
        candidate.name == manifest.name()
            && candidate.content_identity == manifest.content_identity().as_str()
    })?;
    assert_eq!(
        body.name,
        manifest.name(),
        "loaded body matches selected skill"
    );
    assert_eq!(
        body.content_identity,
        manifest.content_identity().as_str(),
        "loaded body identity matches selected descriptor"
    );
    let validated_body =
        super::validation::required_text(super::BODY_FIELD, Some(body.body)).ok()?;
    assert!(!validated_body.is_empty(), "loaded body is non-empty");
    Some(LoadedBody {
        name: manifest.name.clone(),
        content_identity: manifest.content_identity.clone(),
        body: validated_body,
    })
}
