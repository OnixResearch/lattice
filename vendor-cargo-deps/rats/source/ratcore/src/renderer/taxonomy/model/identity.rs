semantic_string_id! {
    /// Stable renderer component-family identity.
    pub struct RendererFamilyId;
}

semantic_string_id! {
    /// Stable concrete renderer runtime-target identity.
    pub struct RendererTargetId;
}

semantic_string_id! {
    /// Stable renderer evidence-adapter identity.
    pub struct EvidenceAdapterId;
}

semantic_string_id! {
    /// Stable renderer requirement-scope identity.
    pub struct RendererScopeId;
}

/// Renderer roadmap lifecycle used by validation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererLifecycle {
    /// Visible only in scopes that explicitly opt into experimental coverage.
    Experimental,
    /// Required where a maintained scope names the renderer.
    Maintained,
    /// No longer accepted as a required renderer.
    Retired,
}

/// Kind of renderer identity accepted by a requirement scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererSubjectKind {
    /// Component-family identity.
    Family,
    /// Concrete runtime-target identity.
    Target,
    /// Evidence-adapter identity.
    EvidenceAdapter,
}

/// Nominal renderer identity retained in policy cells.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererSubject {
    /// Component-family identity.
    Family(RendererFamilyId),
    /// Concrete runtime-target identity.
    Target(RendererTargetId),
    /// Evidence-adapter identity.
    EvidenceAdapter(EvidenceAdapterId),
}

impl RendererSubject {
    /// Returns the nominal identity kind.
    #[must_use]
    pub const fn kind(&self) -> RendererSubjectKind {
        match self {
            Self::Family(_) => RendererSubjectKind::Family,
            Self::Target(_) => RendererSubjectKind::Target,
            Self::EvidenceAdapter(_) => RendererSubjectKind::EvidenceAdapter,
        }
    }

    /// Returns the stable identity text without erasing its nominal kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Family(id) => id.as_str(),
            Self::Target(id) => id.as_str(),
            Self::EvidenceAdapter(id) => id.as_str(),
        }
    }
}
