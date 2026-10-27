//! Explicit in-process Basalt issuance and durable exchange replay.
//!
//! The embedder supplies admitted issuer custody, one stable holder signer and
//! public-network discovery ports. This module never resolves ambient keys,
//! invents an issuer endpoint, or treats requested recipient bits as possession.
// r[impl lattice.github_ci.attenuated_exchange]
// r[impl lattice.github_ci.evidence_boundary]

use std::io::Write;

use basalt::oidc_exchange::BoundedHttpDocumentPort;
use basalt::oidc_exchange::ExchangeReceipt;
use basalt::oidc_exchange::ExchangeRequest;
use basalt::oidc_exchange::IssuerAdmission;
use basalt::oidc_exchange::JwksCache;
use basalt::oidc_exchange::MantleOperation;
use basalt::oidc_exchange::OidcPolicy;
use basalt::oidc_exchange::ReplayCommit;
use basalt::oidc_exchange::ReplayFailure;
use basalt::oidc_exchange::ReplayKey;
use basalt::oidc_exchange::ReplayPort;
use basalt::ucan::TokenSigner;
use basalt::ucan::{self};

use super::core::CapabilityFacts;
use super::core::CapabilityPlan;
use super::core::Identity;
use super::core::Operation;
use super::github::OidcToken;

const VOCABULARY: &str = "onix.mantle.capability.v1";
const MILLIS_PER_SECOND: u64 = 1_000;
const MAX_RESOURCE_BYTES: usize = 4_096;
const MAX_REPLAY_ENTRIES: usize = 100_000;
const MUTATION_LOCK: &str = ".mutation";
const RESERVATION_BYTES: u64 = 8;

/// Redacted failure classes only: underlying token, provider and I/O text is omitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExchangeError {
    #[error("Basalt exchange policy or exact request does not match the plan")]
    Policy,
    #[error("Basalt exchange lifetime cannot fit the whole-second token contract")]
    Lifetime,
    #[error("Basalt issuer or stable holder signing capability is unavailable")]
    Signer,
    #[error("Basalt denied the in-process exchange")]
    Denied,
    #[error("Basalt issued token does not verify against the exact plan")]
    Grant,
    #[error("durable exchange replay is unavailable, unsafe, full or uncertain")]
    Replay,
}

/// Explicit borrowed production ports. Neither signer is loaded by this module.
pub struct BasaltHostPorts<'a> {
    pub policy: &'a OidcPolicy,
    pub cache: &'a mut JwksCache,
    pub documents: &'a mut BoundedHttpDocumentPort,
    pub replay: &'a mut DurableExchangeReplay,
    pub issuer: &'a IssuerAdmission,
    pub issuer_signer: &'a dyn TokenSigner,
    pub holder_signer: &'a dyn TokenSigner,
}

/// An embedder assembles reviewed SecretSpec issuer admission and credential
/// context signers here. There is intentionally no default or environment host.
pub trait BasaltIssuerHost {
    fn ports(&mut self) -> BasaltHostPorts<'_>;
}

impl BasaltIssuerHost for BasaltHostPorts<'_> {
    fn ports(&mut self) -> BasaltHostPorts<'_> {
        BasaltHostPorts {
            policy: self.policy,
            cache: &mut *self.cache,
            documents: &mut *self.documents,
            replay: &mut *self.replay,
            issuer: self.issuer,
            issuer_signer: self.issuer_signer,
            holder_signer: self.holder_signer,
        }
    }
}

/// A live UCAN in the producer's zeroizing CompactToken owner. Deliberately no
/// Debug, Clone, Serialize or public bearer constructor. Public receipt/facts
/// are observations, not reusable authorization or holder-possession proofs.
pub struct ExchangedUcan {
    token: ucan::CompactToken,
    holder: ucan::AudienceDid,
    resource: String,
    operation: Operation,
    facts: CapabilityFacts,
    receipt: ExchangeReceipt,
}

impl ExchangedUcan {
    pub fn expose_for_transport(&self) -> &str {
        self.token.as_str()
    }

    pub fn holder_did(&self) -> &str {
        self.holder.as_str()
    }

    pub fn facts(&self) -> &CapabilityFacts {
        &self.facts
    }

