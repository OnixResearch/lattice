use super::super::*;

const FILE_CAPABILITY: &str = "context.file";
const DIRECTORY_CAPABILITY: &str = "context.directory";
const DIFF_CAPABILITY: &str = "context.diff";
const IMAGE_CAPABILITY: &str = "context.image";
const URL_CAPABILITY: &str = "context.url";
const LAYER_CAPABILITY: &str = "context.layer-source";
const HANDLE_CAPABILITY: &str = "context.external-handle";
const FILE_EVIDENCE: &str = "context.file.positive";
const DIRECTORY_EVIDENCE: &str = "context.directory.positive";
const DIFF_EVIDENCE: &str = "context.diff.positive";
const IMAGE_EVIDENCE: &str = "context.image.positive";
const URL_EVIDENCE: &str = "context.url.positive";
const LAYER_EVIDENCE: &str = "context.layer.positive";
const HANDLE_EVIDENCE: &str = "context.handle.positive";
const UNSAFE_CAPABILITY_LABEL: &str = "/home/example/private.txt";

#[test]
fn context_adapter_roles_register_independently() -> Result<(), ContextReferenceError> {
    let records = [
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::File,
            capability_label: FILE_CAPABILITY,
            evidence_label: FILE_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::Directory,
            capability_label: DIRECTORY_CAPABILITY,
            evidence_label: DIRECTORY_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::Diff,
            capability_label: DIFF_CAPABILITY,
            evidence_label: DIFF_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::Image,
            capability_label: IMAGE_CAPABILITY,
            evidence_label: IMAGE_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::Url,
            capability_label: URL_CAPABILITY,
            evidence_label: URL_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::ContextLayerSource,
            capability_label: LAYER_CAPABILITY,
            evidence_label: LAYER_EVIDENCE,
        })?,
        role(&AvailableRoleFixture {
            role: ContextAdapterRole::ExternalHandle,
            capability_label: HANDLE_CAPABILITY,
            evidence_label: HANDLE_EVIDENCE,
        })?,
    ];
    let manifest = ContextAdapterFamilyManifest::new(&records)?;

    assert_eq!(manifest.roles().len(), records.len());
    assert!(
        manifest
            .require_admitted_role(ContextAdapterRole::File)?
            .is_dispatch_admitted()
    );
    assert_eq!(
        manifest
            .role_record(ContextAdapterRole::Url)
            .expect("URL role is registered")
            .capability_label(),
        URL_CAPABILITY
    );
    assert_eq!(ContextAdapterRole::ContextLayerSource.resolver_kind(), None);
    Ok(())
}

#[test]
fn context_adapter_roles_fail_closed_for_mismatches_and_unsafe_metadata() {
    assert_eq!(
        validate_context_resolver_role(ContextAdapterRole::File, ContextResolverKind::Url),
        Err(ContextReferenceError::UnsupportedKind {
            reason: "context adapter role does not match resolver kind",
        })
    );
    assert_eq!(
        validate_context_resolver_role(
            ContextAdapterRole::ExternalHandle,
            ContextResolverKind::File,
        ),
        Err(ContextReferenceError::UnsupportedKind {
            reason: "context adapter role does not match resolver kind",
        })
    );

    let missing = role_with(&RoleFixture {
        role: ContextAdapterRole::File,
        availability: ContextAdapterAvailability::Missing,
        policy_decision: ContextAdapterPolicyDecision::Allowed,
        capability_label: FILE_CAPABILITY,
        evidence_label: FILE_EVIDENCE,
    })
    .expect("missing role record uses safe metadata");
    let denied = role_with(&RoleFixture {
        role: ContextAdapterRole::Directory,
        availability: ContextAdapterAvailability::Available,
        policy_decision: ContextAdapterPolicyDecision::Denied,
        capability_label: DIRECTORY_CAPABILITY,
        evidence_label: DIRECTORY_EVIDENCE,
    })
    .expect("denied role record uses safe metadata");
    let manifest = ContextAdapterFamilyManifest::new(&[missing, denied])
        .expect("manifest with unique roles should validate");
    assert_eq!(
        manifest.require_admitted_role(ContextAdapterRole::File),
        Err(ContextReferenceError::ResolverMissing)
    );
    assert_eq!(
        manifest.require_admitted_role(ContextAdapterRole::Directory),
        Err(ContextReferenceError::ResolverDenied)
    );
    assert!(matches!(
        role_with(&RoleFixture {
            role: ContextAdapterRole::Url,
            availability: ContextAdapterAvailability::Available,
            policy_decision: ContextAdapterPolicyDecision::Allowed,
            capability_label: UNSAFE_CAPABILITY_LABEL,
            evidence_label: URL_EVIDENCE,
        }),
        Err(ContextReferenceError::SafeMetadata(_))
    ));
}

#[test]
fn context_adapter_manifest_rejects_duplicate_roles() -> Result<(), ContextReferenceError> {
    let fixture = AvailableRoleFixture {
        role: ContextAdapterRole::File,
        capability_label: FILE_CAPABILITY,
        evidence_label: FILE_EVIDENCE,
    };
    let first = role(&fixture)?;
    let second = role(&fixture)?;

    assert!(matches!(
        ContextAdapterFamilyManifest::new(&[first, second]),
        Err(ContextReferenceError::UnsupportedKind { .. })
    ));
    Ok(())
}

fn role(fixture: &AvailableRoleFixture) -> Result<ContextAdapterRoleRecord, ContextReferenceError> {
    role_with(&RoleFixture {
        role: fixture.role,
        availability: ContextAdapterAvailability::Available,
        policy_decision: ContextAdapterPolicyDecision::Allowed,
        capability_label: fixture.capability_label,
        evidence_label: fixture.evidence_label,
    })
}

struct AvailableRoleFixture {
    role: ContextAdapterRole,
    capability_label: &'static str,
    evidence_label: &'static str,
}

struct RoleFixture {
    role: ContextAdapterRole,
    availability: ContextAdapterAvailability,
    policy_decision: ContextAdapterPolicyDecision,
    capability_label: &'static str,
    evidence_label: &'static str,
}

fn role_with(fixture: &RoleFixture) -> Result<ContextAdapterRoleRecord, ContextReferenceError> {
    ContextAdapterRoleRecord::new(ContextAdapterRoleDraft {
        role: fixture.role,
        availability: fixture.availability,
        policy_decision: fixture.policy_decision,
        capability_label: fixture.capability_label,
        evidence_label: fixture.evidence_label,
    })
}
