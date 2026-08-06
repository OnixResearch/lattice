/// Renderer family registry descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererFamilyDescriptor {
    /// Stable family identity.
    pub id: super::RendererFamilyId,
    /// Human-readable family label.
    pub label: String,
    /// Roadmap lifecycle.
    pub lifecycle: super::RendererLifecycle,
    /// Required retirement or lifecycle reason when applicable.
    pub reason: Option<String>,
    /// Optional replacement family.
    pub replacement: Option<super::RendererFamilyId>,
}

impl RendererFamilyDescriptor {
    /// Creates a renderer family descriptor.
    #[must_use]
    pub fn new(
        id: impl Into<super::RendererFamilyId>,
        label: impl Into<String>,
        lifecycle: super::RendererLifecycle,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            lifecycle,
            reason: None,
            replacement: None,
        }
    }

    /// Adds a lifecycle or retirement reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds a replacement family.
    #[must_use]
    pub fn with_replacement(mut self, replacement: impl Into<super::RendererFamilyId>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// Concrete renderer target registry descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererTargetDescriptor {
    /// Stable target identity.
    pub id: super::RendererTargetId,
    /// Parent renderer family.
    pub family: super::RendererFamilyId,
    /// Human-readable target label.
    pub label: String,
    /// Roadmap lifecycle.
    pub lifecycle: super::RendererLifecycle,
    /// Required retirement or lifecycle reason when applicable.
    pub reason: Option<String>,
    /// Optional replacement target.
    pub replacement: Option<super::RendererTargetId>,
}

impl RendererTargetDescriptor {
    /// Creates a concrete renderer target descriptor.
    #[must_use]
    pub fn new(
        id: impl Into<super::RendererTargetId>,
        family: impl Into<super::RendererFamilyId>,
        label: impl Into<String>,
        lifecycle: super::RendererLifecycle,
    ) -> Self {
        Self {
            id: id.into(),
            family: family.into(),
            label: label.into(),
            lifecycle,
            reason: None,
            replacement: None,
        }
    }

    /// Adds a lifecycle or retirement reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds a replacement target.
    #[must_use]
    pub fn with_replacement(mut self, replacement: impl Into<super::RendererTargetId>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// Evidence class produced by a registered adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceRole {
    /// Package or binary compilation evidence without runtime claims.
    Compile,
    /// Pure interaction replay against an authoritative shared reducer.
    PureReplay,
    /// Pure semantic projection that is not visual/platform runtime evidence.
    SemanticProjection,
    /// Interaction or route summary that is not a renderer snapshot.
    RouteSummary,
    /// Native window/backend initialization and render-smoke evidence.
    NativeRenderSmoke,
    /// Visual renderer snapshot accepted by snapshot documentation gates.
    VisualSnapshot,
}

/// Renderer evidence-adapter registry descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAdapterDescriptor {
    /// Stable adapter identity.
    pub id: super::EvidenceAdapterId,
    /// Concrete target whose output is adapted.
    pub target: super::RendererTargetId,
    /// Evidence class produced by the adapter.
    pub role: EvidenceRole,
    /// Human-readable adapter label.
    pub label: String,
    /// Roadmap lifecycle.
    pub lifecycle: super::RendererLifecycle,
    /// Required retirement or lifecycle reason when applicable.
    pub reason: Option<String>,
    /// Optional replacement adapter.
    pub replacement: Option<super::EvidenceAdapterId>,
}

impl EvidenceAdapterDescriptor {
    /// Creates an evidence adapter descriptor.
    #[must_use]
    pub fn new(
        id: impl Into<super::EvidenceAdapterId>,
        target: impl Into<super::RendererTargetId>,
        role: EvidenceRole,
        label: impl Into<String>,
        lifecycle: super::RendererLifecycle,
    ) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            role,
            label: label.into(),
            lifecycle,
            reason: None,
            replacement: None,
        }
    }

    /// Adds a lifecycle or retirement reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds a replacement adapter.
    #[must_use]
    pub fn with_replacement(mut self, replacement: impl Into<super::EvidenceAdapterId>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// Pure renderer registry data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RendererRegistry {
    /// Registered component families.
    pub families: Vec<RendererFamilyDescriptor>,
    /// Registered concrete targets.
    pub targets: Vec<RendererTargetDescriptor>,
    /// Registered evidence adapters.
    pub evidence_adapters: Vec<EvidenceAdapterDescriptor>,
}

impl RendererRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            families: Vec::new(),
            targets: Vec::new(),
            evidence_adapters: Vec::new(),
        }
    }

    /// Adds a family descriptor.
    #[must_use]
    pub fn with_family(mut self, family: RendererFamilyDescriptor) -> Self {
        self.families.push(family);
        self
    }

    /// Adds a concrete target descriptor.
    #[must_use]
    pub fn with_target(mut self, target: RendererTargetDescriptor) -> Self {
        self.targets.push(target);
        self
    }

    /// Adds an evidence adapter descriptor.
    #[must_use]
    pub fn with_evidence_adapter(mut self, adapter: EvidenceAdapterDescriptor) -> Self {
        self.evidence_adapters.push(adapter);
        self
    }

    /// Returns a family descriptor by nominal identity.
    #[must_use]
    pub fn family(&self, id: &super::RendererFamilyId) -> Option<&RendererFamilyDescriptor> {
        self.families.iter().find(|family| family.id == *id)
    }

    /// Returns a concrete target descriptor by nominal identity.
    #[must_use]
    pub fn target(&self, id: &super::RendererTargetId) -> Option<&RendererTargetDescriptor> {
        self.targets.iter().find(|target| target.id == *id)
    }

    /// Returns an evidence adapter descriptor by nominal identity.
    #[must_use]
    pub fn evidence_adapter(&self, id: &super::EvidenceAdapterId) -> Option<&EvidenceAdapterDescriptor> {
        self.evidence_adapters.iter().find(|adapter| adapter.id == *id)
    }

    /// Resolves one visual or semantic adapter for a family and role.
    #[must_use]
    pub fn evidence_adapter_for_family(
        &self,
        family: &super::RendererFamilyId,
        role: EvidenceRole,
    ) -> Option<&EvidenceAdapterDescriptor> {
        self.evidence_adapters.iter().find(|adapter| {
            adapter.role == role && self.target(&adapter.target).is_some_and(|target| target.family == *family)
        })
    }
}