    pub fn receipt(&self) -> &ExchangeReceipt {
        &self.receipt
    }

    /// Sign only this exact verified grant. The caller supplies the native
    /// Basalt operation binding of the actual canonical service request and a
    /// fresh nonce; the receiver independently recomputes the bindings and
    /// verifies current token/proof authority and consumptive invocation replay.
    pub fn sign_invocation(
        &self,
        holder: &dyn TokenSigner,
        exact_resource: &str,
        ability: &str,
        operation: ucan::OperationBinding,
        nonce: [u8; 32],
    ) -> Result<[u8; 64], ExchangeError> {
        if exact_resource != self.resource || ability != self.operation.name() {
            return Err(ExchangeError::Grant);
        }
        let nonce = ucan::HolderInvocationNonce::try_from_slice(&nonce).map_err(|_| ExchangeError::Signer)?;
        let signature = ucan::sign_holder_invocation(
            &ucan::HolderInvocationSigningRequest::new(
                self.token.as_str(),
                &self.holder,
                exact_resource,
                ability,
                operation,
                nonce,
            ),
            holder,
            ucan::VerificationLimits::default(),
            ucan::HolderInvocationLimits::default(),
        )
        .map_err(|_| ExchangeError::Signer)?;
        signature.as_bytes().try_into().map_err(|_| ExchangeError::Signer)
    }
}

/// Consume a transient GitHub identity token through the real Basalt producer.
/// `plan.resource` is the core's semantic request/attempt identity. The caller
/// constructs `exact_resource` using the native Basalt Mantle verifier and
/// signs the resulting native operation binding, not a consumer-owned digest.
pub fn exchange(
    host: &mut dyn BasaltIssuerHost,
    token: OidcToken,
    plan: &CapabilityPlan,
    exact_project: &str,
    exact_resource: &str,
    now_ms: u64,
) -> Result<ExchangedUcan, ExchangeError> {
    let ports = host.ports();
    admit_request(ports.policy, plan, exact_project, exact_resource)?;
    let now_seconds = now_ms / MILLIS_PER_SECOND;
    let not_before_ms = now_seconds * MILLIS_PER_SECOND;
    let deadline_seconds = plan.expires_ms / MILLIS_PER_SECOND;
    let lifetime_seconds = deadline_seconds
        .checked_sub(now_seconds)
        .filter(|seconds| *seconds > 0)
        .ok_or(ExchangeError::Lifetime)?;
    if not_before_ms < plan.not_before_ms
        || plan.not_before_ms > now_ms
        || lifetime_seconds > ports.policy.time.max_ucan_lifetime_seconds
    {
        return Err(ExchangeError::Lifetime);
    }
    let holder_did = ports.holder_signer.issuer().map_err(|_| ExchangeError::Signer)?;
    let holder_key = ports.holder_signer.public_key().map_err(|_| ExchangeError::Signer)?;
    if holder_key.issuer().map_err(|_| ExchangeError::Signer)? != holder_did {
        return Err(ExchangeError::Signer);
    }
    let holder = ucan::AudienceDid::from(holder_did);
    let issuer_did = ports.issuer_signer.issuer().map_err(|_| ExchangeError::Signer)?;
    let issuer_key = ucan::Ed25519VerificationKey::new(
        issuer_did,
        ports.issuer_signer.public_key().map_err(|_| ExchangeError::Signer)?,
    )
    .map_err(|_| ExchangeError::Signer)?;
    let request = ExchangeRequest {
        audience: plan.audience.clone(),
        project: exact_project.to_owned(),
        operation: producer_operation(plan.operation),
        resource: exact_resource.to_owned(),
        lifetime_seconds,
    };
    let issued = basalt::oidc_exchange::exchange_to_recipient(
        token.expose_for_exchange(),
        now_seconds,
        ports.policy,
        &request,
        ports.cache,
        ports.documents,
        ports.replay,
        ports.issuer,
        ports.issuer_signer,
        &holder,
    )
    .map_err(|_| ExchangeError::Denied)?;
    let facts = verify_issued_grant(&issued.token, issuer_key, &holder, &request, plan, now_ms)?;
    Ok(ExchangedUcan {
        token: issued.token,
        holder,
        resource: request.resource,
        operation: plan.operation,
        facts,
        receipt: issued.receipt,
    })
}

