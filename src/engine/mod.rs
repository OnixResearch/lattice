//! Workflow execution engine.
//!
//! Runs DAGs of nodes, threading content-addressed values through ports.
//! Supports both step-through debug and async parallel execution.

pub mod executor;
pub mod value;
pub mod workflow;

pub use executor::{ExecutionMode, Executor, RunId, StepResult};
pub use value::{ContentHash, TypeTag, Value, ValueData};
pub use workflow::{NodeConfig, Workflow, WorkflowMeta};
