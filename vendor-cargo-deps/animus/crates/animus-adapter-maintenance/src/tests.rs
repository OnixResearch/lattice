use alloc::{string::String, vec, vec::Vec};

use animus_adapter_change_request::PublicationMode;

use crate::*;

const REPOSITORY_ID: &str = "repo:animus";
const POLICY_ID: &str = "policy:maintenance";
const PLAN_ID: &str = "plan:maintenance";
const IMPLEMENTATION_ID: &str = "impl:fixture";
const CORRELATION_ID: &str = "correlation:maintenance";
const MANIFEST_ID: &str = "manifest:root-cargo";
const SECOND_MANIFEST_ID: &str = "manifest:tools-cargo";
const HASH: &str = "b3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UPPER_HASH: &str = "b3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const ROUTINE_GROUP_SIZE: usize = 2;
const EXPECTED_GROUPS: usize = 4;
const NEXT_GENERATION: u64 = 2;
const SOURCE_COHORT_FILES: usize = 4;

fn binding() -> MaintenanceBinding {
    MaintenanceBinding::new(
        String::from(REPOSITORY_ID),
        String::from(POLICY_ID),
        String::from(PLAN_ID),
        1,
        String::from(IMPLEMENTATION_ID),
        String::from(CORRELATION_ID),
    )
    .expect("fixture binding must validate")
}

fn candidate(
    identity: &str,
    name: &str,
    manifest_identity: &str,
    update_kind: UpdateKind,
    risk: RiskLevel,
) -> UpdateCandidate {
    UpdateCandidate {
        candidate_identity: String::from(identity),
        dependency_name: String::from(name),
        ecosystem: Ecosystem::Cargo,
        manifest_identity: String::from(manifest_identity),
        current_version: String::from("1.0.0"),
        target_version: String::from("1.1.0"),
        update_kind,
        risk,
        advisory_identity: if update_kind == UpdateKind::Security {
            Some(String::from("advisory:fixture"))
        } else {
            None
        },
    }
}

fn policy() -> UpdatePolicy {
    UpdatePolicy {
        publication_mode: PublicationMode::Draft,
        require_review_for_major: true,
        isolate_security_updates: true,
        validation_profiles: vec![ValidationProfileRef {
            profile_identity: String::from("validation:workspace"),
        }],
    }
}

#[test]
fn groups_compatible_updates_and_isolates_major_and_security() {
    // r[verify agent_harness_runtime.remora_adapters.maintenance.grouping]
    // r[verify agent_harness_runtime.remora_adapters.maintenance.isolation]
    let input = UpdatePlanInput {
        binding: binding(),
        candidates: vec![
            candidate(
                "candidate:serde",
                "serde",
                MANIFEST_ID,
                UpdateKind::Minor,
                RiskLevel::Medium,
            ),
            candidate(
                "candidate:snafu",
                "snafu",
                MANIFEST_ID,
                UpdateKind::Patch,
                RiskLevel::Low,
            ),
            candidate(
                "candidate:anyhow",
                "anyhow",
                SECOND_MANIFEST_ID,
                UpdateKind::Patch,
                RiskLevel::Low,
            ),
            candidate(
                "candidate:tokio",
                "tokio",
                MANIFEST_ID,
                UpdateKind::Major,
                RiskLevel::High,
            ),
            candidate(
                "candidate:ring",
                "ring",
                MANIFEST_ID,
                UpdateKind::Security,
                RiskLevel::Unknown,
            ),
        ],
        policy: policy(),
        bounds: RunBounds::conservative(),
    };

    let plan = plan_update_run(input).expect("bounded candidates must plan");
    assert_eq!(plan.groups.len(), EXPECTED_GROUPS);
    let routine = plan.groups.first().expect("routine group must exist");
    assert_eq!(routine.updates.len(), ROUTINE_GROUP_SIZE);
    assert!(!routine.requires_human_review);
    assert_eq!(routine.validate(), Ok(()));
    let mut tampered_group = routine.clone();
    tampered_group.updates[0].target_version = String::from("1.2.0");
    assert_eq!(tampered_group.validate(), Err(Error::BindingMismatch));
    assert!(
        plan.groups
            .iter()
            .filter(|group| group.updates.len() == 1)
            .all(|group| group.updates.first().is_some_and(|update| {
                update.manifest_identity == SECOND_MANIFEST_ID
                    || matches!(update.update_kind, UpdateKind::Major | UpdateKind::Security)
            }))
    );
    assert!(
        plan.effects
            .iter()
            .any(|effect| matches!(effect, UpdateEffect::RequestValidation { .. }))
    );
    assert!(
        plan.effects
            .iter()
            .any(|effect| matches!(effect, UpdateEffect::PrepareChangeRequest { .. }))
    );
}

