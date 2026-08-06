use super::data::ContextReference;
use super::data::ContextReferenceError;
use super::data::ResolvedContextCandidate;

/// Host-owned resolver for file references.
pub trait FileContextResolver {
    /// Resolves a file reference without fallback authority.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when the host denies, cannot service, or
    /// cannot safely describe the requested file reference.
    fn resolve_file(&self, reference: &ContextReference) -> Result<ResolvedContextCandidate, ContextReferenceError>;
}

/// Host-owned resolver for directory references.
pub trait DirectoryContextResolver {
    /// Resolves a directory reference without fallback authority.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when the host denies, cannot service, or
    /// cannot safely describe the requested directory reference.
    fn resolve_directory(
        &self,
        reference: &ContextReference,
    ) -> Result<ResolvedContextCandidate, ContextReferenceError>;
}

/// Host-owned resolver for diff references.
pub trait DiffContextResolver {
    /// Resolves a diff reference without fallback authority.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when the host denies, cannot service, or
    /// cannot safely describe the requested diff reference.
    fn resolve_diff(&self, reference: &ContextReference) -> Result<ResolvedContextCandidate, ContextReferenceError>;
}

/// Host-owned resolver for image references.
pub trait ImageContextResolver {
    /// Resolves an image reference without fallback authority.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when the host denies, cannot service, or
    /// cannot safely describe the requested image reference.
    fn resolve_image(&self, reference: &ContextReference) -> Result<ResolvedContextCandidate, ContextReferenceError>;
}

/// Host-owned resolver for URL references.
pub trait UrlContextResolver {
    /// Resolves a URL reference without fallback authority.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when the host denies, cannot service, or
    /// cannot safely describe the requested URL reference.
    fn resolve_url(&self, reference: &ContextReference) -> Result<ResolvedContextCandidate, ContextReferenceError>;
}

/// Optional host resolver set used by the imperative dispatch shell.
#[derive(Clone, Copy, Default)]
pub struct ContextResolverSet<'a> {
    pub(super) file: Option<&'a dyn FileContextResolver>,
    pub(super) directory: Option<&'a dyn DirectoryContextResolver>,
    pub(super) diff: Option<&'a dyn DiffContextResolver>,
    pub(super) image: Option<&'a dyn ImageContextResolver>,
    pub(super) url: Option<&'a dyn UrlContextResolver>,
}

impl<'a> ContextResolverSet<'a> {
    /// Builds an empty resolver set.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            file: None,
            directory: None,
            diff: None,
            image: None,
            url: None,
        }
    }

    /// Returns a copy with a file resolver installed.
    #[must_use]
    pub const fn with_file(mut self, resolver: &'a dyn FileContextResolver) -> Self {
        self.file = Some(resolver);
        self
    }

    /// Returns a copy with a directory resolver installed.
    #[must_use]
    pub const fn with_directory(mut self, resolver: &'a dyn DirectoryContextResolver) -> Self {
        self.directory = Some(resolver);
        self
    }

    /// Returns a copy with a diff resolver installed.
    #[must_use]
    pub const fn with_diff(mut self, resolver: &'a dyn DiffContextResolver) -> Self {
        self.diff = Some(resolver);
        self
    }

    /// Returns a copy with an image resolver installed.
    #[must_use]
    pub const fn with_image(mut self, resolver: &'a dyn ImageContextResolver) -> Self {
        self.image = Some(resolver);
        self
    }

    /// Returns a copy with a URL resolver installed.
    #[must_use]
    pub const fn with_url(mut self, resolver: &'a dyn UrlContextResolver) -> Self {
        self.url = Some(resolver);
        self
    }
}
