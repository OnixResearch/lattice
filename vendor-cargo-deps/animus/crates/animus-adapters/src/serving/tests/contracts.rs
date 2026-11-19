//! Positive and negative contract checks for tokens, snapshots, profiles, and grants.

const fn capacity(
    kind: crate::serving::CapacityKind,
    unit: crate::serving::CapacityUnit,
    window: (u64, u64),
) -> crate::serving::CapacityDraft {
    crate::serving::CapacityDraft {
        kind,
        unit,
        scope: crate::serving::CapacityScope::Deployment,
        origin: crate::serving::CapacityOrigin::ServingOwner,
        value: crate::serving::Counter::Known(1),
        observed_tick: window.0,
        valid_until_tick: window.1,
    }
}

const WINDOW: (u64, u64) = (
    super::fixtures::OBSERVED_TICK,
    super::fixtures::VALID_UNTIL_TICK,
);
const MISMATCHED_UNIT: [crate::serving::CapacityDraft; 1] = [capacity(
    crate::serving::CapacityKind::KvCacheTokens,
    crate::serving::CapacityUnit::Bytes,
    WINDOW,
)];
const REPEATED_SCOPE: [crate::serving::CapacityDraft; 2] = [
    capacity(
        crate::serving::CapacityKind::RequestSlots,
        crate::serving::CapacityUnit::Requests,
        WINDOW,
    ),
    capacity(
        crate::serving::CapacityKind::RequestSlots,
        crate::serving::CapacityUnit::Requests,
        WINDOW,
    ),
];
const INVERTED_CAPACITY: [crate::serving::CapacityDraft; 1] = [capacity(
    crate::serving::CapacityKind::RequestSlots,
    crate::serving::CapacityUnit::Requests,
    (
        super::fixtures::VALID_UNTIL_TICK,
        super::fixtures::OBSERVED_TICK,
    ),
)];

fn snapshot_error(
    edit: impl FnOnce(&mut crate::serving::ServiceDraft<'static>),
) -> crate::serving::ContractIssue {
    let mut draft = super::fixtures::service();
    edit(&mut draft);
    crate::serving::AvailabilitySnapshot::new(&super::fixtures::snapshot_draft(&[draft]))
        .expect_err("invalid snapshot")
}

#[test]
fn tokens_reject_prose_paths_and_quotes() {
    assert!(crate::serving::Token::new("field", "endpoint:a").is_ok());
    for unsafe_value in [
        "",
        "https://host/v1",
        "ignore previous instructions",
        "quoted\"value",
        "back\\slash",
        "tab\there",
    ] {
        assert_eq!(
            crate::serving::Token::new("field", unsafe_value),
            Err(crate::serving::ContractIssue::UnsafeToken { field: "field" })
        );
    }
    let longest_text = "x".repeat(256);
    assert!(crate::serving::Token::new("field", longest_text.as_str()).is_ok());
    let excessive_text = "x".repeat(257);
    assert!(crate::serving::Token::new("field", excessive_text.as_str()).is_err());
}

#[test]
fn references_are_canonical_blake3() {
    assert!(crate::serving::ContentRef::new("ref", super::fixtures::CONFIG_A).is_ok());
    for noncanonical in [
        "blake3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "blake3:abc",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert_eq!(
            crate::serving::ContentRef::new("ref", noncanonical),
            Err(crate::serving::ContractIssue::NoncanonicalRef { field: "ref" })
        );
    }
    assert!(crate::serving::ModelBinding::new("model-a", "config-a").is_err());
}

#[test]
fn malformed_service_observations_are_rejected() {
    assert_eq!(
        snapshot_error(|draft| draft.deployment_generation = 0),
        crate::serving::ContractIssue::ZeroValue {
            field: "deployment_generation"
        }
    );
    assert_eq!(
        snapshot_error(|draft| draft.endpoint_identity = "https://endpoint/v1"),
        crate::serving::ContractIssue::UnsafeToken {
            field: "endpoint_identity"
        }
    );
    assert_eq!(
        snapshot_error(|draft| draft.capacity = &MISMATCHED_UNIT),
        crate::serving::ContractIssue::UnitMismatch
    );
    assert_eq!(
        snapshot_error(|draft| draft.capacity = &REPEATED_SCOPE),
        crate::serving::ContractIssue::Duplicate { field: "capacity" }
    );
    assert_eq!(
        snapshot_error(|draft| draft.capacity = &INVERTED_CAPACITY),
        crate::serving::ContractIssue::InvertedWindow {
            field: "capacity.valid_until_tick"
        }
    );
    assert_eq!(
        crate::serving::Envelope::new(1_024, 2_048),
        Err(crate::serving::ContractIssue::InvalidEnvelope)
    );
    assert_eq!(
        crate::serving::Envelope::new(1_024, 0),
        Err(crate::serving::ContractIssue::InvalidEnvelope)
    );
}