#[test]
fn zero_bounds_duplicates_and_unsafe_profiles_fail_closed() {
    let mut zero_bounds = RunBounds::conservative();
    zero_bounds.max_candidates = 0;
    assert_eq!(
        zero_bounds.validate(),
        Err(Error::InvalidBound("run_bounds.max_candidates"))
    );

    let duplicate = candidate(
        "candidate:duplicate",
        "serde",
        MANIFEST_ID,
        UpdateKind::Patch,
        RiskLevel::Low,
    );
    let duplicate_plan = UpdatePlanInput {
        binding: binding(),
        candidates: vec![duplicate.clone(), duplicate],
        policy: policy(),
        bounds: RunBounds::conservative(),
    };
    assert_eq!(
        plan_update_run(duplicate_plan),
        Err(Error::InvalidField("update_plan.duplicate_candidate"))
    );

    let raw_command = ValidationProfileRef {
        profile_identity: String::from("cargo test --workspace"),
    };
    assert_eq!(
        raw_command.validate(),
        Err(Error::InvalidField("validation_profile.profile_identity"))
    );
}

#[test]
fn missing_validation_profile_is_explicit_escalation() {
    let mut no_validation = policy();
    no_validation.validation_profiles.clear();
    let plan = plan_update_run(UpdatePlanInput {
        binding: binding(),
        candidates: vec![candidate(
            "candidate:serde",
            "serde",
            MANIFEST_ID,
            UpdateKind::Patch,
            RiskLevel::Low,
        )],
        policy: no_validation,
        bounds: RunBounds::conservative(),
    })
    .expect("missing validation must produce an explicit plan escalation");

    assert!(plan.effects.iter().any(|effect| matches!(
        effect,
        UpdateEffect::Escalate {
            reason: UpdateEscalationReason::MissingValidationProfile,
            ..
        }
    )));
}

#[test]
fn independent_service_catalog_fails_closed_for_missing_services() {
    // r[verify agent_harness_runtime.remora_adapters.ports.subset]
    // r[verify agent_harness_runtime.remora_adapters.ports.absent]
    let catalog = ServiceCatalog {
        services: vec![ServiceDescriptor {
            kind: MaintenanceServiceKind::Inventory,
            implementation_identity: String::from(IMPLEMENTATION_ID),
            availability: ServiceAvailability::Available,
        }],
    };

    assert_eq!(
        catalog
            .admit(MaintenanceServiceKind::Inventory)
            .map(|item| item.kind),
        Ok(MaintenanceServiceKind::Inventory)
    );
    assert_eq!(
        catalog.admit(MaintenanceServiceKind::WorkspaceUpdate),
        Err(Error::MissingService(
            MaintenanceServiceKind::WorkspaceUpdate
        ))
    );

    let stale = ServiceCatalog {
        services: vec![ServiceDescriptor {
            kind: MaintenanceServiceKind::CandidateDiscovery,
            implementation_identity: String::from(IMPLEMENTATION_ID),
            availability: ServiceAvailability::Stale,
        }],
    };
    assert_eq!(
        stale.admit(MaintenanceServiceKind::CandidateDiscovery),
        Err(Error::ServiceBlocked {
            service: MaintenanceServiceKind::CandidateDiscovery,
            availability: ServiceAvailability::Stale,
        })
    );
}

