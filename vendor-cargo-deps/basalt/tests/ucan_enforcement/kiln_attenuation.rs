use crate::support::*;
use basalt::authority_core::BasaltPolicyRef;
use basalt::kiln::{
    Denied, ImplementationFraming, ImplementationScope, Invocation, LegacyFraming, Operation,
    PathScope, Request, Session, SourceAttempt, SourceOperation, SourceRequest, SourceScope,
    SourceSession, Verifier,
};

const AUDIENCE: &str = "kiln-service-a";
const GENERATION: u64 = 7;
const PROJECT: &str = "OnixResearch/kiln";
const WORK: &str = "kiln.implementation.work";
const REPOSITORY: &str = "OnixResearch/kiln";
const SECOND_DELEGATE_SEED_BYTE: u8 = 43;
const SECOND_HOLDER_SEED_BYTE: u8 = 47;
const DELEGATE: &str = "kiln/delegate";
const SOURCE_DELEGATE: &str = "kiln/source/delegate";

const ALL_SOURCE_OPERATIONS: [SourceOperation; 8] = [
    SourceOperation::Discover,
    SourceOperation::ReadContent,
    SourceOperation::ReadHistory,
    SourceOperation::PrepareWorkspace,
    SourceOperation::ModifySource,
    SourceOperation::CreateSource,
    SourceOperation::Delegate,
    SourceOperation::RetrieveResult,
];

fn did(seed: u8) -> String {
    signer(seed).issuer().unwrap().as_str().into()
}

fn policy_ref() -> BasaltPolicyRef {
    BasaltPolicyRef::new(
        format!(
            "blake3:{}",
            blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()
        ),
        basalt::SUPPORTED_SCHEMA_VERSION,
        "kiln-dispatch",
    )
}

const fn verifier<'a>(
    policy: &'a basalt::Policy,
    policy_ref: &'a BasaltPolicyRef,
    roots: &'a [String],
    max_delegation_depth: u8,
) -> Verifier<'a> {
    Verifier {
        policy,
        policy_ref,
        generation: GENERATION,
        audience: AUDIENCE,
        trusted_roots: roots,
        max_delegation_depth,
        implementation_v1: LegacyFraming::Accept,
    }
}

fn implementation(operation: Operation, holder: u8, project: &str, attempt: &str) -> Request {
    Request {
        operation,
        subject: did(holder),
        audience: AUDIENCE.into(),
        project: project.into(),
        work: WORK.into(),
        work_version: 3,
        attempt: attempt.into(),
        fence: 5,
        generation: GENERATION,
        request_digest: [41; 32],
        target: "candidate/approved".into(),
        candidate: if operation.requires_candidate() {
            [43; 32]
        } else {
            [0; 32]
        },
        framing: ImplementationFraming::V2,
    }
}

fn project_prefix(project: &str) -> String {
    ImplementationScope {
        audience: AUDIENCE,
        generation: GENERATION,
        project: Some(project),
        work: None,
        work_version: None,
        attempt: None,
        fence: None,
    }
    .prefix()
    .unwrap()
}

fn work_prefix() -> String {
    ImplementationScope {
        audience: AUDIENCE,
        generation: GENERATION,
        project: Some(PROJECT),
        work: Some(WORK),
        work_version: Some(3),
        attempt: None,
        fence: None,
    }
    .prefix()
    .unwrap()
}

fn attempt_prefix(attempt: &str) -> String {
    ImplementationScope {
        audience: AUDIENCE,
        generation: GENERATION,
        project: Some(PROJECT),
        work: Some(WORK),
        work_version: Some(3),
        attempt: Some(attempt),
        fence: Some(5),
    }
    .prefix()
    .unwrap()
}

fn path(components: &[&str]) -> Vec<String> {
    components
        .iter()
        .map(|component| (*component).into())
        .collect()
}

fn source(
    operation: SourceOperation,
    holder: u8,
    components: &[&str],
    scope: PathScope,
) -> SourceRequest {
    SourceRequest {
        operation,
        subject: did(holder),
        audience: AUDIENCE.into(),
        generation: GENERATION,
        repository: REPOSITORY.into(),
        attempt: None,
        path: path(components),
        scope,
        snapshot: "refs/heads/main@abc123".into(),
        view: "view-binding-1".into(),
        request_digest: [51; 32],
    }
}

fn source_prefix(components: &[&str], terminal: Option<PathScope>) -> String {
    SourceScope {
        audience: AUDIENCE,
        generation: GENERATION,
        repository: REPOSITORY,
        attempt: None,
        path: &path(components),
        terminal,
    }
    .prefix()
    .unwrap()
}

/// Issue one hop from `issuer` to `audience` with `grants`, proved by `parent`.
fn issue(
    issuer: u8,
    audience: u8,
    grants: &[(&str, &str)],
    parent: Option<&CompactToken>,
) -> CompactToken {
    let capabilities = grants
        .iter()
        .map(|(resource, ability)| {
            CapabilityDocument::new((*resource).into(), (*ability).into()).unwrap()
        })
        .collect();
    issue_documents(issuer, audience, capabilities, parent.as_slice())
}

/// Issue one hop with explicit capability documents and any number of parents.
fn issue_documents(
    issuer: u8,
    audience: u8,
    capabilities: Vec<CapabilityDocument>,
    parents: &[&CompactToken],
) -> CompactToken {
    let proofs = if parents.is_empty() {
        ProofReferences::empty()
    } else {
        ProofReferences::from_references(
            parents
                .iter()
                .map(|parent| proof_reference(parent))
                .collect(),
        )
    };
    issue_token_with_signer(
        &signer(issuer),
        &AudienceDid::from(signer(audience).issuer().unwrap()),
        &CapabilitySet::new(capabilities).unwrap(),
        &proofs,
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).unwrap(),
    )
    .unwrap()
}

/// The holder's leaf token plus every ancestor proof.
fn chain(holder: u8, leaf: CompactToken, ancestors: Vec<CompactToken>) -> InvocationFixture {
    InvocationFixture {
        token: leaf,
        audience: AudienceDid::from(signer(holder).issuer().unwrap()),
        holder: signer(holder),
        keys: KeyResolutionContext::new(
            [
                ISSUER_SEED_BYTE,
                DELEGATE_SEED_BYTE,
                SECOND_DELEGATE_SEED_BYTE,
                HOLDER_SEED_BYTE,
                SECOND_HOLDER_SEED_BYTE,
                WRONG_ISSUER_SEED_BYTE,
            ]
            .into_iter()
            .map(|seed| signer(seed).verification_key().unwrap())
            .collect(),
        ),
        trusted_roots: vec![did(ISSUER_SEED_BYTE)],
        proofs: ProofCollection::from_tokens(ancestors),
    }
}

