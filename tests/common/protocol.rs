use lattice::engine::protocol::Cohort;
use lattice::engine::protocol::FrozenProjection;
use lattice::engine::protocol::Identity;
use lattice::engine::protocol::PlacementKind;
use lattice::engine::protocol::ReviewedLimits;
use lattice::engine::protocol::RoleId;
use lattice::engine::protocol::RolePlacement;
use lattice::engine::protocol::admit_projection;
use lattice::engine::protocol::reviewed_limits;

pub const ROLE_A: choregraph_choreography::RoleId = choregraph_choreography::RoleId(1);
pub const ROLE_B: choregraph_choreography::RoleId = choregraph_choreography::RoleId(2);
pub const ROLE_C: choregraph_choreography::RoleId = choregraph_choreography::RoleId(3);
pub const VALUE_A: choregraph_choreography::ValueId = choregraph_choreography::ValueId(10);
pub const PAYLOAD_A: choregraph_choreography::PayloadTag = choregraph_choreography::PayloadTag(20);
pub const PAYLOAD_B: choregraph_choreography::PayloadTag = choregraph_choreography::PayloadTag(21);
pub const LABEL_A: choregraph_choreography::LabelId = choregraph_choreography::LabelId(30);
pub const LABEL_B: choregraph_choreography::LabelId = choregraph_choreography::LabelId(31);
pub const MAX_ENVELOPE_BYTES: u64 = 4_096;
pub const ADAPTER_TIMEOUT_MS: u64 = 1_000;
const SPAN_START: u32 = 0;
const SPAN_END: u32 = 8;

pub fn identity(seed: &str) -> Identity {
    match Identity::new(format!("blake3:{}", blake3::hash(seed.as_bytes()).to_hex())) {
        Ok(identity) => identity,
        Err(error) => panic!("fixture identity failed: {error:?}"),
    }
}

pub fn end() -> choregraph_choreography::ChoreographyNode {
    choregraph_choreography::ChoreographyNode::End { span: span() }
}

pub fn communication(
    from: choregraph_choreography::RoleId,
    to: choregraph_choreography::RoleId,
    label: choregraph_choreography::LabelId,
    payload_tag: choregraph_choreography::PayloadTag,
    next: choregraph_choreography::ChoreographyNode,
) -> choregraph_choreography::ChoreographyNode {
    choregraph_choreography::ChoreographyNode::Communication {
        from,
        to,
        label,
        payload_tag,
        next: Box::new(next),
        span: span(),
    }
}

pub fn transfer(
    from: choregraph_choreography::RoleId,
    to: choregraph_choreography::RoleId,
    label: choregraph_choreography::LabelId,
    next: choregraph_choreography::ChoreographyNode,
) -> choregraph_choreography::ChoreographyNode {
    choregraph_choreography::ChoreographyNode::Transfer {
        from,
        to,
        label,
        payload_tag: PAYLOAD_A,
        value: VALUE_A,
        next: Box::new(next),
        span: span(),
    }
}

pub fn choice() -> choregraph_choreography::ChoreographyNode {
    choregraph_choreography::ChoreographyNode::Choice {
        decider: ROLE_A,
        branches: vec![
            choregraph_choreography::ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end()),
                span: span(),
            },
            choregraph_choreography::ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_A, ROLE_B, LABEL_B, PAYLOAD_B, end()),
                span: span(),
            },
        ],
        span: span(),
    }
}

pub fn native_source(
    protocol: choregraph_choreography::ChoreographyNode,
) -> choregraph_choreography::RoleChoreographySource {
    source(protocol, false)
}

pub fn external_source(
    protocol: choregraph_choreography::ChoreographyNode,
) -> choregraph_choreography::RoleChoreographySource {
    source(protocol, true)
}

pub fn projection_parts(
    source: choregraph_choreography::RoleChoreographySource,
) -> (choregraph_choreography::ProjectedChoreography, choregraph_choreography::ProjectionBundle) {
    let admitted = match choregraph_choreography::admit_source(source) {
        Ok(admitted) => admitted,
        Err(diagnostics) => panic!("fixture admission failed: {diagnostics:?}"),
    };
    let projected = match choregraph_choreography::project_all(&admitted) {
        Ok(projected) => projected,
        Err(diagnostics) => panic!("fixture projection failed: {diagnostics:?}"),
    };
    let materials = choregraph_choreography::ProjectionMaterials::reviewed(
        b"lattice-role-protocol-source",
        b"lattice-role-protocol-evaluated",
    );
    let bundle = match choregraph_choreography::build_projection_bundle(&admitted, &materials) {
        Ok(bundle) => bundle,
        Err(diagnostics) => panic!("fixture bundle failed: {diagnostics:?}"),
    };
    (projected, bundle)
}

pub fn cohort(source: choregraph_choreography::RoleChoreographySource) -> Cohort {
    let (projected, bundle) = projection_parts(source);
    match admit_projection(FrozenProjection {
        projected: &projected,
        bundle: &bundle,
    }) {
        Ok(cohort) => cohort,
        Err(error) => panic!("fixture cohort failed: {error:?}"),
    }
}

