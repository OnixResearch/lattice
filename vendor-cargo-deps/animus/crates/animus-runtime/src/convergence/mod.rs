//! Binding contracts for robust agent-harness result and supervision rails.
//!
//! The contracts compose existing tool output, execution-stage, and host
//! boundaries. They add no provider, model, transport, or tool authority.

mod data;
mod ledger;
mod projection;
mod reference;
mod supervision;
mod waiting;

pub use data::{Datum, DatumDraft, Error, ResourceTelemetry, StorePort, admit, resolve};
pub use ledger::{Event, EventDraft, EventKind, History, HistoryPolicy, render_history};
pub use projection::{
    HumanView, ModelView, Omission, PersistedRecord, inspect_record, project_human, project_model,
    project_record,
};
pub use reference::{HANDLE_MAX_CHARS, Handle, StoreKind};
pub use supervision::{
    Cycle, CyclePhase, GuardObservation, RunObservation, RunStatus, commit_cycle, observe_batch,
    park_cycle, prepare_cycle,
};
pub use waiting::{
    SilenceDisposition, SilenceObservation, SilencePolicy, Suspended, SuspensionDump, plan_silence,
    project_suspension_dump,
};

#[cfg(test)]
mod tests;
