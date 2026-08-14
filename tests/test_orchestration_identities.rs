use lattice::orchestration::ApprovalId;
use lattice::orchestration::AuthorityPolicyIdentity;
use lattice::orchestration::DefinitionId;
use lattice::orchestration::DefinitionRevision;
use lattice::orchestration::DispatchId;
use lattice::orchestration::EventId;
use lattice::orchestration::FactId;
use lattice::orchestration::InstanceId;
use lattice::orchestration::LedgerHeadId;
use lattice::orchestration::PreviousEventId;
use lattice::orchestration::ProposalId;
use lattice::orchestration::ReservationId;
use lattice::orchestration::StateDigest;
use lattice::orchestration::TriggerId;
use lattice::orchestration::TriggerIdempotencyKey;
use lattice::orchestration::TriggerScopedKey;
use lattice::orchestration::WorkflowRevision;

const VALID_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UPPERCASE_DIGEST: &str = "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";
const PREFIXED_DIGEST: &str = "blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OVERSIZED_ID_BYTES: usize = 257;

// r[verify lattice.agent_orchestration.nominal_identities.validation]
#[test]
fn checked_text_identities_accept_supported_values_and_preserve_wire_text() {
    let definition = DefinitionId::try_from_text("review-loop").expect("definition identity must be admitted");
    let trigger = TriggerId::try_from_text("webhook.review").expect("trigger identity must be admitted");
    let idempotency =
        TriggerIdempotencyKey::try_from_text("request:one").expect("idempotency identity must be admitted");
    let reservation = ReservationId::try_from_text(VALID_DIGEST).expect("reservation identity must be admitted");

    assert_eq!(definition.as_str(), "review-loop");
    assert_eq!(trigger.as_str(), "webhook.review");
    assert_eq!(idempotency.as_str(), "request:one");
    assert_eq!(reservation.as_str(), VALID_DIGEST);
    assert_eq!(serde_json::to_string(&definition).expect("definition identity serializes"), "\"review-loop\"");
}

// r[verify lattice.agent_orchestration.nominal_identities.validation]
#[test]
fn checked_text_identities_reject_empty_oversized_and_malformed_values_without_echo() {
    let oversized = "x".repeat(OVERSIZED_ID_BYTES);
    let empty = DefinitionId::try_from_text("").expect_err("empty identity must fail");
    let whitespace = TriggerId::try_from_text(" trigger").expect_err("non-canonical whitespace must fail");
    let control = TriggerIdempotencyKey::try_from_text("request\nsecret").expect_err("control text must fail");
    let too_long = ReservationId::try_from_text(&oversized).expect_err("oversized identity must fail");

    for error in [empty, whitespace, control, too_long] {
        assert!(!error.to_string().contains("secret"));
        assert!(!error.to_string().contains(&oversized));
    }
}

// r[verify lattice.agent_orchestration.nominal_identities.types.same_role]
// r[verify lattice.agent_orchestration.nominal_identities.validation]
#[test]
fn every_digest_role_accepts_one_canonical_lowercase_blake3_spelling() {
    assert_eq!(DefinitionRevision::try_from_hex(VALID_DIGEST).expect("definition revision").as_str(), VALID_DIGEST);
    assert_eq!(WorkflowRevision::try_from_hex(VALID_DIGEST).expect("workflow revision").as_str(), VALID_DIGEST);
    assert_eq!(InstanceId::try_from_hex(VALID_DIGEST).expect("instance identity").as_str(), VALID_DIGEST);
    assert_eq!(DispatchId::try_from_hex(VALID_DIGEST).expect("dispatch identity").as_str(), VALID_DIGEST);
    assert_eq!(ProposalId::try_from_hex(VALID_DIGEST).expect("proposal identity").as_str(), VALID_DIGEST);
    assert_eq!(ApprovalId::try_from_hex(VALID_DIGEST).expect("approval identity").as_str(), VALID_DIGEST);
    assert_eq!(FactId::try_from_hex(VALID_DIGEST).expect("fact identity").as_str(), VALID_DIGEST);
    assert_eq!(
        AuthorityPolicyIdentity::try_from_hex(VALID_DIGEST).expect("authority identity").as_str(),
        VALID_DIGEST
    );
    assert_eq!(EventId::try_from_hex(VALID_DIGEST).expect("event identity").as_str(), VALID_DIGEST);
    assert_eq!(PreviousEventId::try_from_hex(VALID_DIGEST).expect("previous event").as_str(), VALID_DIGEST);
    assert_eq!(LedgerHeadId::try_from_hex(VALID_DIGEST).expect("ledger head").as_str(), VALID_DIGEST);
    assert_eq!(StateDigest::try_from_hex(VALID_DIGEST).expect("state digest").as_str(), VALID_DIGEST);
    assert_eq!(TriggerScopedKey::try_from_hex(VALID_DIGEST).expect("trigger scope").as_str(), VALID_DIGEST);
}

// r[verify lattice.agent_orchestration.nominal_identities.validation]
#[test]
fn digest_roles_reject_empty_short_uppercase_and_prefixed_spellings() {
    for value in ["", "abcd", UPPERCASE_DIGEST, PREFIXED_DIGEST] {
        let error = InstanceId::try_from_hex(value).expect_err("unsupported digest spelling must fail");
        assert_eq!(error.to_string(), "orchestration instance identity must be one lowercase BLAKE3 hex digest");
        if !value.is_empty() {
            assert!(!error.to_string().contains(value));
        }
    }
}

// r[verify lattice.agent_orchestration.nominal_identities.wire_boundary]
#[test]
fn serde_adapters_keep_string_shape_and_reject_invalid_wire_values() {
    let identity = DispatchId::try_from_hex(VALID_DIGEST).expect("dispatch identity must be valid");
    let encoded = serde_json::to_string(&identity).expect("dispatch identity serializes");
    let decoded: DispatchId = serde_json::from_str(&encoded).expect("dispatch identity deserializes");
    let malformed = serde_json::from_str::<DispatchId>("\"dispatch-one\"");

    assert_eq!(encoded, format!("\"{VALID_DIGEST}\""));
    assert_eq!(decoded, identity);
    assert!(malformed.is_err());
}
