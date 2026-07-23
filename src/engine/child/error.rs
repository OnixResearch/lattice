impl std::fmt::Display for super::SubworkflowExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunnerUnavailable => write!(formatter, "subworkflow runner shell is not configured"),
            Self::MissingParentInput(label) => write!(formatter, "mapped parent input '{label}' has no value"),
            Self::MissingParentInputPort(label) => {
                write!(formatter, "mapped parent input port '{label}' does not exist")
            }
            Self::DuplicateChildInput(label) => write!(formatter, "child input '{label}' is mapped more than once"),
            Self::MissingParentOutputPort(label) => {
                write!(formatter, "mapped parent output port '{label}' does not exist")
            }
            Self::DuplicateParentOutput(label) => write!(formatter, "parent output '{label}' is mapped more than once"),
            Self::MissingChildOutput(label) => write!(formatter, "child output '{label}' was not produced"),
            Self::MissingChildInputPort(label) => write!(formatter, "child input port '{label}' does not exist"),
            Self::AmbiguousChildInputPort(label) => write!(formatter, "child input port '{label}' is ambiguous"),
            Self::AmbiguousChildOutput(label) => write!(formatter, "child output '{label}' is ambiguous"),
            Self::ChildWorkflowMissing(name) => write!(formatter, "saved child workflow '{name}' does not exist"),
            Self::ChildWorkflowLoadFailed => write!(formatter, "saved child workflow could not be loaded"),
            Self::ChildWorkflowExecutionFailed => write!(formatter, "saved child workflow execution failed"),
            Self::WorkflowIdentityMismatch => {
                write!(formatter, "subworkflow result identity does not match the invocation")
            }
            Self::MissingLifecycleEvents => write!(formatter, "subworkflow result omitted required lifecycle events"),
            Self::LifecycleOrderMismatch => {
                write!(formatter, "subworkflow lifecycle events are not ordered start-to-completion")
            }
            Self::EventCorrelationMismatch => {
                write!(formatter, "subworkflow event correlation does not match the invocation")
            }
        }
    }
}

impl std::error::Error for super::SubworkflowExecutionError {}
