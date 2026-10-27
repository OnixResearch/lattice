// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const BODY: &str = "Use project-safe typed context only.";
const DESCRIPTOR_ID: &str = "ctx.project";
const HIGH_PRECEDENCE: u32 = 20;
const LOW_PRECEDENCE: u32 = 10;
const MAX_BODY_CHARS: usize = 128;
const SECTION: &str = "guidance";
const SOURCE_ID: &str = "source:project-agents";

#[test]
fn descriptors_validate_known_classes_and_reject_unknown_sources() {
    // r[agent_harness_runtime.zerostack_context_layering.classes.unknown]
    assert!(validate_context_descriptor(&descriptor(DESCRIPTOR_ID, LOW_PRECEDENCE)).is_ok());

    let invalid = ContextLayerDescriptor {
        class: ContextLayerClass::Unknown,
        source_identity: String::new(),
        ..descriptor("ctx.unknown", LOW_PRECEDENCE)
    };
    let diagnostics = validate_context_descriptor(&invalid).expect_err("invalid descriptor");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::UnknownClass })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::MissingSource })
    );
}

#[test]
fn prompt_assembly_uses_only_in_memory_descriptors() {
    // r[agent_harness_runtime.zerostack_context_layering.discovery.no_io]
    let plan = assemble_context_layers(&policy(), &[descriptor(DESCRIPTOR_ID, LOW_PRECEDENCE)]);

    assert_eq!(
        plan.discovery_boundary.ancestor_walk,
        ContextLayerBoundaryEffect::NotPerformed
    );
    assert_eq!(
        plan.discovery_boundary.file_read,
        ContextLayerBoundaryEffect::NotPerformed
    );
    assert_eq!(
        plan.discovery_boundary.environment_read,
        ContextLayerBoundaryEffect::NotPerformed
    );
    assert_eq!(
        plan.discovery_boundary.path_canonicalization,
        ContextLayerBoundaryEffect::NotPerformed
    );
    assert_eq!(plan.prompt_sections.len(), 1);
}

#[test]
fn admission_rejects_secret_oversized_stale_disabled_and_untrusted_context() {
    // r[agent_harness_runtime.zerostack_context_layering.admission.secret]
    let cases = vec![
        ContextLayerDescriptor {
            body: "API_KEY provider_payload".to_string(),
            size_chars: "API_KEY provider_payload".chars().count(),
            ..descriptor("ctx.secret", LOW_PRECEDENCE)
        },
        ContextLayerDescriptor {
            size_chars: MAX_BODY_CHARS + 1,
            body: "x".repeat(MAX_BODY_CHARS + 1),
            ..descriptor("ctx.large", LOW_PRECEDENCE)
        },
        ContextLayerDescriptor {
            freshness: ContextLayerFreshness::Stale,
            ..descriptor("ctx.stale", LOW_PRECEDENCE)
        },
        ContextLayerDescriptor {
            inclusion: ContextLayerInclusionPolicy::Disabled,
            ..descriptor("ctx.disabled", LOW_PRECEDENCE)
        },
        ContextLayerDescriptor {
            trust: ContextLayerTrust::Untrusted,
            ..descriptor("ctx.untrusted", LOW_PRECEDENCE)
        },
    ];
    let plan = assemble_context_layers(&policy(), &cases);

    assert!(plan.prompt_sections.is_empty());
    assert_eq!(plan.receipt.omitted.len(), cases.len());
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::Unsafe })
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::Oversized })
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::Stale })
    );
}

#[test]
fn precedence_conflicts_select_deterministic_winner_and_record_omissions() {
    // r[agent_harness_runtime.zerostack_context_layering.precedence.conflict]
    let low = descriptor("ctx.low", LOW_PRECEDENCE);
    let high = descriptor("ctx.high", HIGH_PRECEDENCE);
    let plan = assemble_context_layers(&policy(), &[low, high.clone()]);

    assert_eq!(plan.prompt_sections.len(), 1);
    assert_eq!(plan.prompt_sections[0].descriptor_id, high.descriptor_id);
    assert_eq!(plan.receipt.omitted.len(), 1);
    assert_eq!(plan.receipt.omitted[0].reason, "lower-precedence-conflict");
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ContextLayerDiagnosticKind::Conflict })
    );
}

#[test]
fn receipts_record_hashes_and_safe_omission_reasons() {
    // r[agent_harness_runtime.zerostack_context_layering.receipts.omitted]
    let omitted = ContextLayerDescriptor {
        trust: ContextLayerTrust::Unknown,
        ..descriptor("ctx.omitted", LOW_PRECEDENCE)
    };
    let plan = assemble_context_layers(
        &policy(),
        &[descriptor(DESCRIPTOR_ID, HIGH_PRECEDENCE), omitted],
    );

    assert_eq!(plan.receipt.included.len(), 1);
    assert_eq!(plan.receipt.omitted.len(), 1);
    assert!(
        plan.receipt.included[0]
            .body_hash
            .starts_with(BLAKE3_ALGORITHM)
    );
    assert_eq!(plan.receipt.omitted[0].reason, "untrusted");
    assert_eq!(plan.receipt.omitted[0].source_identity, SOURCE_ID);
}

fn descriptor(descriptor_id: &str, precedence: u32) -> ContextLayerDescriptor {
    ContextLayerDescriptor {
        descriptor_id: descriptor_id.to_string(),
        class: ContextLayerClass::AgentGuidance,
        source_identity: SOURCE_ID.to_string(),
        source_kind: ContextLayerSourceKind::Project,
        trust: ContextLayerTrust::Trusted,
        freshness: ContextLayerFreshness::Fresh,
        size_chars: BODY.chars().count(),
        redaction: ContextLayerRedaction::Safe,
        inclusion: ContextLayerInclusionPolicy::Enabled,
        precedence,
        section: SECTION.to_string(),
        body: BODY.to_string(),
    }
}

fn policy() -> ContextLayerAdmissionPolicy {
    ContextLayerAdmissionPolicy {
        discovery: ContextLayerPolicySwitch::Enabled,
        max_body_chars: MAX_BODY_CHARS,
        stale_context: ContextLayerPolicySwitch::Disabled,
        unknown_freshness: ContextLayerPolicySwitch::Disabled,
        untrusted_context: ContextLayerPolicySwitch::Disabled,
        conflict_diagnostics: ContextLayerPolicySwitch::Enabled,
    }
}
