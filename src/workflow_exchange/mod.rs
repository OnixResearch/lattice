//! Durable application handler for the bounded workflow exchange transport.

mod admission;
mod application;
mod authority;
mod core;
mod model;
mod projection;

pub use core::complete;
pub use core::initial_record;
pub use core::mark_running;
pub use core::mark_unknown;
pub use core::request_cancellation;

pub use admission::handler_profile_identity;
pub use admission::select_route;
pub use admission::validate_handler_profile;
pub use admission::validate_route;
pub use application::ApplicationHandler;
pub use application::load_profile;
pub use application::serve;
pub use authority::RouteExactAuthorityProvider;
pub use model::*;
pub use projection::observation;
pub use projection::retained_log;

#[cfg(test)]
mod tests;
