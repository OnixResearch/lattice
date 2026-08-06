use alloc::string::ToString;

const FIRST_HASH_TEXT: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SECOND_HASH_TEXT: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const LETTER_HASH_TEXT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECRET_PAYLOAD: &str = "credential-looking secret payload";
const PUBLIC_PAYLOAD: &str = "safe public payload";
const UNIQUE_DEPENDENCY_COUNT: usize = 2;

#[test]
fn identity_is_deterministic_for_dependency_order() {
    let first = super::Hash::new(FIRST_HASH_TEXT).expect("first dependency hash should parse");
    let second = super::Hash::new(SECOND_HASH_TEXT).expect("second dependency hash should parse");
    let left_dependencies = [second.clone(), first.clone(), second.clone()];
    let right_dependencies = [first.clone(), second.clone()];

    let left = super::Envelope::new(
        super::Draft::new(super::Kind::RuntimeReceipt, super::RedactionClass::MetadataOnly, PUBLIC_PAYLOAD)
            .with_dependencies(&left_dependencies),
    )
    .expect("left envelope should build");
    let right = super::Envelope::new(
        super::Draft::new(super::Kind::RuntimeReceipt, super::RedactionClass::MetadataOnly, PUBLIC_PAYLOAD)
            .with_dependencies(&right_dependencies),
    )
    .expect("right envelope should build");

    assert_eq!(left.dependencies(), right.dependencies());
    assert_eq!(left.identity(), right.identity());
    assert_eq!(left.dependencies().len(), UNIQUE_DEPENDENCY_COUNT);
}

#[test]
fn hash_display_and_parse_round_trip() {
    let envelope = super::Envelope::new(super::Draft::new(
        super::Kind::ReviewSummary,
        super::RedactionClass::Public,
        PUBLIC_PAYLOAD,
    ))
    .expect("envelope should build");

    let displayed = envelope.identity().to_string();
    let parsed: super::Hash = displayed.parse().expect("displayed hash should parse");

    assert_eq!(parsed, *envelope.identity());
    assert_eq!(displayed.len(), super::BLAKE3_HASH_HEX_CHARS);
}

#[test]
fn non_public_outputs_hide_payloads() {
    let classes = [
        super::RedactionClass::MetadataOnly,
        super::RedactionClass::RedactedPayload,
        super::RedactionClass::Secret,
    ];

    for redaction in classes {
        let envelope =
            super::Envelope::new(super::Draft::new(super::Kind::ProvenanceObservation, redaction, SECRET_PAYLOAD))
                .expect("non-public envelope should build");
        let summary = envelope.review_summary();

        assert!(!redaction.allows_payload_display());
        assert!(!summary.contains(SECRET_PAYLOAD));
        assert_ne!(envelope.review_payload(), SECRET_PAYLOAD);
    }
}

#[test]
fn public_payload_is_displayable() {
    let envelope = super::Envelope::new(super::Draft::new(
        super::Kind::ReviewSummary,
        super::RedactionClass::Public,
        PUBLIC_PAYLOAD,
    ))
    .expect("public envelope should build");

    assert!(envelope.redaction().allows_payload_display());
    assert_eq!(envelope.review_payload(), PUBLIC_PAYLOAD);
    assert!(envelope.review_summary().contains(PUBLIC_PAYLOAD));
}

#[test]
fn uppercase_hash_is_rejected() {
    let uppercase = LETTER_HASH_TEXT.to_ascii_uppercase();
    let error = super::Hash::new(&uppercase).expect_err("uppercase hash should fail");

    assert_eq!(error, super::Error::InvalidHash {
        field: super::digest::test_support::FIELD,
        reason: super::digest::test_support::NOT_HEX_REASON,
    });
    assert_eq!(uppercase.len(), super::BLAKE3_HASH_HEX_CHARS);
}