fn boundaries<'a>(
    fixture: &'a InvocationFixture,
    replay: &'a RecordingReplay<'_>,
    revocations: &'a dyn RevocationChecker,
) -> UcanEnforcementBoundaries<'a> {
    UcanEnforcementBoundaries {
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        key_resolver: &fixture.keys,
        trusted_roots: &fixture.trusted_roots,
        proof_store: &fixture.proofs,
        revocations,
        caveat_policies: &DenyUnknownCaveats,
        replay,
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    }
}

fn invocation<'a>(
    fixture: &'a InvocationFixture,
    signature: &'a SignerSignature,
) -> Invocation<'a> {
    Invocation {
        compact_token: fixture.token.as_str(),
        nonce: nonce(),
        holder_signature: signature,
    }
}

/// Verify and revalidate one implementation request; return the replay call count.
fn admit(
    verifier: &Verifier<'_>,
    request: &Request,
    fixture: &InvocationFixture,
) -> (Result<(), Denied>, u32) {
    let exact = verifier.enforcement(request).unwrap();
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = sign_operation(fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    let bounds = boundaries(fixture, &replay, &NoRevocations);
    let result = verifier
        .verify(request.clone(), &invocation(fixture, &signature), &bounds)
        .and_then(|session| verifier.revalidate(&session, &bounds));
    (result, replay.calls.get())
}

/// Verify and revalidate one source request; return the replay call count.
fn admit_source(
    verifier: &Verifier<'_>,
    request: &SourceRequest,
    fixture: &InvocationFixture,
) -> (Result<(), Denied>, u32) {
    let exact = verifier.source_enforcement(request).unwrap();
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = sign_operation(fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    let bounds = boundaries(fixture, &replay, &NoRevocations);
    let result = verifier
        .verify_source(request.clone(), &invocation(fixture, &signature), &bounds)
        .and_then(|session| verifier.revalidate_source(&session, &bounds));
    (result, replay.calls.get())
}

/// `CompactToken` is not `Clone`; re-admit the same bytes.
fn copy(token: &CompactToken) -> CompactToken {
    CompactToken::new(token.as_str()).unwrap()
}

fn hex(text: &str) -> String {
    use std::fmt::Write as _;
    text.bytes().fold(String::new(), |mut out, byte| {
        write!(out, "{byte:02x}").unwrap();
        out
    })
}

#[test]
// r[verify basalt.kiln_authority.attenuable.scenario.cross_holder]
// r[verify basalt.kiln_authority.delegation.chain.scenario.scoped]
fn kiln_v2_project_ceiling_delegates_to_different_holders() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let project = project_prefix(PROJECT);
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );

    let first = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&first).unwrap();
    assert!(exact.resource().starts_with(&project));
    assert!(exact.resource().starts_with(&attempt_prefix("attempt-a")));
    assert!(!exact.resource().contains(&hex(&first.subject)));
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    assert_eq!(admit(&verifier, &first, &fixture), (Ok(()), 1));

    // A second holder receives an attempt ceiling, not an exact leaf.
    let second = implementation(
        Operation::Dispatch,
        SECOND_HOLDER_SEED_BYTE,
        PROJECT,
        "attempt-b",
    );
    let attempt = attempt_prefix("attempt-b");
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        SECOND_HOLDER_SEED_BYTE,
        &[(&attempt, "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(SECOND_HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    assert_eq!(admit(&verifier, &second, &fixture), (Ok(()), 1));

    // The attempt ceiling does not reach a sibling attempt of the same work.
    let sibling = implementation(
        Operation::Dispatch,
        SECOND_HOLDER_SEED_BYTE,
        PROJECT,
        "attempt-c",
    );
    assert_eq!(admit(&verifier, &sibling, &fixture), (Err(Denied), 0));

    // Revoking the delegator's ceiling denies a later revalidation.
    let exact = verifier.enforcement(&second).unwrap();
    let binding = bind_enforcement_action(second.operation.action(), &exact).unwrap();
    let signature = sign_operation(&fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    let session = verifier
        .verify(
            second,
            &invocation(&fixture, &signature),
            &boundaries(&fixture, &replay, &NoRevocations),
        )
        .unwrap();
    let revoked = RevocationList::from_references(vec![proof_reference(&ceiling)]);
    assert_eq!(
        verifier.revalidate(&session, &boundaries(&fixture, &replay, &revoked)),
        Err(Denied)
    );
    assert_eq!(replay.calls.get(), 1);
}

#[test]
// r[verify basalt.kiln_authority.delegation.scenario.widen]
fn kiln_v2_delegation_cannot_widen_the_parent_ceiling() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 2);
    let project = project_prefix(PROJECT);
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );

    // Another project is outside the ceiling.
    let foreign = implementation(
        Operation::Dispatch,
        HOLDER_SEED_BYTE,
        "Other/repo",
        "attempt-a",
    );
    let exact = verifier.enforcement(&foreign).unwrap();
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    assert_eq!(admit(&verifier, &foreign, &fixture), (Err(Denied), 0));

    // Another operation is outside the ceiling.
    let publish = implementation(
        Operation::PublicationPublish,
        HOLDER_SEED_BYTE,
        PROJECT,
        "attempt-a",
    );
    let exact = verifier.enforcement(&publish).unwrap();
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/publication/publish")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    assert_eq!(admit(&verifier, &publish, &fixture), (Err(Denied), 0));

    // A narrowed attempt ceiling cannot be re-widened to the whole work.
    let attempt = attempt_prefix("attempt-a");
    let narrowed = issue(
        DELEGATE_SEED_BYTE,
        SECOND_DELEGATE_SEED_BYTE,
        &[(&attempt, "kiln/dispatch"), (&attempt, DELEGATE)],
        Some(&ceiling),
    );
    let work = work_prefix();
    let widened = issue(
        SECOND_DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&work, "kiln/dispatch")],
        Some(&narrowed),
    );
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let fixture = chain(HOLDER_SEED_BYTE, widened, vec![ceiling, narrowed]);
    assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.missing_delegate]
