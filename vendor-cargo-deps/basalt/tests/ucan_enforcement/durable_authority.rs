use std::cell::{Cell, RefCell};

use basalt::ucan::{
    DurableReplayContext, DurableReplayContextInput, DurableReplayDurability,
    DurableRevocationContext, DurableRevocationContextInput,
};
use replay_ledger_core::{
    CommitObservation as ReplayCommitObservation, CompareAndSwapCommand as ReplayCommand,
    DurabilityObservation as ReplayDurabilityObservation, ReplayLedgerPort, ReplayRecord,
    ReplaySlotIdentity, SlotObservation,
};
use revocation_view_core::{
    AuthenticityObservation, CommitObservation as RevocationCommitObservation,
    CompareAndSwapCommand as RevocationCommand, CompletenessObservation, EntryLimit,
    RevocationSnapshot, RevocationViewPort, SnapshotInput, SnapshotObservation, SourceDigest,
    SourceGeneration, StoredSnapshot, UnixTime, UpdateDecision, plan_update,
};

use crate::support::*;

const TENANT_BYTE: u8 = 2;
const OPERATION_BYTE: u8 = 3;
const NAMESPACE_BYTE: u8 = 4;
const ATTEMPT_BYTE: u8 = 5;
const SOURCE_BYTE: u8 = 6;
const REVOCATION_ENTRY_LIMIT: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryError {
    Injected,
}

struct ReplayPort {
    record: RefCell<Option<ReplayRecord>>,
    writes: Cell<usize>,
}

impl ReplayPort {
    const fn new() -> Self {
        Self {
            record: RefCell::new(None),
            writes: Cell::new(0),
        }
    }
}

impl ReplayLedgerPort for ReplayPort {
    type Error = MemoryError;

    fn load(&self, slot: ReplaySlotIdentity) -> Result<SlotObservation, Self::Error> {
        Ok(match *self.record.borrow() {
            Some(record) if record.slot() == slot => SlotObservation::Present(record),
            Some(_) | None => SlotObservation::Empty,
        })
    }

    fn compare_and_swap(
        &self,
        command: ReplayCommand,
    ) -> Result<ReplayCommitObservation, Self::Error> {
        let current_version = self.record.borrow().as_ref().map(ReplayRecord::version);
        if current_version != command.expected_version() {
            return Ok(ReplayCommitObservation::Stale {
                observed_version: current_version,
            });
        }
        let replacement = command.replacement();
        *self.record.borrow_mut() = Some(replacement);
        self.writes.set(self.writes.get().saturating_add(1));
        Ok(ReplayCommitObservation::Applied {
            version: replacement.version(),
            durability: ReplayDurabilityObservation::Synchronized,
        })
    }
}

struct RevocationPort {
    observation: SnapshotObservation,
    fail_load: bool,
}

impl RevocationViewPort for RevocationPort {
    type Error = MemoryError;

    fn load(&self, _source: SourceDigest) -> Result<SnapshotObservation, Self::Error> {
        if self.fail_load {
            Err(MemoryError::Injected)
        } else {
            Ok(self.observation.clone())
        }
    }

    fn compare_and_swap(
        &self,
        _command: RevocationCommand,
    ) -> Result<RevocationCommitObservation, Self::Error> {
        Err(MemoryError::Injected)
    }
}

fn replay_context() -> DurableReplayContext {
    DurableReplayContext::new(DurableReplayContextInput {
        tenant: [TENANT_BYTE; replay_ledger_core::DIGEST_BYTES],
        operation_profile: [OPERATION_BYTE; replay_ledger_core::DIGEST_BYTES],
        authority_generation: 1,
        ledger_namespace: [NAMESPACE_BYTE; replay_ledger_core::DIGEST_BYTES],
        attempt: [ATTEMPT_BYTE; replay_ledger_core::DIGEST_BYTES],
        durability: DurableReplayDurability::Synchronized,
    })
    .expect("complete replay context must be admitted")
}

