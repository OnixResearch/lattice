type Text = super::Text;
type Collection<T> = super::Collection<T>;

/// Model-visible resource summary that omits full content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary {
    pub(super) kind: super::Kind,
    pub(super) name: Text,
    pub(super) description: Text,
    pub(super) scope: super::Scope,
    pub(super) provenance: Text,
}

impl Summary {
    /// Returns the resource kind.
    #[must_use]
    pub const fn kind(&self) -> super::Kind {
        self.kind
    }

    /// Returns the resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe human-facing summary.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn scope(&self) -> super::Scope {
        self.scope
    }

    /// Returns the safe provenance label.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }
}

/// Resource catalog issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// More than one descriptor used the same name.
    DuplicateName,
    /// Descriptor was withheld because host trust did not admit it.
    UntrustedDescriptor,
}

/// Safe diagnostic emitted while building resource summaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub(super) issue: Issue,
    pub(super) name: Text,
    pub(super) kind: super::Kind,
    pub(super) scope: super::Scope,
}

impl Diagnostic {
    /// Returns the diagnostic issue class.
    #[must_use]
    pub const fn issue(&self) -> Issue {
        self.issue
    }

    /// Returns the descriptor name associated with the issue.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the descriptor kind associated with the issue.
    #[must_use]
    pub const fn kind(&self) -> super::Kind {
        self.kind
    }

    /// Returns the descriptor scope associated with the issue.
    #[must_use]
    pub const fn scope(&self) -> super::Scope {
        self.scope
    }
}

/// Deterministic prompt resource catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub(super) summaries: Collection<Summary>,
    pub(super) diagnostics: Collection<Diagnostic>,
}

impl Catalog {
    /// Returns deterministic summaries admitted for prompt/resource lookup.
    #[must_use]
    pub fn summaries(&self) -> &[Summary] {
        self.summaries.as_slice()
    }

    /// Returns deterministic safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Borrowed full-content payload explicitly supplied by a host load decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentDraft<'a> {
    /// Resource name selected by the host.
    pub name: &'a str,
    /// Lowercase BLAKE3 identity for this content.
    pub content_identity: &'a str,
    /// Full content supplied after explicit host approval.
    pub body: &'a str,
}

/// Loaded resource content selected for prompt construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedContent {
    pub(super) name: Text,
    pub(super) kind: super::Kind,
    pub(super) content_identity: crate::artifact::Hash,
    pub(super) body: Text,
}

impl LoadedContent {
    /// Returns the loaded resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the loaded resource kind.
    #[must_use]
    pub const fn kind(&self) -> super::Kind {
        self.kind
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

/// Inputs for prompt resource progressive disclosure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    /// Validated descriptors supplied by the host.
    pub descriptors: &'a [super::Descriptor],
    /// Full bodies made available by an explicit host load action.
    pub loaded_contents: &'a [ContentDraft<'a>],
    /// Optional selected resource name to load into prompt context.
    pub selected_name: Option<&'a str>,
}

/// Progressive prompt resource context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    pub(super) catalog: Catalog,
    pub(super) selected_content: Option<LoadedContent>,
}

impl Context {
    /// Returns resource summaries plus safe diagnostics.
    #[must_use]
    pub const fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Returns the selected full content, when a trusted matching body was loaded.
    #[must_use]
    pub const fn selected_content(&self) -> Option<&LoadedContent> {
        self.selected_content.as_ref()
    }
}
