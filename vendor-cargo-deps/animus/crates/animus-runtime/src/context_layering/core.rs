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
use std::collections::BTreeMap;

use super::{
    BLAKE3_ALGORITHM, CONTEXT_LAYER_HASH_DOMAIN, EMPTY_COUNT, FIELD_BODY, FIELD_CLASS,
    FIELD_DISCOVERY, FIELD_PRECEDENCE, FIELD_SOURCE, HASH_PART_SEPARATOR,
    data::{
        ContextLayerAdmissionPolicy, ContextLayerBoundaryEffect, ContextLayerClass,
        ContextLayerDescriptor, ContextLayerDiagnostic, ContextLayerDiagnosticKind,
        ContextLayerDiscoveryBoundary, ContextLayerFreshness, ContextLayerInclusionPolicy,
        ContextLayerPlan, ContextLayerPolicySwitch, ContextLayerReceipt, ContextLayerReceiptEntry,
        ContextLayerRedaction, ContextLayerTrust, ContextLayerValidationResult,
    },
};

const RAW_SECRET_MARKER: &str = "API_KEY";
const RAW_HOST_PATH_MARKER: &str = "/home/";
const RAW_PROVIDER_MARKER: &str = "provider_payload";
const WINDOWS_PATH_MARKER: &str = ":\\";

/// Computes a context-layer BLAKE3 fingerprint.
#[must_use]
pub fn blake3_context_layer_fingerprint(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CONTEXT_LAYER_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    format!("{BLAKE3_ALGORITHM}:{}", hasher.finalize().to_hex())
}

/// Validates a context descriptor shape.
///
/// # Errors
///
/// Returns diagnostics for unknown classes, missing source identity, missing
/// section labels, size/body mismatch, unsafe bodies, or missing descriptor IDs.
pub fn validate_context_descriptor(
    descriptor: &ContextLayerDescriptor,
) -> ContextLayerValidationResult {
    let diagnostics = descriptor_shape_diagnostics(descriptor);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Assembles admitted context layers without ambient I/O.
#[must_use]
pub fn assemble_context_layers(
    policy: &ContextLayerAdmissionPolicy,
    descriptors: &[ContextLayerDescriptor],
) -> ContextLayerPlan {
    let mut diagnostics = Vec::new();
    let mut admitted = Vec::new();
    let mut omitted = Vec::new();
    for descriptor in descriptors {
        let descriptor_diagnostics = admission_diagnostics(policy, descriptor);
        if descriptor_diagnostics.is_empty() {
            admitted.push(descriptor.clone());
        } else {
            diagnostics.extend(descriptor_diagnostics.clone());
            omitted.push(receipt_entry(
                descriptor,
                reason_for(&descriptor_diagnostics),
            ));
        }
    }
    admitted.sort_by(|left, right| {
        right
            .precedence
            .cmp(&left.precedence)
            .then_with(|| left.descriptor_id.cmp(&right.descriptor_id))
    });
    let mut winners: BTreeMap<String, ContextLayerDescriptor> = BTreeMap::new();
    for descriptor in admitted {
        if let Some(existing) = winners.get(&descriptor.section) {
            if policy.conflict_diagnostics == ContextLayerPolicySwitch::Enabled {
                diagnostics.push(ContextLayerDiagnostic::new(
                    ContextLayerDiagnosticKind::Conflict,
                    FIELD_PRECEDENCE,
                    "lower-precedence context descriptor omitted",
                ));
            }
            omitted.push(receipt_entry(
                if existing.precedence >= descriptor.precedence {
                    &descriptor
                } else {
                    existing
                },
                "lower-precedence-conflict",
            ));
            if descriptor.precedence > existing.precedence {
                winners.insert(descriptor.section.clone(), descriptor);
            }
        } else {
            winners.insert(descriptor.section.clone(), descriptor);
        }
    }
    let mut included: Vec<ContextLayerReceiptEntry> = winners
        .values()
        .map(|descriptor| receipt_entry(descriptor, "included"))
        .collect();
    included.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    omitted.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    ContextLayerPlan {
        prompt_sections: included.clone(),
        receipt: ContextLayerReceipt { included, omitted },
        discovery_boundary: ContextLayerDiscoveryBoundary {
            ancestor_walk: ContextLayerBoundaryEffect::NotPerformed,
            file_read: ContextLayerBoundaryEffect::NotPerformed,
            environment_read: ContextLayerBoundaryEffect::NotPerformed,
            path_canonicalization: ContextLayerBoundaryEffect::NotPerformed,
        },
        diagnostics,
    }
}

fn admission_diagnostics(
    policy: &ContextLayerAdmissionPolicy,
    descriptor: &ContextLayerDescriptor,
) -> Vec<ContextLayerDiagnostic> {
    let mut diagnostics = descriptor_shape_diagnostics(descriptor);
    if policy.discovery == ContextLayerPolicySwitch::Disabled
        || descriptor.inclusion == ContextLayerInclusionPolicy::Disabled
    {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::DiscoveryDisabled,
            FIELD_DISCOVERY,
            "context discovery or inclusion is disabled",
        ));
    }
    if descriptor.trust != ContextLayerTrust::Trusted
        && policy.untrusted_context == ContextLayerPolicySwitch::Disabled
    {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Untrusted,
            FIELD_SOURCE,
            "context source is not trusted for prompt influence",
        ));
    }
    if descriptor.freshness == ContextLayerFreshness::Stale
        && policy.stale_context == ContextLayerPolicySwitch::Disabled
    {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Stale,
            FIELD_SOURCE,
            "context descriptor is stale",
        ));
    }
    if descriptor.freshness == ContextLayerFreshness::Unknown
        && policy.unknown_freshness == ContextLayerPolicySwitch::Disabled
    {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Stale,
            FIELD_SOURCE,
            "context descriptor freshness is unknown",
        ));
    }
    if descriptor.size_chars > policy.max_body_chars {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Oversized,
            FIELD_BODY,
            "context body exceeds prompt inclusion size policy",
        ));
    }
    diagnostics
}

