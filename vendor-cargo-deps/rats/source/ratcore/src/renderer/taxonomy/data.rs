/// Built-in Subwayrat family identity.
pub const FAMILY_SUBWAYRAT: &str = "subwayrat";
/// Built-in Meteorite family identity.
pub const FAMILY_METEORITE: &str = "meteorite";
/// Experimental Iced pilot family identity.
pub const FAMILY_ICED: &str = "iced";
/// Maintained Subwayrat terminal target.
pub const TARGET_SUBWAYRAT_TERMINAL: &str = "subwayrat-terminal";
/// Maintained Subwayrat browser/Ratzilla target.
pub const TARGET_SUBWAYRAT_RATZILLA: &str = "subwayrat-ratzilla";
/// Maintained Meteorite DOM target.
pub const TARGET_METEORITE_DOM: &str = "meteorite-dom";
/// Maintained Meteorite host for an embedded Ratzilla surface.
pub const TARGET_METEORITE_EMBEDDED_RATZILLA: &str = "meteorite-embedded-ratzilla";
/// Explicit native-DOM column used by embedding parity.
pub const TARGET_METEORITE_NATIVE_DOM: &str = "meteorite-native-dom";
/// Experimental Iced native pilot target.
pub const TARGET_ICED_NATIVE: &str = "iced-native";
/// Built-in Subwayrat Typst visual evidence adapter.
pub const ADAPTER_SUBWAYRAT_TYPST: &str = "subwayrat-typst";
/// Built-in Meteorite DOM visual evidence adapter.
pub const ADAPTER_METEORITE_DOM: &str = "meteorite-dom";
/// Experimental Iced compile evidence adapter.
pub const ADAPTER_ICED_COMPILE: &str = "iced-compile";
/// Experimental Iced pure replay evidence adapter.
pub const ADAPTER_ICED_REPLAY: &str = "iced-replay";
/// Experimental Iced semantic projection evidence adapter.
pub const ADAPTER_ICED_SEMANTIC: &str = "iced-semantic";
/// Experimental Iced native render-smoke evidence adapter.
pub const ADAPTER_ICED_NATIVE_SMOKE: &str = "iced-native-smoke";

/// Named built-in renderer scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInRendererScope {
    /// Generic feature parity families.
    GeneralParityFamilies,
    /// Shared showcase fixture families.
    FixtureParityFamilies,
    /// Visual snapshot evidence adapters.
    SnapshotVisualAdapters,
    /// Full-mouse concrete targets.
    FullMouseTargets,
    /// Browser showcase concrete targets.
    BrowserTargets,
    /// Meteorite-hosted Ratzilla concrete targets.
    EmbeddedRatzillaTargets,
    /// Experimental Iced native target.
    IcedPilotTarget,
    /// Experimental Iced compile, replay, semantic, and native-smoke adapters.
    IcedPilotEvidenceAdapters,
}

/// Builds the deterministic maintained renderer registry.
#[must_use]
pub fn built_in_renderer_registry() -> super::RendererRegistry {
    super::RendererRegistry::new()
        .with_family(super::RendererFamilyDescriptor::new(
            FAMILY_SUBWAYRAT,
            "Subwayrat",
            super::RendererLifecycle::Maintained,
        ))
        .with_family(super::RendererFamilyDescriptor::new(
            FAMILY_METEORITE,
            "Meteorite",
            super::RendererLifecycle::Maintained,
        ))
        .with_family(super::RendererFamilyDescriptor::new(FAMILY_ICED, "Iced", super::RendererLifecycle::Experimental))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_SUBWAYRAT_TERMINAL,
            FAMILY_SUBWAYRAT,
            "Subwayrat terminal",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_SUBWAYRAT_RATZILLA,
            FAMILY_SUBWAYRAT,
            "Subwayrat Ratzilla",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_METEORITE_DOM,
            FAMILY_METEORITE,
            "Meteorite DOM",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_METEORITE_EMBEDDED_RATZILLA,
            FAMILY_METEORITE,
            "Meteorite embedded Ratzilla",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_METEORITE_NATIVE_DOM,
            FAMILY_METEORITE,
            "Meteorite native DOM",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            TARGET_ICED_NATIVE,
            FAMILY_ICED,
            "Iced native pilot",
            super::RendererLifecycle::Experimental,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_SUBWAYRAT_TYPST,
            TARGET_SUBWAYRAT_TERMINAL,
            super::EvidenceRole::VisualSnapshot,
            "Subwayrat Typst snapshot",
            super::RendererLifecycle::Maintained,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_METEORITE_DOM,
            TARGET_METEORITE_DOM,
            super::EvidenceRole::VisualSnapshot,
            "Meteorite DOM snapshot",
            super::RendererLifecycle::Maintained,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_ICED_COMPILE,
            TARGET_ICED_NATIVE,
            super::EvidenceRole::Compile,
            "Iced package compile receipt",
            super::RendererLifecycle::Experimental,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_ICED_REPLAY,
            TARGET_ICED_NATIVE,
            super::EvidenceRole::PureReplay,
            "Iced pure replay receipt",
            super::RendererLifecycle::Experimental,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_ICED_SEMANTIC,
            TARGET_ICED_NATIVE,
            super::EvidenceRole::SemanticProjection,
            "Iced semantic projection",
            super::RendererLifecycle::Experimental,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            ADAPTER_ICED_NATIVE_SMOKE,
            TARGET_ICED_NATIVE,
            super::EvidenceRole::NativeRenderSmoke,
            "Iced native render smoke",
            super::RendererLifecycle::Experimental,
        ))
}

