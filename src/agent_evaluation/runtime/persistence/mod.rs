#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrialRecord {
    pub plan: super::planning::TrialPlan,
    pub status: super::model::TrialStatus,
    pub started_epoch_ms: u64,
    pub updated_epoch_ms: u64,
    pub cancel_requested: bool,
    pub receipt_identity: Option<String>,
}