pub fn native_placements(cohort: &Cohort) -> Vec<RolePlacement> {
    cohort
        .roles
        .iter()
        .map(|role| RolePlacement {
            role: role.role,
            adapter_cohort: identity(&format!("native-adapter-{}", role.role.0)),
            limits: reviewed_limits(ReviewedLimits {
                max_envelope_bytes: MAX_ENVELOPE_BYTES,
                timeout_ms: ADAPTER_TIMEOUT_MS,
            }),
            kind: PlacementKind::Native {
                runtime_route: identity(&format!("native-route-{}", role.role.0)),
            },
        })
        .collect()
}

pub fn external_placements(cohort: &Cohort) -> Vec<RolePlacement> {
    cohort
        .roles
        .iter()
        .map(|role| match &role.kind {
            lattice::engine::protocol::RoleKind::Native => RolePlacement {
                role: role.role,
                adapter_cohort: identity(&format!("native-adapter-{}", role.role.0)),
                limits: reviewed_limits(ReviewedLimits {
                    max_envelope_bytes: MAX_ENVELOPE_BYTES,
                    timeout_ms: ADAPTER_TIMEOUT_MS,
                }),
                kind: PlacementKind::Native {
                    runtime_route: identity(&format!("native-route-{}", role.role.0)),
                },
            },
            lattice::engine::protocol::RoleKind::External {
                contract,
                input_schema,
                output_schema,
            } => RolePlacement {
                role: role.role,
                adapter_cohort: identity("external-adapter"),
                limits: reviewed_limits(ReviewedLimits {
                    max_envelope_bytes: MAX_ENVELOPE_BYTES,
                    timeout_ms: ADAPTER_TIMEOUT_MS,
                }),
                kind: PlacementKind::External {
                    contract: contract.clone(),
                    input_schema: input_schema.clone(),
                    output_schema: output_schema.clone(),
                    destination_policy: identity("destination-policy"),
                    credential_reference: identity("credential-reference"),
                },
            },
        })
        .collect()
}

fn source(
    protocol: choregraph_choreography::ChoreographyNode,
    external_b: bool,
) -> choregraph_choreography::RoleChoreographySource {
    let role_b_kind = if external_b {
        choregraph_choreography::RoleKind::External {
            contract_identity: identity("external-contract").as_str().to_owned(),
            input_schema_identity: identity("external-input").as_str().to_owned(),
            output_schema_identity: identity("external-output").as_str().to_owned(),
        }
    } else {
        choregraph_choreography::RoleKind::Native
    };
    choregraph_choreography::RoleChoreographySource {
        schema: choregraph_choreography::ROLE_CHOREOGRAPHY_SCHEMA.to_owned(),
        limits: choregraph_choreography::ChoreographyLimits::reviewed(),
        imports: Vec::new(),
        roles: vec![
            role(ROLE_A, choregraph_choreography::RoleKind::Native),
            role(ROLE_B, role_b_kind),
        ],
        values: vec![choregraph_choreography::ValueDeclaration {
            id: VALUE_A,
            payload_tag: PAYLOAD_A,
            initial_owner: ROLE_A,
            span: span(),
        }],
        protocol,
        non_claims: choregraph_choreography::REQUIRED_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

fn role(
    id: choregraph_choreography::RoleId,
    kind: choregraph_choreography::RoleKind,
) -> choregraph_choreography::RoleDeclaration {
    choregraph_choreography::RoleDeclaration { id, kind, span: span() }
}

const fn span() -> choregraph_choreography::SourceSpan {
    choregraph_choreography::SourceSpan {
        start: SPAN_START,
        end: SPAN_END,
    }
}

pub const fn lattice_role(id: choregraph_choreography::RoleId) -> RoleId {
    RoleId(id.0)
}

pub fn prepared_transfer() -> lattice::engine::protocol::SessionRecord {
    let cohort = cohort(native_source(transfer(ROLE_A, ROLE_B, LABEL_A, end())));
    let plan =
        lattice::engine::protocol::plan_session(cohort.clone(), native_placements(&cohort), identity("session-nonce"))
            .unwrap_or_else(|error| panic!("session plan failed: {error:?}"));
    let session = lattice::engine::protocol::create_session(plan)
        .unwrap_or_else(|error| panic!("session creation failed: {error:?}"));
    let action = lattice::engine::protocol::plan_action(&session, lattice::engine::protocol::ActionRequest {
        source: lattice_role(ROLE_A),
        label: lattice::engine::protocol::LabelId(LABEL_A.0),
        payload_reference: identity("payload"),
    })
    .unwrap_or_else(|error| panic!("action plan failed: {error:?}"));
    lattice::engine::protocol::prepare_attempt(&session, action)
        .unwrap_or_else(|error| panic!("attempt prepare failed: {error:?}"))
}

pub fn unknown(prepared: &lattice::engine::protocol::SessionRecord) -> lattice::engine::protocol::SessionRecord {
    let attempt = prepared.attempts.last().unwrap_or_else(|| panic!("prepared fixture has no attempt"));
    let authorized = lattice::engine::protocol::record_authorization(prepared, &attempt.identity, vec![identity(
        "authority-attempt",
    )])
    .unwrap_or_else(|error| panic!("authorization record failed: {error:?}"));
    lattice::engine::protocol::mark_dispatched_unknown(&authorized, &attempt.identity)
        .unwrap_or_else(|error| panic!("dispatch state failed: {error:?}"))
}