fn kiln_v2_delegator_without_explicit_delegate_is_refused() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 2);
    let project = project_prefix(PROJECT);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&request).unwrap();
    let narrow_delegate = attempt_prefix("attempt-b");
    for held in [
        // Dispatch alone does not permit delegation.
        vec![(project.as_str(), "kiln/dispatch")],
        // A wildcard ability is not the explicit delegate ability.
        vec![(project.as_str(), "*")],
        // The source-family delegate ability does not cover implementation grants.
        vec![
            (project.as_str(), "kiln/dispatch"),
            (project.as_str(), SOURCE_DELEGATE),
        ],
        // Delegation over a sibling attempt does not cover this attempt.
        vec![
            (project.as_str(), "kiln/dispatch"),
            (narrow_delegate.as_str(), DELEGATE),
        ],
    ] {
        let ceiling = issue(ISSUER_SEED_BYTE, DELEGATE_SEED_BYTE, &held, None);
        let leaf = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(exact.resource(), "kiln/dispatch")],
            Some(&ceiling),
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
        assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
    }
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.depth]
fn kiln_v2_delegation_depth_is_bounded() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let project = project_prefix(PROJECT);
    let work = work_prefix();
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier(&policy, &reference, &roots, 0)
        .enforcement(&request)
        .unwrap();

    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );
    let middle = issue(
        DELEGATE_SEED_BYTE,
        SECOND_DELEGATE_SEED_BYTE,
        &[(&work, "kiln/dispatch"), (&work, DELEGATE)],
        Some(&ceiling),
    );
    let leaf = issue(
        SECOND_DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&middle),
    );
    let two_hops = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling), middle]);
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 1),
            &request,
            &two_hops
        ),
        (Err(Denied), 0)
    );
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 2),
            &request,
            &two_hops
        ),
        (Ok(()), 1)
    );

    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&ceiling),
    );
    let one_hop = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 0),
            &request,
            &one_hop
        ),
        (Err(Denied), 0)
    );
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        None,
    );
    let direct = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    assert_eq!(
        admit(&verifier(&policy, &reference, &roots, 0), &request, &direct),
        (Ok(()), 1)
    );
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.aligned]
fn kiln_v2_grants_must_stop_on_segment_boundaries() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&request).unwrap();
    let project = project_prefix(PROJECT);
    // Cut inside the project's hex bytes: it would also cover `OnixResearch/kilnx`.
    let partial = &project[..project.len() - 3];
    assert!(exact.resource().starts_with(partial));

    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(partial, "kiln/dispatch"), (partial, DELEGATE)],
        None,
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));

    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(partial, "kiln/dispatch")],
        None,
    );
    let direct = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    assert_eq!(admit(&verifier, &request, &direct), (Err(Denied), 0));
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.root]
fn kiln_attenuable_chain_root_must_be_a_trusted_issuer() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 2);
    let project = project_prefix(PROJECT);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&request).unwrap();
    let subtree = source_prefix(&["a"], None);
    let read = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "b"],
        PathScope::Entry,
    );
    let entry = verifier.source_enforcement(&read).unwrap();

    // An untrusted issuer granting a ceiling plus delegate is a root substitution.
    for (resource, ability, delegate, leaf) in [
        (
            project.as_str(),
            "kiln/dispatch",
            DELEGATE,
            exact.resource(),
        ),
        (
            subtree.as_str(),
            "kiln/source/read",
            SOURCE_DELEGATE,
            entry.resource(),
        ),
    ] {
        let substituted = issue(
            WRONG_ISSUER_SEED_BYTE,
            DELEGATE_SEED_BYTE,
            &[(resource, ability), (resource, delegate)],
            None,
        );
        let token = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(leaf, ability)],
            Some(&substituted),
        );
        let fixture = chain(HOLDER_SEED_BYTE, token, vec![substituted]);
        let result = if ability == "kiln/dispatch" {
            admit(&verifier, &request, &fixture)
        } else {
            admit_source(&verifier, &read, &fixture)
        };
        assert_eq!(result, (Err(Denied), 0));

        // A delegator holding the delegate ability cannot act as its own root.
        let self_rooted = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(leaf, ability)],
            None,
        );
        let fixture = chain(HOLDER_SEED_BYTE, self_rooted, Vec::new());
        let result = if ability == "kiln/dispatch" {
            admit(&verifier, &request, &fixture)
        } else {
            admit_source(&verifier, &read, &fixture)
        };
        assert_eq!(result, (Err(Denied), 0));
    }
}

#[test]
// r[verify basalt.kiln_authority.attenuable]
fn kiln_v2_scope_builder_matches_request_framing_and_rejects_gaps() {
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let exact = verifier(&policy, &reference, &roots, 1)
        .enforcement(&request)
        .unwrap();
    assert!(exact.resource().starts_with("kiln://implementation/v2/"));
    assert!(exact.resource().starts_with(&work_prefix()));
    let gap = ImplementationScope {
        audience: AUDIENCE,
        generation: GENERATION,
        project: None,
        work: Some(WORK),
        work_version: None,
        attempt: None,
        fence: None,
    };
    assert_eq!(gap.prefix(), Err(Denied));
    let zero = ImplementationScope {
        generation: 0,
        project: Some(PROJECT),
        ..gap
    };
    assert_eq!(zero.prefix(), Err(Denied));
}

#[test]
// r[verify basalt.kiln_authority.attenuable.scenario.v1_stable]
fn kiln_v1_chain_ignores_the_attenuable_depth_and_delegate_rule() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 0);
    let mut request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    request.framing = ImplementationFraming::V1;
    let exact = verifier.enforcement(&request).unwrap();
    assert!(exact.resource().starts_with("kiln://implementation/v1/"));
    let parent = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[("kiln://implementation/v1/", "kiln/dispatch")],
        None,
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&parent),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![parent]);
    assert_eq!(admit(&verifier, &request, &fixture), (Ok(()), 1));

    // A v1 grant never covers the same request framed as v2.
    request.framing = ImplementationFraming::V2;
    assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
}

