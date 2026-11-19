//! Declared preference-factor metadata recorded from an admitted annotated cohort.
//!
//! These DTOs carry metadata only. No cursor, label, ownership, or authority
//! decision reads them.

use serde::Deserialize;
use serde::Serialize;

use super::Identity;
use super::LabelId;
use super::RoleId;

pub const PREFERENCE_REPORT_SCHEMA: &str = "lattice.role-protocol-preference-report.v1";
/// Exact producer profile selector that an annotated cohort must carry.
pub const ANNOTATED_PRODUCER_PROFILE: &str = "choregraph.role-choreography.profile.preference-annotations.v1";
/// Exact producer receipt claim for an annotated cohort.
pub const ANNOTATED_PRODUCER_CLAIM: &str = "identity-linkage-only-no-claim-promotion";
/// Exact producer non-claims that every recorded annotation keeps.
pub const ANNOTATION_NON_CLAIMS: [&str; 9] = [
    "no-branch-selection",
    "no-equilibrium-analysis",
    "no-game-semantics",
    "no-label-boolean-conversion",
    "no-preference-authority",
    "no-self-interested-projection-soundness",
    "no-solver",
    "no-utility-function",
    "no-world-distribution",
];

/// Session profile selected by the embedding caller. The exact-label baseline is the default.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreferenceProfile {
    #[default]
    ExactLabelBaseline,
    RecordDeclaredFactors,
}

/// What happened to the declared factors that arrived with an annotated cohort.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactorDisposition {
    FactorsRecorded,
    FactorsIgnoredProfileOff,
}

/// Status of any mapping from the annotated protocol to a formal game.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameMapping {
    NotFormalized,
}

/// One step from a protocol node to its continuation, as declared by the producer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ChoiceStep {
    Next,
    Branch { label: LabelId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredFactor {
    pub id: u32,
    pub role: RoleId,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredWorld {
    pub id: u32,
    pub name: String,
}

/// Declared factors and optional world variable on one decider `InternalChoice`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredChoice {
    pub global_path: Vec<ChoiceStep>,
    pub local_path: Vec<ChoiceStep>,
    pub labels: Vec<LabelId>,
    pub factors: Vec<DeclaredFactor>,
    pub world: Option<DeclaredWorld>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleChoices {
    pub role: RoleId,
    pub local_artifact: Identity,
    pub decisions: Vec<DeclaredChoice>,
}

/// Identity-bound declared-factor metadata of one admitted annotated cohort.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceRecord {
    pub producer_profile: String,
    pub annotated_receipt: Identity,
    pub base_receipt: Identity,
    pub complete_local_set: Identity,
    pub annotations: Identity,
    pub annotated_local_set: Identity,
    pub game_mapping: GameMapping,
    pub non_claims: Vec<String>,
    pub roles: Vec<RoleChoices>,
}

/// Intake report that separates recorded factors, ignored factors, and the unformalized game
/// mapping.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceReport {
    pub schema: String,
    pub profile: PreferenceProfile,
    pub disposition: FactorDisposition,
    pub game_mapping: GameMapping,
    pub annotated_receipt: Identity,
    pub non_claims: Vec<String>,
}
