/// Renderer label supported by shared showcase fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShowcaseRenderer {
    /// Subwayrat terminal renderer.
    Subwayrat,
    /// Meteorite web renderer.
    Meteorite,
}

impl ShowcaseRenderer {
    /// Stable renderer label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Subwayrat => crate::targeting_core::FAMILY_SUBWAYRAT,
            Self::Meteorite => crate::targeting_core::FAMILY_METEORITE,
        }
    }

    /// Typed family identity used by scoped validation.
    #[must_use]
    pub fn family_id(self) -> crate::targeting_core::RendererFamilyId {
        crate::targeting_core::RendererFamilyId::new(self.label())
    }
}

impl From<ShowcaseRenderer> for crate::targeting_core::RendererFamilyId {
    fn from(renderer: ShowcaseRenderer) -> Self {
        renderer.family_id()
    }
}

/// Renderer support metadata for one fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererSupport {
    /// Renderer label.
    pub renderer: ShowcaseRenderer,
    /// Whether the renderer supports this fixture.
    pub supported: bool,
    /// Optional reason when unsupported.
    pub reason: Option<String>,
}

impl RendererSupport {
    /// Marks a renderer as supported.
    #[must_use]
    pub fn supported(renderer: ShowcaseRenderer) -> Self {
        Self {
            renderer,
            supported: true,
            reason: None,
        }
    }

    /// Marks a renderer as unsupported with a reason.
    #[must_use]
    pub fn unsupported(renderer: ShowcaseRenderer, reason: impl Into<String>) -> Self {
        Self {
            renderer,
            supported: false,
            reason: Some(reason.into()),
        }
    }
}

/// Typed source-of-truth renderer support cell for one fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureRendererCell {
    /// Nominal renderer family identity.
    pub family: crate::targeting_core::RendererFamilyId,
    /// Whether this family supports the fixture.
    pub supported: bool,
    /// Required reason when unsupported.
    pub reason: Option<String>,
}

impl FixtureRendererCell {
    /// Marks a typed renderer family as supported.
    #[must_use]
    pub fn supported(family: impl Into<crate::targeting_core::RendererFamilyId>) -> Self {
        Self {
            family: family.into(),
            supported: true,
            reason: None,
        }
    }

    /// Marks a typed renderer family as unsupported.
    #[must_use]
    pub fn unsupported(family: impl Into<crate::targeting_core::RendererFamilyId>, reason: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            supported: false,
            reason: Some(reason.into()),
        }
    }
}

impl From<&RendererSupport> for FixtureRendererCell {
    fn from(support: &RendererSupport) -> Self {
        Self {
            family: support.renderer.family_id(),
            supported: support.supported,
            reason: support.reason.clone(),
        }
    }
}