#[test]
// r[verify basalt.kiln_authority.source_actions.scenario.distinct]
fn kiln_source_operations_are_distinct_signed_and_exactly_checked() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 0);
    let mut contracts = std::collections::BTreeSet::new();
    let mut abilities = std::collections::BTreeSet::new();
    for operation in ALL_SOURCE_OPERATIONS {
        let action = operation.action();
        assert!(contracts.insert(action.contract_id()));
        assert!(abilities.insert(action.ability()));
        assert!(action.ability().starts_with("kiln/source/"));
        assert_eq!(action.resource_prefix(), "kiln://source/v1/");

        let request = source(
            operation,
            HOLDER_SEED_BYTE,
            &["src", "lib.rs"],
            PathScope::Entry,
        );
        let exact = verifier.source_enforcement(&request).unwrap();
        let fixture = InvocationFixture::direct(
            CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap(),
        );
        let binding = bind_enforcement_action(action, &exact).unwrap();
        let signature = sign_operation(&fixture, &exact, nonce(), binding);
        let replay = RecordingReplay::new();
        let bounds = boundaries(&fixture, &replay, &NoRevocations);
        let session = verifier
            .verify_source(request.clone(), &invocation(&fixture, &signature), &bounds)
            .unwrap();
        verifier.revalidate_source(&session, &bounds).unwrap();
        assert_eq!(replay.calls.get(), 1);
        session
            .require(
                operation,
                &request.path,
                PathScope::Entry,
                request.request_digest,
            )
            .unwrap();
        let other = if operation == SourceOperation::Delegate {
            SourceOperation::ReadContent
        } else {
            SourceOperation::Delegate
        };
        assert_eq!(
            session.require(
                other,
                &request.path,
                PathScope::Entry,
                request.request_digest
            ),
            Err(Denied)
        );
        assert_eq!(
            session.require(
                operation,
                &path(&["src", "lib.rsx"]),
                PathScope::Entry,
                request.request_digest
            ),
            Err(Denied)
        );
        assert_eq!(
            session.require(
                operation,
                &request.path,
                PathScope::Subtree,
                request.request_digest
            ),
            Err(Denied)
        );

        // One operation's signature cannot stand in for another.
        let mut substituted = request.clone();
        substituted.operation = other;
        let replay = RecordingReplay::new();
        assert!(
            verifier
                .verify_source(
                    substituted,
                    &invocation(&fixture, &signature),
                    &boundaries(&fixture, &replay, &NoRevocations)
                )
                .is_err()
        );
        assert_eq!(replay.calls.get(), 0);

        let subtree = source(operation, HOLDER_SEED_BYTE, &["src"], PathScope::Subtree);
        assert_eq!(
            verifier.source_enforcement(&subtree).is_err(),
            operation.requires_entry()
        );
    }
}

#[test]
// r[verify basalt.kiln_authority.source_path.scenario.alias]
fn kiln_source_path_markers_never_alias_siblings_or_descendants() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let read = |components: &[&str], scope| {
        verifier
            .source_enforcement(&source(
                SourceOperation::ReadHistory,
                HOLDER_SEED_BYTE,
                components,
                scope,
            ))
            .unwrap()
            .resource()
            .to_owned()
    };
    let subtree_ab = source_prefix(&["a", "b"], None);
    let entry_ab = source_prefix(&["a", "b"], Some(PathScope::Entry));
    let node_ab = source_prefix(&["a", "b"], Some(PathScope::Subtree));

    // A subtree grant covers the entry, the node and descendants only.
    assert!(read(&["a", "b"], PathScope::Entry).starts_with(&subtree_ab));
    assert!(read(&["a", "b"], PathScope::Subtree).starts_with(&subtree_ab));
    assert!(read(&["a", "b", "c"], PathScope::Entry).starts_with(&subtree_ab));
    assert!(!read(&["a", "bc"], PathScope::Entry).starts_with(&subtree_ab));
    assert!(!read(&["a", "bc"], PathScope::Subtree).starts_with(&subtree_ab));
    assert!(!read(&["ab"], PathScope::Entry).starts_with(&subtree_ab));
    // An entry grant covers exactly one entry.
    assert!(read(&["a", "b"], PathScope::Entry).starts_with(&entry_ab));
    assert!(!read(&["a", "b"], PathScope::Subtree).starts_with(&entry_ab));
    assert!(!read(&["a", "b", "c"], PathScope::Entry).starts_with(&entry_ab));
    assert!(!read(&["a", "bc"], PathScope::Entry).starts_with(&entry_ab));
    // A subtree-node grant covers only subtree-scoped requests at that node.
    assert!(read(&["a", "b"], PathScope::Subtree).starts_with(&node_ab));
    assert!(!read(&["a", "b"], PathScope::Entry).starts_with(&node_ab));
    assert!(!read(&["a", "b", "c"], PathScope::Entry).starts_with(&node_ab));

    // Signed: a subtree grant for a/b never authorizes a/bc.
    let grant = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&subtree_ab, "kiln/source/read")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, grant, Vec::new());
    let inside = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "b", "c"],
        PathScope::Entry,
    );
    assert_eq!(admit_source(&verifier, &inside, &fixture), (Ok(()), 1));
    let sibling = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "bc"],
        PathScope::Entry,
    );
    assert_eq!(
        admit_source(&verifier, &sibling, &fixture),
        (Err(Denied), 0)
    );

    // An operator-context grant never covers an attempt-context request.
    let mut attempt = inside;
    attempt.attempt = Some(SourceAttempt {
        attempt: "attempt-a".into(),
        fence: 5,
    });
    assert_eq!(
        admit_source(&verifier, &attempt, &fixture),
        (Err(Denied), 0)
    );
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.scoped]
// r[verify basalt.kiln_authority.delegation.scenario.widen]
// r[verify basalt.kiln_authority.delegation.chain.scenario.missing_delegate]
// r[verify basalt.kiln_authority.delegation.chain.scenario.depth]
fn kiln_source_delegation_is_scoped_explicit_and_bounded() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let subtree_ab = source_prefix(&["a", "b"], None);
    let subtree_a = source_prefix(&["a"], None);
    let entry_abc = source_prefix(&["a", "b", "c"], Some(PathScope::Entry));
    let request = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "b", "c"],
        PathScope::Entry,
    );
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[
            (&subtree_ab, "kiln/source/read"),
            (&subtree_ab, SOURCE_DELEGATE),
        ],
        None,
    );

    // Scoped delegation of one entry is admitted.
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&entry_abc, "kiln/source/read")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    assert_eq!(admit_source(&verifier, &request, &fixture), (Ok(()), 1));

    // Widening to the parent directory or to another operation is refused.
    for (resource, ability) in [
        (subtree_a.as_str(), "kiln/source/read"),
        (entry_abc.as_str(), "kiln/source/modify"),
    ] {
        let leaf = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(resource, ability)],
            Some(&ceiling),
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
        let mut widened = request.clone();
        if ability == "kiln/source/modify" {
            widened.operation = SourceOperation::ModifySource;
        }
        assert_eq!(
            admit_source(&verifier, &widened, &fixture),
            (Err(Denied), 0)
        );
    }

    // Without the explicit source delegate ability the chain is refused.
    for held in [
        vec![(subtree_ab.as_str(), "kiln/source/read")],
        vec![
            (subtree_ab.as_str(), "kiln/source/read"),
            (subtree_ab.as_str(), DELEGATE),
        ],
    ] {
        let ceiling = issue(ISSUER_SEED_BYTE, DELEGATE_SEED_BYTE, &held, None);
        let leaf = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(&entry_abc, "kiln/source/read")],
            Some(&ceiling),
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
        assert_eq!(
            admit_source(&verifier, &request, &fixture),
            (Err(Denied), 0)
        );
    }

    // A second hop exceeds the configured depth.
    let middle = issue(
        DELEGATE_SEED_BYTE,
        SECOND_DELEGATE_SEED_BYTE,
        &[
            (&subtree_ab, "kiln/source/read"),
            (&subtree_ab, SOURCE_DELEGATE),
        ],
        Some(&ceiling),
    );
    let leaf = issue(
        SECOND_DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&entry_abc, "kiln/source/read")],
        Some(&middle),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling, middle]);
    assert_eq!(
        admit_source(&verifier, &request, &fixture),
        (Err(Denied), 0)
    );
    let deeper = Verifier {
        max_delegation_depth: 2,
        ..verifier
    };
    assert_eq!(admit_source(&deeper, &request, &fixture), (Ok(()), 1));
}