fn verify_issued_grant(
    token: &ucan::CompactToken,
    issuer_key: ucan::Ed25519VerificationKey,
    holder: &ucan::AudienceDid,
    request: &ExchangeRequest,
    plan: &CapabilityPlan,
    now_ms: u64,
) -> Result<CapabilityFacts, ExchangeError> {
    let expected_issuer = issuer_key.issuer().clone();
    let now_seconds = now_ms / MILLIS_PER_SECOND;
    let context = ucan::VerificationContext::new(
        ucan::VerificationTime::from_unix_seconds(now_seconds),
        ucan::KeyResolutionContext::new(vec![issuer_key]),
        ucan::ProofCollection::empty(),
    );
    let verified = ucan::verify_compact_token(token, &context).map_err(|_| ExchangeError::Grant)?;
    let grants = verified.effective_delegation().capabilities().as_slice();
    let bounds = verified.effective_delegation().time_bounds();
    if verified.issuer() != &expected_issuer
        || verified.audience() != holder
        || !verified.proofs().as_slice().is_empty()
        || verified.capabilities().as_slice() != grants
        || grants.len() != 1
        || grants[0].resource != request.resource
        || grants[0].ability != plan.operation.name()
        || !grants[0].caveats.is_empty()
        || bounds.not_before_seconds() != now_seconds
        || bounds.expires_at_seconds() > plan.expires_ms / MILLIS_PER_SECOND
    {
        return Err(ExchangeError::Grant);
    }
    let facts = CapabilityFacts {
        audience: request.audience.clone(),
        project: Identity(*blake3::hash(request.project.as_bytes()).as_bytes()),
        resource: plan.resource,
        operations: [plan.operation].into_iter().collect(),
        not_before_ms: bounds.not_before_seconds().checked_mul(MILLIS_PER_SECOND).ok_or(ExchangeError::Lifetime)?,
        expires_ms: bounds.expires_at_seconds().checked_mul(MILLIS_PER_SECOND).ok_or(ExchangeError::Lifetime)?,
    };
    plan.admit(&facts, now_ms).map_err(|_| ExchangeError::Grant)?;
    Ok(facts)
}

fn admit_request(
    policy: &OidcPolicy,
    plan: &CapabilityPlan,
    project: &str,
    resource: &str,
) -> Result<(), ExchangeError> {
    if policy.capability.vocabulary != VOCABULARY
        || plan.audience != policy.capability.audience
        || project != policy.capability.project
        || Identity(*blake3::hash(project.as_bytes()).as_bytes()) != plan.project
        || !policy.capability.operations.contains(&producer_operation(plan.operation))
        || !resource.starts_with("mantle://gateway/v1/")
        || !resource.starts_with(&policy.capability.resource_prefix)
        || resource.len() > MAX_RESOURCE_BYTES
    {
        return Err(ExchangeError::Policy);
    }
    Ok(())
}

const fn producer_operation(operation: Operation) -> MantleOperation {
    match operation {
        Operation::Submit => MantleOperation::BuildSubmit,
        Operation::Status => MantleOperation::StatusRead,
        Operation::Events => MantleOperation::EventRead,
        Operation::Log => MantleOperation::LogRead,
        Operation::Result => MantleOperation::ResultRead,
        Operation::OwnedCancel => MantleOperation::OwnedAttemptCancel,
    }
}

/// Owner-private bounded create-only replay storage. Each digest-named file is
/// an atomic absent-to-reserved CAS containing only its expiration (8 bytes).
/// A create-new mutation owner serializes capacity admission across processes.
/// Successful reservation fsyncs file and directory before issuance can occur.
///
/// Crashes/I/O uncertainty retain the mutation owner and fail closed; never
/// reclaim an uncertain owner automatically. Reservations are not evicted or
/// pruned by this adapter. The operator must provision capacity and retain all
/// reservations until expiry before explicit offline recovery/rotation. This
/// is not a claim about power-loss behavior of an untrustworthy filesystem.
pub struct DurableExchangeReplay {
    root: cap_std::fs::Dir,
    maximum_entries: usize,
    owner: u32,
}