fn contains_unsafe_marker(text: &str) -> bool {
    text.contains(RAW_SECRET_MARKER)
        || text.contains(RAW_HOST_PATH_MARKER)
        || text.contains(RAW_PROVIDER_MARKER)
        || text.contains(WINDOWS_PATH_MARKER)
}

fn descriptor_shape_diagnostics(
    descriptor: &ContextLayerDescriptor,
) -> Vec<ContextLayerDiagnostic> {
    let mut diagnostics = Vec::new();
    if descriptor.descriptor_id.trim().is_empty() || descriptor.section.trim().is_empty() {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::MissingSource,
            FIELD_SOURCE,
            "descriptor id and section are required",
        ));
    }
    if descriptor.class == ContextLayerClass::Unknown {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::UnknownClass,
            FIELD_CLASS,
            "context descriptor class is unknown",
        ));
    }
    if descriptor.source_identity.trim().is_empty() {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::MissingSource,
            FIELD_SOURCE,
            "context descriptor source identity is required",
        ));
    }
    if descriptor.size_chars != descriptor.body.chars().count() {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Oversized,
            FIELD_BODY,
            "declared context size must match supplied body characters",
        ));
    }
    if descriptor.body.chars().count() == EMPTY_COUNT
        || descriptor.redaction == ContextLayerRedaction::Unsafe
        || contains_unsafe_marker(&descriptor.body)
        || contains_unsafe_marker(&descriptor.source_identity)
    {
        diagnostics.push(ContextLayerDiagnostic::new(
            ContextLayerDiagnosticKind::Unsafe,
            FIELD_BODY,
            "context body or source identity is unsafe for prompt influence",
        ));
    }
    diagnostics
}

fn reason_for(diagnostics: &[ContextLayerDiagnostic]) -> &'static str {
    diagnostics
        .first()
        .map_or("omitted", |diagnostic| match diagnostic.kind {
            ContextLayerDiagnosticKind::UnknownClass => "unknown-class",
            ContextLayerDiagnosticKind::MissingSource => "missing-source",
            ContextLayerDiagnosticKind::DiscoveryDisabled => "disabled",
            ContextLayerDiagnosticKind::Untrusted => "untrusted",
            ContextLayerDiagnosticKind::Stale => "stale",
            ContextLayerDiagnosticKind::Oversized => "oversized",
            ContextLayerDiagnosticKind::Unsafe => "unsafe",
            ContextLayerDiagnosticKind::Conflict => "conflict",
        })
}

fn receipt_entry(descriptor: &ContextLayerDescriptor, reason: &str) -> ContextLayerReceiptEntry {
    ContextLayerReceiptEntry {
        descriptor_id: descriptor.descriptor_id.clone(),
        class: descriptor.class,
        source_kind: descriptor.source_kind,
        source_identity: descriptor.source_identity.clone(),
        body_hash: blake3_context_layer_fingerprint(
            &descriptor.descriptor_id,
            descriptor.body.as_bytes(),
        ),
        reason: reason.to_string(),
    }
}
