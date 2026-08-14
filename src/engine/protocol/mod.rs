//! Durable execution of frozen Choregraph projected role protocols.

pub mod cli;
pub mod core;
pub mod evidence;
mod identity;
pub mod intake;
pub mod model;
pub mod shell;

pub use core::abandon_unknown;
pub use core::admit_observation;
pub use core::create_session;
pub use core::fail_before_dispatch;
pub use core::mark_dispatched_unknown;
pub use core::plan_action;
pub use core::prepare_attempt;
pub use core::record_authorization;
pub use core::replay;
pub use core::retry_unknown;
pub use core::settle_attempt;
pub use core::validate_dispatch_envelope;

pub use evidence::SessionInspection;
pub use evidence::identity;
pub use evidence::inspect;
pub use evidence::safe_diagnostic_text;
pub use intake::FrozenProjection;
pub use intake::ReviewedLimits;
pub use intake::admit_projection;
pub use intake::plan_session;
pub use intake::reviewed_limits;
pub use model::*;
pub use shell::authority::EffectAuthority;
pub use shell::transport::ExternalProfile;
pub use shell::transport::HttpAdapter;
