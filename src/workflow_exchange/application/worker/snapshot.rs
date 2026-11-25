#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    reason = "snapshot dispatch mirrors WorkerInput across the AppConfig and authority-only headless ports"
)]

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub(super) struct SnapshotDispatch<'a> {
    pub store: &'a crate::store::Store,
    pub workflow: crate::engine::workflow::Workflow,
    pub run_id: crate::engine::RunId,
    pub agent_runtime: crate::agent_node::RuntimeOptions,
    pub app_config: Option<crate::config::AppConfig>,
    pub cancellation: Arc<AtomicBool>,
    pub authority: &'a dyn crate::engine::effect::execution::RunContextProvider,
    pub restored: Vec<crate::engine::executor::StepResult>,
    pub progress: Option<crate::engine::executor::progress::Sender>,
}

pub(super) fn run_snapshot(
    dispatch: SnapshotDispatch<'_>,
) -> Option<Result<crate::headless::SnapshotRunOutcome, crate::headless::HeadlessRunError>> {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().ok()?;
    let result = match dispatch.app_config {
        Some(app_config) => runtime.block_on(
            crate::headless::AppConfigSnapshotRun {
                workflow: dispatch.workflow,
                run_id: dispatch.run_id,
                owner: crate::store::RunOwner::Unscoped,
                cancellation: dispatch.cancellation,
            }
            .run_from_checkpoint(
                dispatch.store,
                &app_config,
                dispatch.authority,
                dispatch.restored,
                dispatch.progress,
            ),
        ),
        None => runtime.block_on(
            crate::headless::SnapshotRunRequest {
                workflow: dispatch.workflow,
                run_id: dispatch.run_id,
                owner: crate::store::RunOwner::Unscoped,
                agent_runtime: dispatch.agent_runtime,
                cancellation: dispatch.cancellation,
            }
            .run_from_checkpoint(dispatch.store, dispatch.authority, dispatch.restored, dispatch.progress),
        ),
    };
    Some(result)
}
