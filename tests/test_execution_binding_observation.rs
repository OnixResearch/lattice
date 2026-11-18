//! Synthetic `lattice.compute-allocation-observation.v1` fixtures derived from
//! the producer's `agent-a-serving-b` example. They are not live observations.

// r[verify lattice.execution_binding.allocation_observation]
// r[verify lattice.execution_binding.allocation_observation.scenario.admitted]
// r[verify lattice.execution_binding.allocation_observation.scenario.malformed]
// r[verify lattice.execution_binding.allocation_observation.scenario.unknown]

use lattice::execution_binding::core::admission::facts::*;
use lattice::execution_binding::core::model::*;
use lattice::execution_binding::core::validation::*;

const POSITIVE: [&str; 4] = [
    "agent-a-1-active",
    "agent-a-2-active",
    "serving-b-deployment",
    "device-active",
];

fn bytes(relative: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations")
        .join(format!("{relative}.json"));
    std::fs::read(path).expect("fixture bytes")
}

fn observation(name: &str) -> ComputeAllocationObservation {
    decode_observation(&bytes(&format!("positive/{name}"))).expect("valid synthetic fixture")
}

#[test]
fn positive_fixtures_preserve_every_producer_field_verbatim() {
    for name in POSITIVE {
        let raw = bytes(&format!("positive/{name}"));
        let decoded = decode_observation(&raw).expect(name);
        let original: serde_json::Value = serde_json::from_slice(&raw).expect("fixture json");
        assert_eq!(serde_json::to_value(&decoded).expect("encode"), original, "{name}");
    }
    let agent = observation("agent-a-1-active");
    assert_eq!(agent.allocation.status, ReservationStatus::Active);
    assert_eq!(agent.allocation.enforcement, Enforcement::LogicalFenceOnly);
    assert_eq!(agent.allocation.serving_allocation_id.as_deref(), Some("serving-b"));
    assert_eq!((agent.port.provider_logical_now, agent.allocation.valid_until), (Some(30), 80));
    let serving = observation("serving-b-deployment");
    let encoded = serde_json::to_string(&serving).expect("encode");
    for spelling in [
        "\"mmio-bundle\"",
        "\"serving-deployment\"",
        "\"backend-fenced\"",
        "\"unified-memory-bytes\"",
    ] {
        assert!(encoded.contains(spelling), "{spelling}");
    }
    assert_eq!(serving.allocation.serving_allocation_id, None);
}

#[test]
fn malformed_or_inconsistent_fixtures_are_rejected_with_typed_reasons() {
    let cases = [
        ("allocation-v1-label", ObservationRejection::ProducerLabel {
            field: "allocation_label",
        }),
        ("readback-v2-label", ObservationRejection::ProducerLabel {
            field: "readback_label",
        }),
        ("wrong-reservation-schema", ObservationRejection::ProducerLabel { field: "schema" }),
        ("id-not-operation", ObservationRejection::IdNotOperation),
        ("repeated-pool", ObservationRejection::RepeatedPool),
        ("zero-quantity", ObservationRejection::Zero { field: "quantity" }),
        ("authority-count", ObservationRejection::AuthorityCount),
        ("serving-references-serving", ObservationRejection::ServingReferencesServing),
        ("active-without-evidence", ObservationRejection::ActivationEvidence),
        ("uppercase-ref", ObservationRejection::Reference { field: "grant_ref" }),
        ("oversized-claims", ObservationRejection::Bound { field: "claims" }),
        ("row-owner-mismatch", ObservationRejection::ReadbackRow {
            field: "owner_generation",
        }),
        ("row-fence-mismatch", ObservationRejection::ReadbackRow {
            field: "is_logically_fenced",
        }),
        ("offer-revision-mismatch", ObservationRejection::Offer { field: "revision" }),
        ("offer-authority-mismatch", ObservationRejection::Offer { field: "authority_id" }),
    ];
    for (name, expected) in cases {
        assert_eq!(decode_observation(&bytes(&format!("negative/{name}"))), Err(expected), "{name}");
    }
}