fn revocation_context() -> DurableRevocationContext {
    DurableRevocationContext::new(DurableRevocationContextInput {
        source: [SOURCE_BYTE; revocation_view_core::DIGEST_BYTES],
        minimum_generation: 1,
        current_time_seconds: VERIFY_AT_SECONDS,
    })
    .expect("complete revocation context must be admitted")
}

fn stored_snapshot(
    completeness: CompletenessObservation,
    authenticity: AuthenticityObservation,
) -> StoredSnapshot {
    let snapshot = RevocationSnapshot::admit(SnapshotInput {
        source: Some(
            SourceDigest::new([SOURCE_BYTE; revocation_view_core::DIGEST_BYTES])
                .expect("source digest must be admitted"),
        ),
        generation: Some(SourceGeneration::new(1).expect("generation must be admitted")),
        observed_at: UnixTime::from_seconds(VERIFY_AT_SECONDS),
        effective_from: UnixTime::from_seconds(TOKEN_NOT_BEFORE_SECONDS),
        expires_at: UnixTime::from_seconds(TOKEN_EXPIRES_SECONDS),
        completeness,
        authenticity,
        entries: Vec::new(),
        entry_limit: EntryLimit::new(REVOCATION_ENTRY_LIMIT).expect("entry limit must be admitted"),
    })
    .expect("snapshot fixture must be admitted");
    let UpdateDecision::Install(command) = plan_update(snapshot, SnapshotObservation::Empty) else {
        panic!("empty view must produce an install command");
    };
    command.replacement().clone()
}

fn evaluate_with_durable<'request>(
    fixture: &InvocationFixture,
    request: &'request EnforcementRequest,
    signature: &SignerSignature,
    replay_port: &ReplayPort,
    revocation_port: &RevocationPort,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    let durable = BasaltDurableAuthority::new(
        replay_port,
        replay_context(),
        revocation_port,
        revocation_context(),
    );
    let policy = default_policy().expect("embedded policy must parse");
    let policy_ref = default_authority_policy_ref(request.contract());
    let raw = UcanEnforcementRequest::new(
        request,
        fixture.token.as_str(),
        BasaltUcanProtocolProfile::OnixCompactEd25519V1,
        EnforcementAction::DocumentWrite,
        nonce(),
        signature,
    );
    let boundaries = durable.boundaries(DurableAuthorityVerificationInputs {
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        key_resolver: &fixture.keys,
        proof_store: &fixture.proofs,
        caveat_policies: &DenyUnknownCaveats,
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    });
    verify_ucan_enforcement_request(&policy, &policy_ref, &raw, &boundaries)
}

#[test]
fn first_use_reserves_once_and_duplicate_denies_without_another_write() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = ReplayPort::new();
    let revocations = RevocationPort {
        observation: SnapshotObservation::Present(stored_snapshot(
            CompletenessObservation::Complete,
            AuthenticityObservation::Verified,
        )),
        fail_load: false,
    };

    assert!(evaluate_with_durable(&fixture, &request, &signature, &replay, &revocations).is_ok());
    let duplicate = evaluate_with_durable(&fixture, &request, &signature, &replay, &revocations)
        .expect_err("duplicate durable replay must deny");
    assert_eq!(duplicate.reason(), UcanEnforcementReason::ReplayDenied);
    assert_eq!(replay.writes.get(), 1);
}

#[test]
fn insufficient_revocation_evidence_denies_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());

    let cases = [
        RevocationPort {
            observation: SnapshotObservation::Present(stored_snapshot(
                CompletenessObservation::Partial,
                AuthenticityObservation::Verified,
            )),
            fail_load: false,
        },
        RevocationPort {
            observation: SnapshotObservation::Present(stored_snapshot(
                CompletenessObservation::Complete,
                AuthenticityObservation::Unverified,
            )),
            fail_load: false,
        },
        RevocationPort {
            observation: SnapshotObservation::Empty,
            fail_load: false,
        },
        RevocationPort {
            observation: SnapshotObservation::Unavailable,
            fail_load: true,
        },
    ];

    for revocations in cases {
        let replay = ReplayPort::new();
        let result = evaluate_with_durable(&fixture, &request, &signature, &replay, &revocations);
        assert!(result.is_err());
        assert_eq!(replay.writes.get(), 0);
    }
}