#[test]
fn inventory_and_candidate_observations_require_exact_bindings_and_bounds() {
    // r[verify agent_harness_runtime.remora_adapters.binding.match]
    // r[verify agent_harness_runtime.remora_adapters.binding.replay]
    let inventory_request = InventoryRequest {
        binding: binding(),
        max_manifests: 1,
        max_dependencies_per_manifest: DEFAULT_MAX_DEPENDENCIES_PER_MANIFEST,
    };
    let inventory = InventoryObservation {
        binding: binding(),
        manifests: vec![ManifestObservation {
            manifest_identity: String::from(MANIFEST_ID),
            ecosystem: Ecosystem::Cargo,
            lockfile_identity: Some(String::from("lock:root-cargo")),
            dependencies: vec![DependencyDeclaration {
                name: String::from("serde"),
                current_version: String::from("1.0.0"),
            }],
            evidence_identity: String::from(HASH),
        }],
        evidence_identity: String::from(HASH),
    };
    assert_eq!(
        validate_inventory_observation(&inventory_request, &inventory),
        Ok(())
    );

    let stale = MaintenanceBinding::new(
        String::from(REPOSITORY_ID),
        String::from(POLICY_ID),
        String::from(PLAN_ID),
        NEXT_GENERATION,
        String::from(IMPLEMENTATION_ID),
        String::from(CORRELATION_ID),
    )
    .expect("stale fixture binding remains structurally valid");
    let stale_inventory = InventoryObservation {
        binding: stale,
        ..inventory
    };
    assert_eq!(
        validate_inventory_observation(&inventory_request, &stale_inventory),
        Err(Error::BindingMismatch)
    );
}

#[test]
fn receipts_are_metadata_only_and_reject_overclaims() {
    // r[verify agent_harness_runtime.remora_adapters.evidence.safe]
    // r[verify agent_harness_runtime.remora_adapters.evidence.reject]
    let mut draft = MaintenanceReceiptDraft {
        request_binding: binding(),
        observation_binding: binding(),
        operation: ReceiptOperation::Plan,
        status: ReceiptStatus::Completed,
        item_count: 1,
        request_identity: String::from(HASH),
        observation_identity: String::from(HASH),
        disclosure: ContentDisclosure::MetadataOnly,
        claim: ReceiptClaim::BoundedContract,
    };
    let receipt = build_receipt(&draft).expect("safe maintenance receipt must build");
    assert!(receipt.receipt_identity.starts_with("b3:"));

    draft.request_identity = String::from(UPPER_HASH);
    assert_eq!(
        build_receipt(&draft),
        Err(Error::InvalidField("receipt.request_identity"))
    );
    draft.request_identity = String::from(HASH);
    draft.disclosure = ContentDisclosure::RawContent;
    assert_eq!(build_receipt(&draft), Err(Error::UnsafeReceipt));
    draft.disclosure = ContentDisclosure::MetadataOnly;
    draft.claim = ReceiptClaim::UpdateCorrectness;
    assert_eq!(build_receipt(&draft), Err(Error::UnsupportedClaim));
}

#[test]
fn source_provenance_is_exact_and_bounded() {
    // r[verify agent_harness_runtime.remora_adapters.provenance.cohort]
    assert_eq!(
        REMORA_SOURCE_REPOSITORY,
        "git@github.com:OnixResearch/remora.git"
    );
    assert_eq!(
        REMORA_SOURCE_REVISION,
        "e3c857c857f4aa170d8a5b6286aad5855db541be"
    );
    assert_eq!(REMORA_DECLARED_LICENSE, "MIT OR Apache-2.0");
    assert_eq!(
        REMORA_SOURCE_COHORT_IDENTITY,
        "b3:a084a8a06727b1fae59377d34754f224065351e550caadf4e12c0bbcdcc26e42"
    );
    assert_eq!(REMORA_SOURCE_COHORT.len(), SOURCE_COHORT_FILES);
    assert!(REMORA_IMPORTED_SCOPE.contains("authority-free"));
    assert!(REMORA_OWNER_GRANT_BASIS.contains("owner adaptation"));
}

#[test]
fn radar_and_scheduler_types_do_not_enter_the_maintenance_family() {
    // r[verify agent_harness_runtime.remora_adapters.composition.radar]
    let prohibited = [
        "scanner runner",
        "dashboard store",
        "cron daemon",
        "provider client",
    ];
    assert!(
        prohibited
            .iter()
            .all(|value| !REMORA_IMPORTED_SCOPE.contains(value))
    );
    assert_eq!(Vec::<String>::new().len(), 0);
}