#[test]
// r[verify basalt.kiln_authority.source_actions.scenario.unknown]
fn kiln_source_unknown_operations_and_malformed_paths_deny() {
    let request = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a"],
        PathScope::Entry,
    );
    let mut json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["operation"], "read");
    for unknown in ["rename", "delete", "admin", "read-content", "*"] {
        json["operation"] = unknown.into();
        assert!(serde_json::from_value::<SourceRequest>(json.clone()).is_err());
    }
    assert!(serde_json::from_str::<EnforcementAction>("\"kiln-source-rename\"").is_err());
    let policy = default_policy().unwrap();
    assert!(policy.contract("kiln-source-rename").is_none());

    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let deep: Vec<&str> = std::iter::repeat_n("d", 65).collect();
    for components in [
        vec![""],
        vec!["."],
        vec![".."],
        vec!["a/b"],
        vec!["a\u{0}"],
        vec!["line\nbreak"],
        Vec::new(),
        deep,
    ] {
        let malformed = source(
            SourceOperation::ReadContent,
            HOLDER_SEED_BYTE,
            &components,
            PathScope::Entry,
        );
        assert_eq!(
            verifier.source_enforcement(&malformed).map(|_| ()),
            Err(Denied)
        );
    }
    let root = source(
        SourceOperation::Discover,
        HOLDER_SEED_BYTE,
        &[],
        PathScope::Subtree,
    );
    assert!(verifier.source_enforcement(&root).is_ok());
    let unicode = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["docs", "résumé.md"],
        PathScope::Entry,
    );
    assert!(verifier.source_enforcement(&unicode).is_ok());

    // A policy without the source contract denies before replay.
    let minimal = basalt::parse_policy_json(
        r#"{
          "schema_version": "ucan-nickel-contracts.policy.v1",
          "contracts": {
            "kiln-dispatch": {
              "id": "kiln-dispatch",
              "resource_prefixes": ["kiln://implementation/v2/"],
              "abilities": ["kiln/dispatch"]
            }
          }
        }"#,
    )
    .unwrap();
    let missing = Verifier {
        policy: &minimal,
        ..verifier
    };
    assert_eq!(
        missing.source_enforcement(&request).map(|_| ()),
        Err(Denied)
    );
}

/// Verify one implementation request and return its live session.
fn session(verifier: &Verifier<'_>, request: &Request, fixture: &InvocationFixture) -> Session {
    let exact = verifier.enforcement(request).unwrap();
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = sign_operation(fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    verifier
        .verify(
            request.clone(),
            &invocation(fixture, &signature),
            &boundaries(fixture, &replay, &NoRevocations),
        )
        .unwrap()
}

/// Verify one source request and return its live session.
fn source_session(
    verifier: &Verifier<'_>,
    request: &SourceRequest,
    fixture: &InvocationFixture,
) -> SourceSession {
    let exact = verifier.source_enforcement(request).unwrap();
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = sign_operation(fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    verifier
        .verify_source(
            request.clone(),
            &invocation(fixture, &signature),
            &boundaries(fixture, &replay, &NoRevocations),
        )
        .unwrap()
}

fn resources(capabilities: &CapabilitySet) -> Vec<String> {
    capabilities
        .as_slice()
        .iter()
        .map(|capability| capability.resource.clone())
        .collect()
}

#[test]
// r[verify basalt.kiln_authority.effective_scope.scenario.unchecked]
fn kiln_effective_capabilities_omit_grants_the_chain_rule_rejects() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);

    // Source: delegate held only over `a`, yet the leaf also names `secret`.
    let repository = source_prefix(&[], None);
    let subtree_a = source_prefix(&["a"], None);
    let subtree_secret = source_prefix(&["secret"], None);
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[
            (&repository, "kiln/source/discover"),
            (&subtree_a, SOURCE_DELEGATE),
        ],
        None,
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[
            (&subtree_a, "kiln/source/discover"),
            (&subtree_secret, "kiln/source/discover"),
        ],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    let request = source(
        SourceOperation::Discover,
        HOLDER_SEED_BYTE,
        &["a"],
        PathScope::Subtree,
    );
    let admitted = source_session(&verifier, &request, &fixture);
    assert_eq!(
        resources(admitted.effective_capabilities()),
        vec![subtree_a]
    );
    let secret = source(
        SourceOperation::Discover,
        HOLDER_SEED_BYTE,
        &["secret"],
        PathScope::Subtree,
    );
    assert_eq!(admit_source(&verifier, &secret, &fixture), (Err(Denied), 0));

    // Implementation v2: delegate held only over attempt-a, leaf also names attempt-b.
    let project = project_prefix(PROJECT);
    let attempt_a = attempt_prefix("attempt-a");
    let attempt_b = attempt_prefix("attempt-b");
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&attempt_a, DELEGATE)],
        None,
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&attempt_a, "kiln/dispatch"), (&attempt_b, "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let admitted = session(&verifier, &request, &fixture);
    assert_eq!(
        resources(admitted.effective_capabilities()),
        vec![attempt_a]
    );
    let other = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-b");
    assert_eq!(admit(&verifier, &other, &fixture), (Err(Denied), 0));
}