impl DurableExchangeReplay {
    /// The host supplies an already-opened dedicated directory capability. It
    /// must be owner-private (0700), and created files must have that same owner.
    pub fn new(root: cap_std::fs::Dir, maximum_entries: usize) -> Result<Self, ExchangeError> {
        if maximum_entries == 0 || maximum_entries > MAX_REPLAY_ENTRIES {
            return Err(ExchangeError::Replay);
        }
        let owner = directory_owner(&root).map_err(|_| ExchangeError::Replay)?;
        Ok(Self {
            root,
            maximum_entries,
            owner,
        })
    }

    fn reserve(&self, name: &str, expires: u64) -> Result<ReplayCommit, ReplayFailure> {
        if directory_owner(&self.root)? != self.owner {
            return Err(ReplayFailure::Invalid);
        }
        let mut lock = create_private(&self.root, MUTATION_LOCK, self.owner)?;
        lock.write_all(b"basalt-exchange-replay.v1\n").map_err(|_| ReplayFailure::Unavailable)?;
        lock.sync_all().map_err(|_| ReplayFailure::Unavailable)?;
        sync_directory(&self.root)?;
        let decision = self.reserve_locked(name, expires)?;
        self.root.remove_file(MUTATION_LOCK).map_err(|_| ReplayFailure::Unavailable)?;
        sync_directory(&self.root)?;
        Ok(decision)
    }

    fn reserve_locked(&self, name: &str, expires: u64) -> Result<ReplayCommit, ReplayFailure> {
        match self.root.symlink_metadata(name) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.len() != RESERVATION_BYTES {
                    return Err(ReplayFailure::Invalid);
                }
                return Ok(ReplayCommit::Duplicate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(ReplayFailure::Unavailable),
        }
        let mut count = 0_usize;
        for entry in self.root.read_dir(".").map_err(|_| ReplayFailure::Unavailable)? {
            let entry = entry.map_err(|_| ReplayFailure::Unavailable)?;
            let file_name = entry.file_name();
            if file_name == MUTATION_LOCK {
                continue;
            }
            count = count.checked_add(1).ok_or(ReplayFailure::Invalid)?;
            if count >= self.maximum_entries {
                return Err(ReplayFailure::Unavailable);
            }
            if !entry.file_type().map_err(|_| ReplayFailure::Unavailable)?.is_file()
                || !file_name.to_str().is_some_and(digest_name)
            {
                return Err(ReplayFailure::Invalid);
            }
        }
        let mut reservation = create_private(&self.root, name, self.owner)?;
        reservation.write_all(&expires.to_le_bytes()).map_err(|_| ReplayFailure::Unavailable)?;
        reservation.sync_all().map_err(|_| ReplayFailure::Unavailable)?;
        sync_directory(&self.root)?;
        Ok(ReplayCommit::Fresh)
    }
}

impl ReplayPort for DurableExchangeReplay {
    fn commit(&mut self, key: &ReplayKey, expires_at_seconds: u64) -> Result<ReplayCommit, ReplayFailure> {
        let name = key
            .as_str()
            .strip_prefix("blake3:")
            .filter(|name| digest_name(name))
            .ok_or(ReplayFailure::Invalid)?;
        if expires_at_seconds == 0 {
            return Err(ReplayFailure::Invalid);
        }
        self.reserve(name, expires_at_seconds)
    }
}

