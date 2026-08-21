//! Workflow execution engine.
//!
//! Runs DAGs of nodes, threading content-addressed values through ports.
//! Supports both step-through debug and async parallel execution.

pub mod child;
pub mod domain;
pub mod effect;
pub mod executor;
pub mod frontier;
pub mod protocol;
pub mod runner;
pub mod value;
pub mod workflow;

pub use child::RepositorySubworkflowRunnerShell;
pub use child::SubworkflowCorrelation;
pub use child::SubworkflowEvent;
pub use child::SubworkflowEventKind;
pub use child::SubworkflowEventMetadata;
pub use child::SubworkflowExecution;
pub use child::SubworkflowExecutionError;
pub use child::SubworkflowInvocation;
pub use child::SubworkflowRunnerShell;
pub use child::WorkflowCatalogRepository;
pub use domain::NodeId;
pub use domain::PortId;
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
pub use workflow::WorkflowTimestamp;