/// Builds one ordered built-in renderer requirement scope.
#[must_use]
pub fn built_in_renderer_scope(scope: BuiltInRendererScope) -> super::RendererRequirementScope {
    match scope {
        BuiltInRendererScope::GeneralParityFamilies => family_scope("renderer-parity.default"),
        BuiltInRendererScope::FixtureParityFamilies => family_scope("showcase-fixtures.default"),
        BuiltInRendererScope::SnapshotVisualAdapters => adapter_scope(),
        BuiltInRendererScope::FullMouseTargets => full_mouse_scope(),
        BuiltInRendererScope::BrowserTargets => browser_scope(),
        BuiltInRendererScope::EmbeddedRatzillaTargets => embedded_scope(),
        BuiltInRendererScope::IcedPilotTarget => iced_target_scope(),
        BuiltInRendererScope::IcedPilotEvidenceAdapters => iced_adapter_scope(),
    }
}

fn family_scope(id: &'static str) -> super::RendererRequirementScope {
    super::RendererRequirementScope::new(
        id,
        super::RendererSubjectKind::Family,
        super::RendererScopeCompletion::Maintained,
    )
    .with_member(super::RendererSubject::Family(super::RendererFamilyId::new(FAMILY_SUBWAYRAT)))
    .with_member(super::RendererSubject::Family(super::RendererFamilyId::new(FAMILY_METEORITE)))
}

fn adapter_scope() -> super::RendererRequirementScope {
    super::RendererRequirementScope::new(
        "snapshot-docs.visual",
        super::RendererSubjectKind::EvidenceAdapter,
        super::RendererScopeCompletion::Maintained,
    )
    .with_member(super::RendererSubject::EvidenceAdapter(super::EvidenceAdapterId::new(ADAPTER_SUBWAYRAT_TYPST)))
    .with_member(super::RendererSubject::EvidenceAdapter(super::EvidenceAdapterId::new(ADAPTER_METEORITE_DOM)))
}

fn full_mouse_scope() -> super::RendererRequirementScope {
    target_scope("full-mouse.targets", &[
        TARGET_SUBWAYRAT_TERMINAL,
        TARGET_SUBWAYRAT_RATZILLA,
        TARGET_METEORITE_DOM,
    ])
}

fn browser_scope() -> super::RendererRequirementScope {
    target_scope("browser-showcase.targets", &[TARGET_SUBWAYRAT_RATZILLA, TARGET_METEORITE_DOM])
}

fn embedded_scope() -> super::RendererRequirementScope {
    target_scope("embedded-ratzilla.targets", &[
        TARGET_SUBWAYRAT_TERMINAL,
        TARGET_SUBWAYRAT_RATZILLA,
        TARGET_METEORITE_EMBEDDED_RATZILLA,
        TARGET_METEORITE_NATIVE_DOM,
    ])
}

fn iced_target_scope() -> super::RendererRequirementScope {
    super::RendererRequirementScope::new(
        "iced-pilot.target",
        super::RendererSubjectKind::Target,
        super::RendererScopeCompletion::Experimental,
    )
    .with_member(super::RendererSubject::Target(super::RendererTargetId::new(TARGET_ICED_NATIVE)))
}

fn iced_adapter_scope() -> super::RendererRequirementScope {
    let mut scope = super::RendererRequirementScope::new(
        "iced-pilot.evidence",
        super::RendererSubjectKind::EvidenceAdapter,
        super::RendererScopeCompletion::Experimental,
    );
    scope.members = [
        ADAPTER_ICED_COMPILE,
        ADAPTER_ICED_REPLAY,
        ADAPTER_ICED_SEMANTIC,
        ADAPTER_ICED_NATIVE_SMOKE,
    ]
    .into_iter()
    .map(|adapter| super::RendererSubject::EvidenceAdapter(super::EvidenceAdapterId::new(adapter)))
    .collect();
    scope
}

fn target_scope(id: &'static str, members: &[&str]) -> super::RendererRequirementScope {
    let mut scope = super::RendererRequirementScope::new(
        id,
        super::RendererSubjectKind::Target,
        super::RendererScopeCompletion::Maintained,
    );
    scope.members = members
        .iter()
        .map(|member| super::RendererSubject::Target(super::RendererTargetId::new(*member)))
        .collect();
    scope
}
