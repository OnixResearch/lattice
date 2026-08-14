// r[impl lattice.architecture.core_crate.compatibility]
pub mod agent_node;
pub mod animus_agent;
mod capability;
pub mod config;
pub mod content_identity;
pub mod control_flow {
    pub use lattice_core::control_flow::*;
}
pub mod credentials;
pub mod engine;
pub mod error;
pub mod expressions;
pub mod file_node;
pub mod headless;
pub mod http;
pub mod image_node;
pub mod n8n_json;
pub mod nickel;
pub mod node_kind_provider;
pub mod openai;
pub mod orchestration;
pub mod process;
pub mod protocol_cli {
    pub use crate::engine::protocol::cli::*;
}
pub mod replay;
pub mod run;
pub mod scheduler;
pub mod store;
pub mod triggers;
pub mod tui;
pub mod webhooks;
pub mod workflow;
pub mod workflow_format;
pub mod workflow_template;
pub mod workflow_validation;
