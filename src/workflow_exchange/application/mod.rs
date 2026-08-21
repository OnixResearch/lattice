#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the application composition module exposes explicit core and adapter boundaries to its child modules"
)]

mod handler;
mod service;
mod shell;
mod worker;

pub use service::load_profile;
pub use service::serve;
pub use shell::ApplicationHandler;
pub(super) use worker::workflow_facts;

pub(super) use super::AdmissionResolution;
pub(super) use super::ApplicationError;
pub(super) use super::CompletionFacts;
pub(super) use super::DurableExecutionState;
pub(super) use super::DurableNodeResult;
pub(super) use super::EffectAuthorityMode;
pub(super) use super::ExecutionRecord;
pub(super) use super::HandlerProfile;
pub(super) use super::RouteExactAuthorityProvider;
pub(super) use super::WorkflowFacts;
pub(super) use super::WorkflowRoute;
pub(super) use super::complete;
pub(super) use super::handler_profile_identity;
pub(super) use super::instantiate_workflow;
pub(super) use super::mark_running;
pub(super) use super::mark_unknown;
pub(super) use super::observation;
pub(super) use super::request_cancellation;
pub(super) use super::retained_log;
pub(super) use super::select_route;
pub(super) use super::validate_handler_profile;
pub(super) use super::validate_route;
