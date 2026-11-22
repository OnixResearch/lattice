//! Bounded local workflow exchange transport for Lattice consumers.
//!
//! The functional core owns wire identity, admission, linkage, safe failure
//! classification, the reconnect rule table, and observation continuity. The
//! Unix and vsock shells own finite socket I/O and invoke only an explicit
//! application-owned workflow handler.

#![forbid(unsafe_code)]

mod core;
#[path = "vsock.rs"]
mod guest;
#[path = "service.rs"]
mod handler;
#[path = "reconnect/continuity.rs"]
mod history;
#[cfg(target_os = "linux")]
#[path = "unix.rs"]
mod local;
mod model;
#[path = "reconnect/rules.rs"]
mod rules;
#[path = "frame.rs"]
mod wire;

pub use core::new_request;
pub use core::new_response;
pub use core::request_identity;
pub use core::response_identity;
pub use core::validate_exchange_request;
pub use core::validate_exchange_response;
pub use core::validate_response_limits;
pub use core::validate_transport_profile;

pub use guest::VSOCK_CID_ANY;
pub use guest::VSOCK_PORT_ANY;
#[cfg(target_os = "linux")]
pub use guest::VsockListener;
#[cfg(target_os = "linux")]
pub use guest::VsockPeer;
#[cfg(target_os = "linux")]
pub use guest::VsockStream;
#[cfg(target_os = "linux")]
pub use guest::serve_vsock;
#[cfg(target_os = "linux")]
pub use guest::serve_vsock_listener;
pub use guest::validate_vsock_transport_profile;
pub use guest::vsock_peer_is_admitted;
pub use handler::WorkflowExchangeHandler;
pub use handler::WorkflowExchangeService;
pub use history::Continuity;
pub use history::ContinuityRejection;
pub use history::LogStreamKey;
pub use history::OBSERVATION_CONTINUITY_SCHEMA;
pub use history::continuity;
#[cfg(target_os = "linux")]
pub use local::Listener;
#[cfg(target_os = "linux")]
pub use local::UnixClient;
#[cfg(target_os = "linux")]
pub use local::UnixEndpoint;
#[cfg(target_os = "linux")]
pub use local::UnixPeer;
#[cfg(target_os = "linux")]
pub use local::exchange_connected;
#[cfg(target_os = "linux")]
pub use local::serve;
#[cfg(target_os = "linux")]
pub use local::serve_listener;
pub use model::*;
pub use rules::CommandKind;
pub use rules::EXCHANGE_RECOVERY_SCHEMA;
pub use rules::ExchangeOutcome;
pub use rules::FailureClass;
pub use rules::REJECTION_CODES;
pub use rules::RecoveryStep;
pub use rules::control_after_observe;
pub use rules::next_step;
pub use wire::FrameFailure;
pub use wire::encode_frame;
pub use wire::read_frame;
pub use wire::write_frame;

#[cfg(test)]
mod tests;