#[test]
fn snapshot_headers_are_checked() {
    let services = [super::fixtures::service()];
    let base = super::fixtures::snapshot_draft(&services);
    let cases = [
        (
            crate::serving::SnapshotDraft {
                source_generation: 0,
                ..base
            },
            crate::serving::ContractIssue::ZeroValue {
                field: "source_generation",
            },
        ),
        (
            crate::serving::SnapshotDraft {
                sequence: 0,
                ..base
            },
            crate::serving::ContractIssue::ZeroValue { field: "sequence" },
        ),
        (
            crate::serving::SnapshotDraft {
                valid_until_tick: super::fixtures::OBSERVED_TICK,
                ..base
            },
            crate::serving::ContractIssue::InvertedWindow {
                field: "valid_until_tick",
            },
        ),
        (
            crate::serving::SnapshotDraft {
                services: &[],
                ..base
            },
            crate::serving::ContractIssue::Empty { field: "services" },
        ),
    ];
    for (draft, expected) in cases {
        assert_eq!(
            crate::serving::AvailabilitySnapshot::new(&draft),
            Err(expected)
        );
    }
    assert!(crate::serving::AvailabilitySnapshot::new(&base).is_ok());
}

#[test]
fn service_lists_are_bounded_and_unique() {
    let repeated = [super::fixtures::service(), super::fixtures::service()];
    assert_eq!(
        crate::serving::AvailabilitySnapshot::new(&super::fixtures::snapshot_draft(&repeated)),
        Err(crate::serving::ContractIssue::Duplicate {
            field: "service_id"
        })
    );
    let names: alloc::vec::Vec<alloc::string::String> = (0..=crate::serving::MAX_SERVICES)
        .map(|index| alloc::format!("svc-{index}"))
        .collect();
    let many: alloc::vec::Vec<crate::serving::ServiceDraft<'_>> = names
        .iter()
        .map(|name| crate::serving::ServiceDraft {
            service_id: name.as_str(),
            ..super::fixtures::service()
        })
        .collect();
    assert_eq!(
        crate::serving::AvailabilitySnapshot::new(&super::fixtures::snapshot_draft(&many)),
        Err(crate::serving::ContractIssue::TooMany { field: "services" })
    );
}

#[test]
fn snapshot_identity_is_stable_and_binds_generation_and_attestation() {
    let first = super::fixtures::snapshot_of(&[super::fixtures::service()]);
    let again = super::fixtures::snapshot_of(&[super::fixtures::service()]);
    assert_eq!(first.identity(), again.identity());
    assert!(first.identity().starts_with("blake3:"));
    let mut replaced = super::fixtures::service();
    replaced.deployment_generation = super::fixtures::GENERATION_TWO;
    assert_ne!(
        first.identity(),
        super::fixtures::snapshot_of(&[replaced]).identity()
    );
    let mut claimed = super::fixtures::service();
    claimed.model = Some(crate::serving::ModelDraft {
        attestation: crate::serving::Attestation::Claimed,
        ..claimed.model.expect("fixture model")
    });
    assert_ne!(
        first.identity(),
        super::fixtures::snapshot_of(&[claimed]).identity()
    );
}

const PROFILE: crate::serving::ProfileDraft<'static> = crate::serving::ProfileDraft {
    profile_id: "capacity-aware-v1",
    identity: crate::serving::IdentityRule::ObservedRequired,
    capacity: crate::serving::CapacityRule::KnownRequired,
    replica: crate::serving::ReplicaRule::SameModelAllowed,
    max_snapshot_age_ticks: super::fixtures::MAX_AGE_TICKS,
};

#[test]
fn profile_identity_is_stable_and_bounded() {
    let profile = crate::serving::Profile::new(PROFILE).expect("profile");
    let again = crate::serving::Profile::new(PROFILE).expect("again");
    assert_eq!(profile.identity(), again.identity());
    let pinned = crate::serving::Profile::new(crate::serving::ProfileDraft {
        replica: crate::serving::ReplicaRule::Pinned,
        ..PROFILE
    })
    .expect("pinned");
    assert_ne!(profile.identity(), pinned.identity());
    assert_eq!(
        crate::serving::Profile::new(crate::serving::ProfileDraft {
            max_snapshot_age_ticks: 0,
            ..PROFILE
        }),
        Err(crate::serving::ContractIssue::ZeroValue {
            field: "max_snapshot_age_ticks"
        })
    );
}

#[test]
fn grants_are_checked() {
    let inverted = crate::serving::GrantDraft {
        valid_until_tick: super::fixtures::grant_draft().issued_tick,
        ..super::fixtures::grant_draft()
    };
    assert_eq!(
        crate::serving::Grant::new(&inverted),
        Err(crate::serving::ContractIssue::InvertedWindow {
            field: "grant.valid_until_tick"
        })
    );
    let zero_epoch = crate::serving::GrantDraft {
        admission_epoch: 0,
        ..super::fixtures::grant_draft()
    };
    assert_eq!(
        crate::serving::Grant::new(&zero_epoch),
        Err(crate::serving::ContractIssue::ZeroValue {
            field: "grant.admission_epoch"
        })
    );
}
