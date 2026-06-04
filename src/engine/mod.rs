//! Workflow execution engine.
//!
//! Runs DAGs of nodes, threading content-addressed values through ports.
//! Supports both step-through debug and async parallel execution.

pub mod executor;
pub mod runner;
pub mod value;
pub mod workflow;

pub use executor::ExecutionMode;
pub use executor::Executor;
pub use executor::RunId;
pub use executor::StepResult;
pub use runner::RunnerWorkflow;
pub use value::ContentHash;
pub use value::TypeTag;
pub use value::Value;
pub use value::ValueData;
pub use workflow::NodeConfig;
pub use workflow::Workflow;
pub use workflow::WorkflowMeta;
