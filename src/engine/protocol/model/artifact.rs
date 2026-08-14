//! Producer artifact, placement, and session-plan DTOs.

use serde::Deserialize;
use serde::Serialize;

use super::Identity;
use super::OwnedValue;
use super::RoleId;
use super::RoleProgram;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RoleKind {
    Native,
    External {
        contract: Identity,
        input_schema: Identity,
        output_schema: Identity,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefinition {
    pub role: RoleId,
    pub kind: RoleKind,
    pub local_artifact: Identity,
    pub protocol: RoleProgram,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    pub source_revision: String,
    pub adapter_identity: Identity,
    pub global_artifact: Identity,
    pub complete_local_set: Identity,
    pub projection_receipt: Identity,
    pub schema_set: Identity,
    pub proof_reference: Identity,
    pub roles: Vec<RoleDefinition>,
    pub initial_owners: Vec<OwnedValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PlacementKind {
    Native {
        runtime_route: Identity,
    },
    External {
        contract: Identity,
        input_schema: Identity,
        output_schema: Identity,
        destination_policy: Identity,
        credential_reference: Identity,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterLimits {
    pub max_envelope_bytes: u64,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RolePlacement {
    pub role: RoleId,
    pub adapter_cohort: Identity,
    pub limits: AdapterLimits,
    pub kind: PlacementKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleInstance {
    pub role: RoleId,
    pub identity: Identity,
    pub local_artifact: Identity,
    pub placement: RolePlacement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPlan {
    pub identity: Identity,
    pub session: Identity,
    pub nonce: Identity,
    pub cohort: Cohort,
    pub roles: Vec<RoleInstance>,
}
