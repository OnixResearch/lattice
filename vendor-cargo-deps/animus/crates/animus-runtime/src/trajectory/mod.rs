//! Pure validation and existing-owner projection for trajectory-v1 transcripts.

mod model;
mod projection;
mod validate;

pub use model::{
    TrajectoryBatchDraft, TrajectoryIssue, TrajectoryOmission, TrajectoryProjection,
    TrajectoryReceipt, TrajectoryReceiptInput, TrajectoryReceiptSensitiveDraft, TrajectoryRole,
    TrajectoryRoleCounts, TrajectoryValidation,
};
pub use projection::{project_trajectory, trajectory_receipt};
pub use validate::{validate_policy, validate_trajectory};

#[cfg(test)]
mod tests;