#[test]
// r[verify basalt.kiln_authority.attenuable.scenario.v1_wide]
// r[verify basalt.kiln_authority.attenuable.scenario.v1_refused]
fn kiln_v1_wide_grants_need_explicit_delegate_and_v1_can_be_refused() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 0);
    let mut request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    request.framing = ImplementationFraming::V1;
    for (held, leaf_resource, expected) in [
        (
            vec![("kiln://", "kiln/dispatch")],
            "kiln://implementation/v1/",
            (Err(Denied), 0),
        ),
        (
            vec![("kiln://", "kiln/dispatch"), ("kiln://", DELEGATE)],
            "kiln://implementation/v1/",
            (Ok(()), 1),
        ),
        // A wide v1 leaf is admitted when every hop holds `kiln/delegate`.
        (
            vec![("kiln://", "kiln/dispatch"), ("kiln://", DELEGATE)],
            "kiln://",
            (Ok(()), 1),
        ),
        (
            vec![("kiln://", "kiln/dispatch")],
            "kiln://",
            (Err(Denied), 0),
        ),
    ] {
        let wide = issue(ISSUER_SEED_BYTE, DELEGATE_SEED_BYTE, &held, None);
        let leaf = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(leaf_resource, "kiln/dispatch")],
            Some(&wide),
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![wide]);
        assert_eq!(admit(&verifier, &request, &fixture), expected);
    }

    // A wide v1 leaf issued directly by a trusted root is admitted, and its covering
    // grant stays in the effective set.
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[("kiln://", "kiln/dispatch")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    let legacy = session(&verifier, &request, &fixture);
    assert_eq!(
        resources(legacy.effective_capabilities()),
        vec!["kiln://".to_owned()]
    );

    // Closing the compatibility window refuses new v1 requests and revalidation of a
    // v1 session minted while v1 was accepted.
    let closed = Verifier {
        implementation_v1: LegacyFraming::Refuse,
        ..verifier
    };
    assert_eq!(closed.enforcement(&request).map(|_| ()), Err(Denied));
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations);
    verifier.revalidate(&legacy, &bounds).unwrap();
    assert_eq!(closed.revalidate(&legacy, &bounds), Err(Denied));

    // A v2 session never exposes a v1 grant, whatever the window.
    let current = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = closed.enforcement(&current).unwrap();
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[
            (exact.resource(), "kiln/dispatch"),
            ("kiln://implementation/v1/", "kiln/dispatch"),
        ],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    for session_verifier in [&verifier, &closed] {
        let admitted = session(session_verifier, &current, &fixture);
        assert_eq!(
            resources(admitted.effective_capabilities()),
            vec![exact.resource().to_owned()]
        );
    }
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.generation]
fn kiln_attenuable_grants_must_pin_the_generation() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = Verifier {
        generation: 8,
        ..verifier(&policy, &reference, &roots, 1)
    };
    let mut request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    request.generation = 8;
    let audience_wide = format!(
        "kiln://implementation/v2/{}/{}/",
        AUDIENCE.len(),
        hex(AUDIENCE)
    );
    assert!(
        verifier
            .enforcement(&request)
            .unwrap()
            .resource()
            .starts_with(&audience_wide)
    );
    for wide in [
        audience_wide.as_str(),
        "kiln://implementation/v2/",
        "kiln://",
    ] {
        let direct = issue(
            ISSUER_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(wide, "kiln/dispatch")],
            None,
        );
        let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
        assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
    }
    let pinned = ImplementationScope {
        audience: AUDIENCE,
        generation: 8,
        project: Some(PROJECT),
        work: None,
        work_version: None,
        attempt: None,
        fence: None,
    }
    .prefix()
    .unwrap();
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&pinned, "kiln/dispatch")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    assert_eq!(admit(&verifier, &request, &fixture), (Ok(()), 1));

    let mut read = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a"],
        PathScope::Entry,
    );
    read.generation = 8;
    let source_audience = format!("kiln://source/v1/{}/{}/", AUDIENCE.len(), hex(AUDIENCE));
    for wide in [source_audience.as_str(), "kiln://source/v1/", "kiln://"] {
        let direct = issue(
            ISSUER_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(wide, "kiln/source/read")],
            None,
        );
        let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
        assert_eq!(admit_source(&verifier, &read, &fixture), (Err(Denied), 0));
    }
    let pinned = SourceScope {
        audience: AUDIENCE,
        generation: 8,
        repository: REPOSITORY,
        attempt: None,
        path: &[],
        terminal: None,
    }
    .prefix()
    .unwrap();
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&pinned, "kiln/source/read")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    assert_eq!(admit_source(&verifier, &read, &fixture), (Ok(()), 1));
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.missing_delegate]
fn kiln_wide_root_grant_cannot_bypass_the_delegate_rule() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&request).unwrap();
    for held in [
        vec![("kiln://", "kiln/dispatch")],
        vec![("kiln://", "kiln/dispatch"), ("kiln://", DELEGATE)],
    ] {
        let wide = issue(ISSUER_SEED_BYTE, DELEGATE_SEED_BYTE, &held, None);
        let leaf = issue(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            &[(exact.resource(), "kiln/dispatch")],
            Some(&wide),
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![wide]);
        assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
    }
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.caveated]
fn kiln_caveated_delegate_is_not_explicit() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let project = project_prefix(PROJECT);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier.enforcement(&request).unwrap();
    let caveat = CaveatDocument::new(
        CAVEAT_DOMAIN.to_owned(),
        CAVEAT_TYPE.to_owned(),
        CAVEAT_KEY.to_owned(),
        CAVEAT_PAYLOAD.to_vec(),
    )
    .unwrap();
    let ceiling = issue_documents(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        vec![
            CapabilityDocument::new(project.clone(), "kiln/dispatch".into()).unwrap(),
            CapabilityDocument::with_caveats(project, DELEGATE.into(), vec![caveat]).unwrap(),
        ],
        &[],
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    assert_eq!(admit(&verifier, &request, &fixture), (Err(Denied), 0));
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.depth]
fn kiln_revalidation_applies_the_current_depth_limit() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let deep = verifier(&policy, &reference, &roots, 2);
    let shallow = verifier(&policy, &reference, &roots, 1);
    let project = project_prefix(PROJECT);
    let work = work_prefix();
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = deep.enforcement(&request).unwrap();
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );
    let middle = issue(
        DELEGATE_SEED_BYTE,
        SECOND_DELEGATE_SEED_BYTE,
        &[(&work, "kiln/dispatch"), (&work, DELEGATE)],
        Some(&ceiling),
    );
    let leaf = issue(
        SECOND_DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&middle),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling, middle]);
    let admitted = session(&deep, &request, &fixture);
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations);
    deep.revalidate(&admitted, &bounds).unwrap();
    assert_eq!(shallow.revalidate(&admitted, &bounds), Err(Denied));
    assert_eq!(replay.calls.get(), 0);
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.multi_proof]
fn kiln_multi_proof_tokens_check_every_path_and_any_parent_delegate() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let project = project_prefix(PROJECT);
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    let exact = verifier(&policy, &reference, &roots, 2)
        .enforcement(&request)
        .unwrap();
    // The delegator's dispatch comes from one parent and its delegate from another,
    // which is one hop longer.
    let direct = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch")],
        None,
    );
    let upper = issue(
        ISSUER_SEED_BYTE,
        SECOND_DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );
    let lower = issue(
        SECOND_DELEGATE_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, DELEGATE)],
        Some(&upper),
    );
    let leaf = issue_documents(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        vec![CapabilityDocument::new(exact.resource().into(), "kiln/dispatch".into()).unwrap()],
        &[&direct, &lower],
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&direct), upper, lower]);
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 2),
            &request,
            &fixture
        ),
        (Ok(()), 1)
    );
    // The longer path through the delegate-carrying parent exceeds depth 1.
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 1),
            &request,
            &fixture
        ),
        (Err(Denied), 0)
    );
    // Without the delegate-carrying parent, the same grant denies.
    let alone = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/dispatch")],
        Some(&direct),
    );
    let fixture = chain(HOLDER_SEED_BYTE, alone, vec![direct]);
    assert_eq!(
        admit(
            &verifier(&policy, &reference, &roots, 2),
            &request,
            &fixture
        ),
        (Err(Denied), 0)
    );
}