#[test]
fn zero_generations_and_unknown_fields_are_rejected() {
    let mut zero = observation("agent-a-1-active");
    zero.allocation.topology_generation = 0;
    assert_eq!(
        validate_observation(&zero),
        Err(ObservationRejection::Zero {
            field: "topology_generation"
        })
    );
    let mut sequence = observation("agent-a-1-active");
    sequence.port.observation_sequence = 0;
    assert_eq!(
        validate_observation(&sequence),
        Err(ObservationRejection::Zero {
            field: "observation_sequence"
        })
    );
    let mut control = observation("agent-a-1-active");
    control.allocation.view_id = "view\nwith-control".into();
    assert_eq!(validate_observation(&control), Err(ObservationRejection::Token { field: "view_id" }));
    let mut value: serde_json::Value = serde_json::from_slice(&bytes("positive/agent-a-1-active")).expect("json");
    value["allocation"]["lease"] = serde_json::Value::from("invented");
    let invented = serde_json::to_vec(&value).expect("bytes");
    assert_eq!(decode_observation(&invented), Err(ObservationRejection::Malformed));
}

#[test]
fn byte_ceiling_and_arbitrary_prefixes_fail_closed_without_panicking() {
    let raw = bytes("positive/agent-a-1-active");
    let mut padded = raw.clone();
    padded.resize(65_537, b' ');
    assert_eq!(decode_observation(&padded), Err(ObservationRejection::Oversized));
    let mut exact = raw.clone();
    exact.resize(65_536, b' ');
    assert!(decode_observation(&exact).is_ok());
    let mut large = observation("agent-a-1-active");
    large.readback.qualification_refs = (0..128).map(|index| format!("blake3:{index:064x}")).collect();
    large.allocation.view_id = "v".repeat(256);
    assert!(validate_observation(&large).is_ok(), "bounded observation stays within the ceiling");
    for end in 0..raw.trim_ascii_end().len() {
        assert!(decode_observation(&raw[..end]).is_err());
    }
}

#[test]
fn unknown_logical_time_keeps_expiry_unknown_and_never_unexpired() {
    let mut absent = observation("agent-a-1-active");
    absent.port.provider_logical_now = None;
    assert_eq!(validate_observation(&absent), Ok(()));
    assert_eq!(classify(&absent).expiry, Expiry::Unknown);
    let mut at_bound = observation("agent-a-1-active");
    at_bound.port.provider_logical_now = Some(80);
    assert_eq!(classify(&at_bound).expiry, Expiry::Expired);
    at_bound.port.provider_logical_now = Some(79);
    assert_eq!(classify(&at_bound).expiry, Expiry::Unexpired);
}

#[test]
fn unknown_offer_facts_keep_device_kind_unknown_and_never_zero() {
    assert_eq!(classify(&observation("agent-a-2-active")).device_kind, DeviceKind::Unknown);
    assert_eq!(classify(&observation("agent-a-1-active")).device_kind, DeviceKind::Known { chips: 0 });
    assert_eq!(classify(&observation("device-active")).device_kind, DeviceKind::Known { chips: 1 });
}

#[test]
fn unknown_freshness_and_health_are_never_reported_healthy() {
    let mut stale = observation("agent-a-1-active");
    stale.readback.is_fresh = false;
    assert_eq!(classify(&stale).freshness, Freshness::Unknown);
    assert_eq!(classify(&observation("agent-a-1-active")).health, HostHealth::Healthy);
    for reported in [Health::Degraded, Health::Unavailable, Health::Unknown] {
        let mut unhealthy = observation("agent-a-1-active");
        unhealthy.readback.health = reported;
        assert_eq!(classify(&unhealthy).health, HostHealth::Unavailable { reported });
    }
}

// r[verify lattice.execution_binding.deferred.scenario.live]
#[test]
fn fixtures_are_synthetic_and_pin_the_producer_revision_and_consumer_schema() {
    assert_eq!(PRODUCER_REVISION, "aac7b5ba974cc9660749dab6d8ef4b7db04e3cb4");
    assert_eq!(OBSERVATION_SCHEMA, "lattice.compute-allocation-observation.v1");
    for name in POSITIVE {
        let observation = observation(name);
        assert!(observation.port.port_id.starts_with("synthetic-"), "{name} is labelled synthetic");
        assert!(
            observation
                .allocation
                .activation_backend_ref
                .as_deref()
                .is_none_or(|value| value.starts_with("synthetic-"))
        );
    }
}