fn digest_name(name: &str) -> bool {
    name.len() == 64 && name.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sync_directory(root: &cap_std::fs::Dir) -> Result<(), ReplayFailure> {
    root.open(".").and_then(|file| file.sync_all()).map_err(|_| ReplayFailure::Unavailable)
}

#[cfg(unix)]
fn directory_owner(root: &cap_std::fs::Dir) -> Result<u32, ReplayFailure> {
    use std::os::unix::fs::MetadataExt as _;
    let metadata = root
        .open(".")
        .map_err(|_| ReplayFailure::Unavailable)?
        .into_std()
        .metadata()
        .map_err(|_| ReplayFailure::Unavailable)?;
    if !metadata.is_dir() || metadata.mode() & 0o777 != 0o700 {
        return Err(ReplayFailure::Invalid);
    }
    Ok(metadata.uid())
}

#[cfg(unix)]
fn create_private(root: &cap_std::fs::Dir, name: &str, owner: u32) -> Result<std::fs::File, ReplayFailure> {
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::PermissionsExt as _;
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let file = root.open_with(name, &options).map_err(|_| ReplayFailure::Unavailable)?.into_std();
    let metadata = file.metadata().map_err(|_| ReplayFailure::Unavailable)?;
    if !metadata.is_file() || metadata.uid() != owner || metadata.nlink() != 1 {
        return Err(ReplayFailure::Invalid);
    }
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .map_err(|_| ReplayFailure::Unavailable)?;
    Ok(file)
}

#[cfg(not(unix))]
fn directory_owner(_root: &cap_std::fs::Dir) -> Result<u32, ReplayFailure> {
    Err(ReplayFailure::Unavailable)
}

#[cfg(not(unix))]
fn create_private(_root: &cap_std::fs::Dir, _name: &str, _owner: u32) -> Result<std::fs::File, ReplayFailure> {
    Err(ReplayFailure::Unavailable)
}

// r[verify lattice.github_ci.attenuated_exchange]
// r[verify lattice.github_ci.evidence_boundary]
#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    const NOW_MS: u64 = 10_000;
    const EXPIRY_MS: u64 = 70_000;
    const SERVICE: &str = "did:key:z6Mkmantle-service";
    const PROJECT: &str = "onix-stack";
    const RESOURCE: &str = "did:key:z6Mkmantle-service/projects/onix-stack/attempt/private-attempt/";

    fn private_directory() -> tempfile::TempDir {
        let directory = tempfile::tempdir().expect("isolated replay directory");
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private replay owner");
        directory
    }

    fn replay_at(path: &std::path::Path, capacity: usize) -> DurableExchangeReplay {
        let root = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())
            .expect("explicit test directory capability");
        DurableExchangeReplay::new(root, capacity).expect("admitted private replay directory")
    }

    #[test]
    fn durable_reservation_survives_reopen_and_cannot_extend_or_replace() {
        let directory = private_directory();
        let name = "a".repeat(64);
        assert_eq!(replay_at(directory.path(), 2).reserve(&name, 70), Ok(ReplayCommit::Fresh));
        assert_eq!(replay_at(directory.path(), 2).reserve(&name, 90), Ok(ReplayCommit::Duplicate));
        assert_eq!(std::fs::read(directory.path().join(&name)).expect("durable reservation"), 70_u64.to_le_bytes());
        let metadata = std::fs::metadata(directory.path().join(&name)).expect("reservation ownership");
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(metadata.nlink(), 1);
        let second = "b".repeat(64);
        assert_eq!(replay_at(directory.path(), 2).reserve(&second, 70), Ok(ReplayCommit::Fresh));
        assert_eq!(replay_at(directory.path(), 2).reserve(&"c".repeat(64), 70), Err(ReplayFailure::Unavailable));
        assert_eq!(
            std::fs::read(directory.path().join(name)).expect("capacity denial retains prior reservation"),
            70_u64.to_le_bytes()
        );
    }

    #[test]
    fn concurrent_owners_cannot_both_reserve_the_same_key() {
        let directory = private_directory();
        let gate = std::sync::Arc::new(std::sync::Barrier::new(2));
        let mut workers = Vec::new();
        for _ in 0..2 {
            let path = directory.path().to_owned();
            let gate = gate.clone();
            workers.push(std::thread::spawn(move || {
                let replay = replay_at(&path, 8);
                gate.wait();
                replay.reserve(&"d".repeat(64), 70)
            }));
        }
        let outcomes: Vec<_> = workers.into_iter().map(|worker| worker.join().expect("reservation owner")).collect();
        assert_eq!(outcomes.iter().filter(|outcome| **outcome == Ok(ReplayCommit::Fresh)).count(), 1);
        assert!(outcomes.iter().all(|outcome| matches!(
            outcome,
            Ok(ReplayCommit::Fresh | ReplayCommit::Duplicate) | Err(ReplayFailure::Unavailable)
        )));
        assert_eq!(replay_at(directory.path(), 8).reserve(&"d".repeat(64), 70), Ok(ReplayCommit::Duplicate));
    }

    #[test]
    fn unsafe_root_and_uncertain_publication_fail_closed_without_reclaiming() {
        let directory = private_directory();
        let name = "e".repeat(64);
        // An interrupted write must not become a fresh reservation or be replaced.
        std::fs::write(directory.path().join(&name), [0_u8; 3]).expect("interrupted publication fixture");
        let replay = replay_at(directory.path(), 8);
        assert_eq!(replay.reserve(&name, 70), Err(ReplayFailure::Invalid));
        assert!(directory.path().join(MUTATION_LOCK).exists());
        assert_eq!(replay.reserve(&"f".repeat(64), 70), Err(ReplayFailure::Unavailable));
        assert_eq!(std::fs::read(directory.path().join(name)).expect("uncertain bytes retained"), [0_u8; 3]);
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o750))
            .expect("unsafe root fixture");
        let root =
            cap_std::fs::Dir::open_ambient_dir(directory.path(), cap_std::ambient_authority()).expect("test root");
        assert!(matches!(DurableExchangeReplay::new(root, 8), Err(ExchangeError::Replay)));
    }

    fn fixture_plan() -> CapabilityPlan {
        CapabilityPlan {
            audience: SERVICE.to_owned(),
            project: Identity(*blake3::hash(PROJECT.as_bytes()).as_bytes()),
            resource: Identity(*blake3::hash(b"private-attempt").as_bytes()),
            operation: Operation::Events,
            not_before_ms: NOW_MS,
            expires_ms: EXPIRY_MS,
        }
    }

    fn fixture_request() -> ExchangeRequest {
        ExchangeRequest {
            audience: SERVICE.to_owned(),
            project: PROJECT.to_owned(),
            operation: MantleOperation::EventRead,
            resource: RESOURCE.to_owned(),
            lifetime_seconds: 60,
        }
    }

    fn signed_token(
        signer: &dyn TokenSigner,
        recipient: &ucan::AudienceDid,
        resource: &str,
        ability: &str,
        expiry_seconds: u64,
    ) -> ucan::CompactToken {
        ucan::issue_token_with_signer(
            signer,
            recipient,
            &ucan::CapabilitySet::single(resource, ability).expect("fixture exact capability"),
            &ucan::ProofReferences::empty(),
            ucan::TokenTimeBounds::new(NOW_MS / 1_000, expiry_seconds).expect("fixture token lifetime"),
        )
        .expect("real signed UCAN fixture")
    }

    #[test]
    fn authenticated_grant_rejects_other_issuer_holder_scope_ability_and_expiry() {
        let issuer = ucan::Ed25519InMemorySigner::from_seed_bytes([31; 32]);
        let other_issuer = ucan::Ed25519InMemorySigner::from_seed_bytes([32; 32]);
        let holder = ucan::AudienceDid::from(
            ucan::Ed25519InMemorySigner::from_seed_bytes([33; 32]).issuer().expect("holder DID"),
        );
        let other_holder = ucan::AudienceDid::from(
            ucan::Ed25519InMemorySigner::from_seed_bytes([34; 32]).issuer().expect("other holder DID"),
        );
        let request = fixture_request();
        let plan = fixture_plan();
        let valid = signed_token(&issuer, &holder, RESOURCE, "mantle/event/read", 70);
        let facts = verify_issued_grant(
            &valid,
            issuer.verification_key().expect("trusted issuer key"),
            &holder,
            &request,
            &plan,
            NOW_MS,
        )
        .expect("actual signed exact grant");
        assert_eq!(facts.expires_ms, EXPIRY_MS);
        assert_eq!(facts.operations, [Operation::Events].into_iter().collect());
        for token in [
            signed_token(&other_issuer, &holder, RESOURCE, "mantle/event/read", 70),
            signed_token(&issuer, &other_holder, RESOURCE, "mantle/event/read", 70),
            signed_token(
                &issuer,
                &holder,
                "did:key:foreign/projects/onix-stack/attempt/private-attempt/",
                "mantle/event/read",
                70,
            ),
            signed_token(&issuer, &holder, RESOURCE, "mantle/build/submit", 70),
            signed_token(&issuer, &holder, RESOURCE, "mantle/event/read", 71),
        ] {
            assert_eq!(
                verify_issued_grant(
                    &token,
                    issuer.verification_key().expect("trusted issuer key"),
                    &holder,
                    &request,
                    &plan,
                    NOW_MS
                ),
                Err(ExchangeError::Grant)
            );
        }
    }

    #[test]
    fn policy_admission_preserves_service_project_operation_and_resource() {
        let mut policy = basalt::oidc_exchange::parse_policy(include_str!(
            "../../vendor-cargo-deps/basalt/contracts/oidc-exchange/generated/policy.json"
        ))
        .expect("accepted producer policy");
        policy.capability.resource_prefix = "mantle://gateway/v1/".to_owned();
        policy.capability.operations.push(MantleOperation::EventRead);
        let plan = fixture_plan();
        let resource = "mantle://gateway/v1/exact-producer-resource";
        assert_eq!(admit_request(&policy, &plan, PROJECT, resource), Ok(()));
        assert_eq!(admit_request(&policy, &plan, "other-project", resource), Err(ExchangeError::Policy));
        assert_eq!(admit_request(&policy, &plan, PROJECT, RESOURCE), Err(ExchangeError::Policy));
        let mut wrong_service = plan.clone();
        wrong_service.audience = "did:key:other-service".to_owned();
        assert_eq!(admit_request(&policy, &wrong_service, PROJECT, resource), Err(ExchangeError::Policy));
        policy.capability.operations.retain(|operation| *operation != MantleOperation::EventRead);
        assert_eq!(admit_request(&policy, &plan, PROJECT, resource), Err(ExchangeError::Policy));
    }

    #[derive(Default)]
    struct InvocationReplay(std::cell::RefCell<std::collections::BTreeSet<(String, [u8; 32])>>);

    impl ucan::HolderInvocationReplayAdmission for InvocationReplay {
        fn admit_holder_invocation(
            &self,
            candidate: &ucan::HolderInvocationReplayCandidate<'_>,
        ) -> Result<(), ucan::HolderInvocationReplayError> {
            if self
                .0
                .borrow_mut()
                .insert((candidate.audience().as_str().to_owned(), *candidate.nonce().as_bytes()))
            {
                Ok(())
            } else {
                Err(ucan::HolderInvocationReplayError::Duplicate)
            }
        }
    }

    #[test]
    fn native_holder_invocations_preserve_exact_actions_and_consume_replay_once() {
        use basalt::mantle::Invocation;
        use basalt::mantle::Request;
        use basalt::mantle::Verifier;
        use basalt::ucan_enforcement::UcanEnforcementBoundaries;
        use basalt::ucan_enforcement::bind_enforcement_action;

        let issuer = ucan::Ed25519InMemorySigner::from_seed_bytes([41; 32]);
        let holder_signer = ucan::Ed25519InMemorySigner::from_seed_bytes([42; 32]);
        let wrong_holder = ucan::Ed25519InMemorySigner::from_seed_bytes([43; 32]);
        let holder = ucan::AudienceDid::from(holder_signer.issuer().expect("holder DID"));
        let policy = basalt::default_policy().expect("native Mantle policy");
        let policy_ref = basalt::authority_core::BasaltPolicyRef::new(
            format!("blake3:{}", blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()),
            basalt::SUPPORTED_SCHEMA_VERSION,
            "mantle-build-submit",
        );
        let roots = [issuer.issuer().expect("issuer DID").as_str().to_owned()];
        let verifier = Verifier {
            policy: &policy,
            policy_ref: &policy_ref,
            generation: 7,
            endpoint: "private-gateway",
            audience: SERVICE,
            trusted_roots: &roots,
        };
        let keys = ucan::KeyResolutionContext::new(vec![
            issuer.verification_key().expect("issuer key"),
            holder_signer.verification_key().expect("holder key"),
        ]);
        let proofs = ucan::ProofCollection::empty();
        let oidc_policy = basalt::oidc_exchange::parse_policy(include_str!(
            "../../vendor-cargo-deps/basalt/contracts/oidc-exchange/generated/policy.json"
        ))
        .expect("OIDC policy fixture");
        for (operation, native) in [
            (Operation::Submit, basalt::mantle::Operation::BuildSubmit),
            (Operation::Status, basalt::mantle::Operation::StatusRead),
            (Operation::Events, basalt::mantle::Operation::EventRead),
            (Operation::Log, basalt::mantle::Operation::LogRead),
            (Operation::Result, basalt::mantle::Operation::ResultRead),
            (Operation::OwnedCancel, basalt::mantle::Operation::OwnedAttemptCancel),
        ] {
            let request = Request {
                operation: native,
                subject: holder.as_str().to_owned(),
                account: "account-a".to_owned(),
                project: PROJECT.to_owned(),
                endpoint: "private-gateway".to_owned(),
                audience: SERVICE.to_owned(),
                generation: 7,
                request_digest: [51; 32],
                target: "attempt-a".to_owned(),
            };
            let exact =
                basalt::mantle::canonical_enforcement(&request, &policy_ref).expect("native canonical signing request");
            let binding = bind_enforcement_action(native.action(), &exact).expect("native binding");
            let token = signed_token(&issuer, &holder, exact.resource(), exact.ability(), 70);
            let exchange_request = ExchangeRequest {
                audience: SERVICE.to_owned(),
                project: PROJECT.to_owned(),
                operation: producer_operation(operation),
                resource: exact.resource().to_owned(),
                lifetime_seconds: 60,
            };
            let mut plan = fixture_plan();
            plan.operation = operation;
            let facts = verify_issued_grant(
                &token,
                issuer.verification_key().expect("issuer key"),
                &holder,
                &exchange_request,
                &plan,
                NOW_MS,
            )
            .expect("cryptographically verified exact holder grant");
            let exchanged = ExchangedUcan {
                token,
                holder: holder.clone(),
                resource: exact.resource().to_owned(),
                operation,
                facts,
                // Honest denied metadata is deliberately insufficient authority:
                // this test admits only the real signed grant above.
                receipt: basalt::oidc_exchange::receipt_for_failure(
                    &oidc_policy,
                    &exchange_request,
                    "fixture-no-oidc-exchange",
                    None,
                ),
            };
            assert_eq!(
                exchanged.sign_invocation(
                    &holder_signer,
                    "mantle://gateway/v1/other",
                    exact.ability(),
                    binding,
                    [61; 32],
                ),
                Err(ExchangeError::Grant)
            );
            assert_eq!(
                exchanged.sign_invocation(&wrong_holder, exact.resource(), exact.ability(), binding, [61; 32],),
                Err(ExchangeError::Signer)
            );
            let signature = ucan::SignerSignature::from_bytes(
                exchanged
                    .sign_invocation(&holder_signer, exact.resource(), exact.ability(), binding, [61; 32])
                    .expect("consumer signs actual native binding"),
            );
            let invocation = Invocation {
                compact_token: exchanged.expose_for_transport(),
                nonce: ucan::HolderInvocationNonce::from_bytes([61; 32]),
                holder_signature: &signature,
            };
            let replay = InvocationReplay::default();
            let boundaries = UcanEnforcementBoundaries {
                verification_time: ucan::VerificationTime::from_unix_seconds(NOW_MS / 1_000),
                key_resolver: &keys,
                proof_store: &proofs,
                revocations: &ucan::NoRevocations,
                caveat_policies: &ucan::token::DenyUnknownCaveats,
                replay: &replay,
                verification_limits: ucan::VerificationLimits::default(),
                holder_limits: ucan::HolderInvocationLimits::default(),
            };
            let mut altered = request.clone();
            altered.request_digest[0] ^= 1;
            assert!(verifier.verify(altered, &invocation, &boundaries).is_err());
            assert!(replay.0.borrow().is_empty(), "substitution cannot consume replay");
            let session = verifier
                .verify(request.clone(), &invocation, &boundaries)
                .expect("native receiver accepts consumer holder invocation");
            session
                .require(native, &request.target, request.request_digest)
                .expect("exact operation remains live");
            verifier.revalidate(&session, &boundaries).expect("non-consuming revalidation");
            assert!(verifier.verify(request, &invocation, &boundaries).is_err(), "nonce cannot be spent twice");
            assert_eq!(replay.0.borrow().len(), 1);
        }
    }
}