#[test]
// r[verify basalt.kiln_authority.delegation.chain.scenario.aligned]
fn kiln_source_grants_must_stop_on_component_boundaries() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let request = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["docs", "x"],
        PathScope::Entry,
    );
    let exact = verifier.source_enforcement(&request).unwrap();
    let docs = source_prefix(&["docs"], None);
    // Cut inside the `docs` component: it would also cover `doc` siblings by bytes.
    let partial = &docs[..docs.len() - 3];
    assert!(exact.resource().starts_with(partial));
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(partial, "kiln/source/read"), (partial, SOURCE_DELEGATE)],
        None,
    );
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(exact.resource(), "kiln/source/read")],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    assert_eq!(
        admit_source(&verifier, &request, &fixture),
        (Err(Denied), 0)
    );
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(partial, "kiln/source/read")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    assert_eq!(
        admit_source(&verifier, &request, &fixture),
        (Err(Denied), 0)
    );
}

#[test]
// r[verify basalt.kiln_authority.source_path.scenario.attempt_context]
fn kiln_source_attempt_context_grant_admits_only_its_attempt_and_fence() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let attempt = SourceAttempt {
        attempt: "attempt-a".into(),
        fence: 5,
    };
    let grant = SourceScope {
        audience: AUDIENCE,
        generation: GENERATION,
        repository: REPOSITORY,
        attempt: Some(&attempt),
        path: &path(&["a"]),
        terminal: None,
    }
    .prefix()
    .unwrap();
    let direct = issue(
        ISSUER_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[(&grant, "kiln/source/read")],
        None,
    );
    let fixture = chain(HOLDER_SEED_BYTE, direct, Vec::new());
    let mut request = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "b"],
        PathScope::Entry,
    );
    request.attempt = Some(attempt.clone());
    assert_eq!(admit_source(&verifier, &request, &fixture), (Ok(()), 1));

    let mut operator = request.clone();
    operator.attempt = None;
    assert_eq!(
        admit_source(&verifier, &operator, &fixture),
        (Err(Denied), 0)
    );

    let mut refenced = request;
    refenced.attempt = Some(SourceAttempt {
        fence: 6,
        ..attempt
    });
    assert_eq!(
        admit_source(&verifier, &refenced, &fixture),
        (Err(Denied), 0)
    );
}

#[test]
// r[verify basalt.kiln_authority.effective_scope.scenario.deployment]
fn kiln_effective_capabilities_stay_in_the_current_deployment_and_scope() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = Verifier {
        generation: 8,
        ..verifier(&policy, &reference, &roots, 1)
    };
    let attempt = SourceAttempt {
        attempt: "attempt-a".into(),
        fence: 5,
    };
    let scope_at = |audience: &str,
                    generation: u64,
                    repository: &str,
                    attempt: Option<&SourceAttempt>,
                    components: &[&str]| {
        SourceScope {
            audience,
            generation,
            repository,
            attempt,
            path: &path(components),
            terminal: None,
        }
        .prefix()
        .unwrap()
    };
    let current_a = scope_at(AUDIENCE, 8, REPOSITORY, None, &["a"]);
    let sibling = scope_at(AUDIENCE, 8, REPOSITORY, None, &["b"]);
    // After a 7→8 bump: a repository-wide grant still pinned to generation 7.
    let stale = scope_at(AUDIENCE, 7, REPOSITORY, None, &[]);
    let foreign = scope_at("kiln-service-b", 8, REPOSITORY, None, &[]);
    let other_repository = scope_at(AUDIENCE, 8, "Other/repo", None, &[]);
    let other_context = scope_at(AUDIENCE, 8, REPOSITORY, Some(&attempt), &[]);
    let all = [
        &current_a,
        &sibling,
        &stale,
        &foreign,
        &other_repository,
        &other_context,
    ];
    let held: Vec<(&str, &str)> = all
        .iter()
        .flat_map(|resource| {
            [
                (resource.as_str(), "kiln/source/read"),
                (resource.as_str(), SOURCE_DELEGATE),
            ]
        })
        .collect();
    let ceiling = issue(ISSUER_SEED_BYTE, DELEGATE_SEED_BYTE, &held, None);
    let granted: Vec<(&str, &str)> = all
        .iter()
        .map(|resource| (resource.as_str(), "kiln/source/read"))
        .collect();
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &granted,
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    let mut request = source(
        SourceOperation::ReadContent,
        HOLDER_SEED_BYTE,
        &["a", "x"],
        PathScope::Entry,
    );
    request.generation = 8;
    let admitted = source_session(&verifier, &request, &fixture);
    assert_eq!(
        resources(admitted.effective_capabilities()),
        vec![current_a.clone(), sibling.clone()]
    );
}

