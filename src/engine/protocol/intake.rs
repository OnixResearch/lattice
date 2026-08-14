//! Frozen Choregraph projection intake and complete role placement admission.
//!
//! r[impl lattice.role_protocol.intake]
//! r[impl lattice.role_protocol.placement]
//! r[impl lattice.role_protocol.session_identity]
#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    reason = "fallible producer admission uses its local DTO vocabulary and typed rejection paths"
)]
#![allow(
    no_recursion,
    reason = "the pure conversion follows the producer-owned bounded recursive protocol tree"
)]

use super::model::AdapterLimits;
use super::model::Branch;
use super::model::Cohort;
use super::model::Error;
use super::model::Identity;
use super::model::Issue;
use super::model::OwnedValue;
use super::model::PlacementKind;
use super::model::RoleDefinition;
use super::model::RoleId;
use super::model::RoleInstance;
use super::model::RoleKind;
use super::model::RolePlacement;
use super::model::RoleProgram;
use super::model::SessionPlan;

const MINIMUM_LIMIT: u64 = 1;
const MINIMUM_TIMEOUT_MS: u64 = 1;

#[derive(Clone, Copy)]
pub struct FrozenProjection<'a> {
    pub projected: &'a choregraph_choreography::ProjectedChoreography,
    pub bundle: &'a choregraph_choreography::ProjectionBundle,
}

#[derive(serde::Serialize)]
struct PlanIdentityMaterial<'a> {
    cohort: &'a Cohort,
    roles: &'a [RoleInstance],
    nonce: &'a Identity,
}