#[test]
// r[verify basalt.kiln_authority.effective_scope.scenario.deployment]
fn kiln_implementation_effective_capabilities_stay_in_the_current_deployment() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = Verifier {
        generation: 8,
        ..verifier(&policy, &reference, &roots, 1)
    };
    let project_at = |audience: &str, generation: u64| {
        ImplementationScope {
            audience,
            generation,
            project: Some(PROJECT),
            work: None,
            work_version: None,
            attempt: None,
            fence: None,
        }
        .prefix()
        .unwrap()
    };
    let attempt_at = |name: &str| {
        ImplementationScope {
            audience: AUDIENCE,
            generation: 8,
            project: Some(PROJECT),
            work: Some(WORK),
            work_version: Some(3),
            attempt: Some(name),
            fence: Some(5),
        }
        .prefix()
        .unwrap()
    };
    let current = project_at(AUDIENCE, 8);
    let stale = project_at(AUDIENCE, 7);
    let foreign = project_at("kiln-service-b", 8);
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[
            (&current, "kiln/dispatch"),
            (&current, DELEGATE),
            (&stale, "kiln/dispatch"),
            (&stale, DELEGATE),
            (&foreign, "kiln/dispatch"),
            (&foreign, DELEGATE),
        ],
        None,
    );
    let attempt_a = attempt_at("attempt-a");
    let attempt_b = attempt_at("attempt-b");
    let leaf = issue(
        DELEGATE_SEED_BYTE,
        HOLDER_SEED_BYTE,
        &[
            (&attempt_a, "kiln/dispatch"),
            (&attempt_b, "kiln/dispatch"),
            (&stale, "kiln/dispatch"),
            (&foreign, "kiln/dispatch"),
        ],
        Some(&ceiling),
    );
    let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![ceiling]);
    let mut request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-a");
    request.generation = 8;
    let admitted = session(&verifier, &request, &fixture);
    assert_eq!(
        resources(admitted.effective_capabilities()),
        vec![attempt_a, attempt_b]
    );
}

/// Counts key resolutions, which every token and proof verification performs.
struct CountingKeys<'a> {
    inner: &'a KeyResolutionContext,
    calls: std::cell::Cell<u32>,
}

impl KeyResolver for CountingKeys<'_> {
    fn resolve_key(&self, issuer: &IssuerDid) -> Result<Ed25519PublicKey, KeyResolverError> {
        self.calls.set(self.calls.get() + 1);
        self.inner.resolve_key(issuer)
    }
}

#[test]
// r[verify basalt.kiln_authority.effective_scope.scenario.memoized]
fn kiln_admission_verifies_proofs_once_regardless_of_grant_count() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [did(ISSUER_SEED_BYTE)];
    let verifier = verifier(&policy, &reference, &roots, 1);
    let project = project_prefix(PROJECT);
    let ceiling = issue(
        ISSUER_SEED_BYTE,
        DELEGATE_SEED_BYTE,
        &[(&project, "kiln/dispatch"), (&project, DELEGATE)],
        None,
    );
    let request = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-0");
    let exact = verifier.enforcement(&request).unwrap();
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    // Returns (key resolutions, replay calls, effective grant count or None if denied).
    let run = |grants: usize, forged: bool| {
        let capabilities = (0..grants)
            .map(|index| {
                CapabilityDocument::new(
                    attempt_prefix(&format!("attempt-{index}")),
                    "kiln/dispatch".into(),
                )
                .unwrap()
            })
            .collect();
        let leaf = issue_documents(
            DELEGATE_SEED_BYTE,
            HOLDER_SEED_BYTE,
            capabilities,
            &[&ceiling],
        );
        let fixture = chain(HOLDER_SEED_BYTE, leaf, vec![copy(&ceiling)]);
        let signature = if forged {
            let other = implementation(Operation::Dispatch, HOLDER_SEED_BYTE, PROJECT, "attempt-1");
            let other_exact = verifier.enforcement(&other).unwrap();
            let other_binding =
                bind_enforcement_action(other.operation.action(), &other_exact).unwrap();
            sign_operation(&fixture, &other_exact, nonce(), other_binding)
        } else {
            sign_operation(&fixture, &exact, nonce(), binding)
        };
        let counting = CountingKeys {
            inner: &fixture.keys,
            calls: std::cell::Cell::new(0),
        };
        let replay = RecordingReplay::new();
        let mut bounds = boundaries(&fixture, &replay, &NoRevocations);
        bounds.key_resolver = &counting;
        let admitted = verifier
            .verify(request.clone(), &invocation(&fixture, &signature), &bounds)
            .ok()
            .map(|session| session.effective_capabilities().as_slice().len());
        (counting.calls.get(), replay.calls.get(), admitted)
    };
    let (one, one_replay, one_admitted) = run(1, false);
    let (many, many_replay, many_admitted) = run(64, false);
    assert_eq!((one_replay, one_admitted), (1, Some(1)));
    assert_eq!((many_replay, many_admitted), (1, Some(64)));
    assert_eq!(
        many, one,
        "key resolutions must not grow with the grant count"
    );
    let (forged, forged_replay, forged_admitted) = run(64, true);
    assert_eq!((forged_replay, forged_admitted), (0, None));
    assert!(
        forged <= one,
        "a forged signature stops before any chain walk"
    );
}