pub fn admit_projection(input: FrozenProjection<'_>) -> Result<Cohort, Error> {
    if input.bundle.locals().len() != input.projected.locals.locals.len() {
        return Err(Issue::IncompleteArtifacts.into());
    }
    let receipt = input.bundle.receipt_body();
    if receipt.trellis_revision != super::model::FROZEN_TRELLIS_REVISION
        || receipt.local_artifacts.len() != input.projected.locals.locals.len()
    {
        return Err(Issue::UnsupportedCohort.into());
    }
    choregraph_choreography::verify_projection_receipt(input.bundle, receipt, &input.bundle.receipt().packed_bytes)
        .map_err(|_| Issue::IdentityMismatch)?;
    verify_projected_artifacts(input)?;

    let roles = input
        .projected
        .locals
        .locals
        .iter()
        .zip(input.bundle.locals())
        .map(|(local, artifact)| {
            if local.role != artifact.role {
                return Err(Error::from(Issue::IdentityMismatch));
            }
            Ok(RoleDefinition {
                role: RoleId(local.role.0),
                kind: role_kind(&local.role_kind)?,
                local_artifact: producer_identity(&artifact.artifact.identity)?,
                protocol: convert_local(&local.protocol)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let initial_owners = input
        .projected
        .global
        .values
        .iter()
        .map(|value| OwnedValue {
            value: super::model::ValueId(value.id.0),
            payload_tag: super::model::PayloadTag(value.payload_tag.0),
            owner: RoleId(value.initial_owner.0),
        })
        .collect();

    Ok(Cohort {
        source_revision: super::model::FROZEN_CHOREGRAPH_REVISION.to_owned(),
        adapter_identity: Identity::new(super::model::FROZEN_ADAPTER_IDENTITY)?,
        global_artifact: producer_identity(&input.bundle.global().identity)?,
        complete_local_set: producer_identity(&input.bundle.local_set().identity)?,
        projection_receipt: producer_identity(&input.bundle.receipt().identity)?,
        schema_set: producer_identity(&receipt.schema_set)?,
        proof_reference: producer_identity(&receipt.trellis_proof_set)?,
        roles,
        initial_owners,
    })
}

pub fn plan_session(cohort: Cohort, placements: Vec<RolePlacement>, nonce: Identity) -> Result<SessionPlan, Error> {
    if cohort.source_revision != super::model::FROZEN_CHOREGRAPH_REVISION
        || cohort.adapter_identity.as_str() != super::model::FROZEN_ADAPTER_IDENTITY
        || placements.len() != cohort.roles.len()
    {
        return Err(Issue::InvalidPlacement.into());
    }
    let mut roles = Vec::with_capacity(cohort.roles.len());
    for (definition, placement) in cohort.roles.iter().zip(placements) {
        validate_placement(definition, &placement)?;
        let role_label = definition.role.0.to_string();
        let placement_bytes = serde_json::to_vec(&placement).map_err(|_| Error::Serialization)?;
        let placement_digest = blake3::hash(&placement_bytes).to_hex().to_string();
        let identity = super::identity::reference(super::identity::ROLE_INSTANCE_DOMAIN, &[
            cohort.complete_local_set.as_str(),
            definition.local_artifact.as_str(),
            role_label.as_str(),
            placement_digest.as_str(),
        ])?;
        roles.push(RoleInstance {
            role: definition.role,
            identity,
            local_artifact: definition.local_artifact.clone(),
            placement,
        });
    }
    let plan_identity =
        super::identity::serialized_reference(super::identity::SESSION_PLAN_DOMAIN, &PlanIdentityMaterial {
            cohort: &cohort,
            roles: &roles,
            nonce: &nonce,
        })?;
    let session =
        super::identity::reference(super::identity::SESSION_DOMAIN, &[plan_identity.as_str(), nonce.as_str()])?;
    Ok(SessionPlan {
        identity: plan_identity,
        session,
        nonce,
        cohort,
        roles,
    })
}

fn verify_projected_artifacts(input: FrozenProjection<'_>) -> Result<(), Error> {
    let global = choregraph_kamacite::encode_role_choreography_global(&input.projected.global)
        .map_err(|_| Issue::IdentityMismatch)?;
    if global.packed_bytes != input.bundle.global().packed_bytes {
        return Err(Issue::IdentityMismatch.into());
    }
    let local_set = choregraph_kamacite::encode_role_choreography_local_set(&input.projected.locals)
        .map_err(|_| Issue::IdentityMismatch)?;
    if local_set.packed_bytes != input.bundle.local_set().packed_bytes {
        return Err(Issue::IdentityMismatch.into());
    }
    for (local, artifact) in input.projected.locals.locals.iter().zip(input.bundle.locals()) {
        let encoded =
            choregraph_kamacite::encode_role_choreography_local(local).map_err(|_| Issue::IdentityMismatch)?;
        if encoded.packed_bytes != artifact.artifact.packed_bytes {
            return Err(Issue::IdentityMismatch.into());
        }
    }
    Ok(())
}

fn validate_placement(definition: &RoleDefinition, placement: &RolePlacement) -> Result<(), Error> {
    let maximum_envelope_bytes =
        u64::try_from(choregraph_choreography::MAX_CANONICAL_OUTPUT_BYTES).map_err(|_| Issue::InvalidPlacement)?;
    let has_matching_role = definition.role == placement.role;
    let has_valid_byte_limit = placement.limits.max_envelope_bytes >= MINIMUM_LIMIT
        && placement.limits.max_envelope_bytes <= maximum_envelope_bytes;
    let has_valid_timeout = placement.limits.timeout_ms >= MINIMUM_TIMEOUT_MS;
    if !(has_matching_role && has_valid_byte_limit && has_valid_timeout) {
        return Err(Issue::InvalidPlacement.into());
    }
    match (&definition.kind, &placement.kind) {
        (RoleKind::Native, PlacementKind::Native { .. }) => Ok(()),
        (
            RoleKind::External {
                contract,
                input_schema,
                output_schema,
            },
            PlacementKind::External {
                contract: placed_contract,
                input_schema: placed_input,
                output_schema: placed_output,
                ..
            },
        ) if contract == placed_contract && input_schema == placed_input && output_schema == placed_output => Ok(()),
        _ => Err(Issue::InvalidPlacement.into()),
    }
}

fn producer_identity(identity: &choregraph_choreography::ChoreographyIdentity) -> Result<Identity, Error> {
    Identity::new(choregraph_choreography::prefixed_identity(identity))
}

fn role_kind(kind: &choregraph_choreography::RoleKind) -> Result<RoleKind, Error> {
    match kind {
        choregraph_choreography::RoleKind::Native => Ok(RoleKind::Native),
        choregraph_choreography::RoleKind::External {
            contract_identity,
            input_schema_identity,
            output_schema_identity,
        } => Ok(RoleKind::External {
            contract: Identity::new(contract_identity)?,
            input_schema: Identity::new(input_schema_identity)?,
            output_schema: Identity::new(output_schema_identity)?,
        }),
    }
}

fn local_branch(branch: &choregraph_choreography::LocalProtocolBranch) -> Result<Branch, Error> {
    Ok(Branch {
        label: super::model::LabelId(branch.label.0),
        body: convert_local(&branch.body)?,
    })
}

fn convert_local(protocol: &choregraph_choreography::LocalProtocol) -> Result<RoleProgram, Error> {
    Ok(match protocol {
        choregraph_choreography::LocalProtocol::End => RoleProgram::End,
        choregraph_choreography::LocalProtocol::Send {
            peer,
            label,
            payload_tag,
            next,
        } => RoleProgram::Send {
            peer: RoleId(peer.0),
            label: super::model::LabelId(label.0),
            payload_tag: super::model::PayloadTag(payload_tag.0),
            next: Box::new(convert_local(next)?),
        },
        choregraph_choreography::LocalProtocol::Receive {
            peer,
            label,
            payload_tag,
            next,
        } => RoleProgram::Receive {
            peer: RoleId(peer.0),
            label: super::model::LabelId(label.0),
            payload_tag: super::model::PayloadTag(payload_tag.0),
            next: Box::new(convert_local(next)?),
        },
        choregraph_choreography::LocalProtocol::SendOwned {
            peer,
            label,
            payload_tag,
            value,
            next,
        } => RoleProgram::SendOwned {
            peer: RoleId(peer.0),
            label: super::model::LabelId(label.0),
            payload_tag: super::model::PayloadTag(payload_tag.0),
            value: super::model::ValueId(value.0),
            next: Box::new(convert_local(next)?),
        },
        choregraph_choreography::LocalProtocol::ReceiveOwned {
            peer,
            label,
            payload_tag,
            value,
            next,
        } => RoleProgram::ReceiveOwned {
            peer: RoleId(peer.0),
            label: super::model::LabelId(label.0),
            payload_tag: super::model::PayloadTag(payload_tag.0),
            value: super::model::ValueId(value.0),
            next: Box::new(convert_local(next)?),
        },
        choregraph_choreography::LocalProtocol::InternalChoice { branches } => RoleProgram::InternalChoice {
            branches: branches.iter().map(local_branch).collect::<Result<_, _>>()?,
        },
        choregraph_choreography::LocalProtocol::Offer { from, branches } => RoleProgram::Offer {
            from: RoleId(from.0),
            branches: branches.iter().map(local_branch).collect::<Result<_, _>>()?,
        },
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewedLimits {
    pub max_envelope_bytes: u64,
    pub timeout_ms: u64,
}

#[must_use]
pub const fn reviewed_limits(limits: ReviewedLimits) -> AdapterLimits {
    AdapterLimits {
        max_envelope_bytes: limits.max_envelope_bytes,
        timeout_ms: limits.timeout_ms,
    }
}
