//! Animus-owned recipe workflow DTOs and pure validation helpers.
//!
//! These contracts adapt reusable workflow recipes without adopting any external
//! file format as runtime authority. Hosts may import or export YAML/JSON, but
//! this module treats all data as untrusted DTOs and performs no I/O or effects.

mod admit;
mod assign;
mod attempt;
mod data;
mod field;
mod graph;
mod shape;

#[cfg(test)]
mod tests;

pub use admit::RecipeInfluence;
pub use admit::RecipeInfluenceStatus;
pub use admit::RecipeProvenanceDraft;
pub use admit::RecipeSourceKind;
pub use admit::RecipeTrustReceipt;
pub use admit::RecipeTrustStatus;
pub use assign::RecipeBinding;
pub use assign::RecipeBindingDraft;
pub use assign::RecipeParameterInput;
pub use assign::RecipeParameterInputDraft;
pub use assign::RecipeParameterInputKind;
pub use assign::SAFE_TEMPLATE_BUILTINS;
pub use assign::bind_recipe_parameters;
pub use attempt::RecipeHostEffectKind;
pub use attempt::RecipeRetryCheck;
pub use attempt::RecipeRetryCheckDraft;
pub use attempt::RecipeRetryDenialReason;
pub use attempt::RecipeRetryPlan;
pub use attempt::RecipeRetryPlanDraft;
pub use attempt::RecipeRetryPolicy;
pub use attempt::RecipeRetryPolicyDraft;
pub use attempt::RecipeRetryRequest;
pub use attempt::RecipeRetryRequestStatus;
pub use attempt::plan_recipe_retry;
pub use data::RecipeWorkflow;
pub use data::RecipeWorkflowDraft;
pub use field::RecipeParameter;
pub use field::RecipeParameterDraft;
pub use field::RecipeParameterKind;
pub use graph::RecipeDagAdmission;
pub use graph::RecipeDagBatch;
pub use graph::RecipeDagBlockReason;
pub use graph::RecipeDagBlockedStep;
pub use graph::RecipeDagPlan;
pub use graph::RecipeDagReceipt;
pub use graph::RecipeDagStep;
pub use graph::RecipeDagStepDraft;
pub use graph::RecipeDagStepKind;
pub use graph::plan_recipe_dag;
pub use shape::RecipeAutomationOutcome;
pub use shape::RecipeAutomationStatus;
pub use shape::RecipeOutputDraft;
pub use shape::RecipeOutputFailure;
pub use shape::RecipeOutputFieldDraft;
pub use shape::RecipeResponseField;
pub use shape::RecipeResponseFieldDraft;
pub use shape::RecipeResponseFieldKind;
pub use shape::RecipeResponseSchema;
pub use shape::RecipeResponseSchemaDraft;
pub use shape::validate_recipe_output;

mod digest;
/// Error raised by recipe workflow DTO validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecipeWorkflowError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// A workflow enum-like field contained an unsupported value.
    UnsupportedWorkflowValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: alloc::string::String,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A named collection contained a duplicate entry.
    DuplicateName {
        /// Boundary collection field name.
        field: &'static str,
        /// Duplicated normalized value.
        value: alloc::string::String,
    },
    /// An optional non-file parameter omitted its required default.
    MissingOptionalDefault {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A file parameter attempted to define a default value.
    FileParameterDefault {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A file parameter was declared optional, which would require an unsafe default.
    OptionalFileParameter {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A select parameter did not declare any allowed option.
    EmptySelectOptions {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A non-select parameter declared select-only options.
    UnexpectedSelectOptions {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A select parameter value was outside its declared options.
    InvalidSelectValue {
        /// Parameter name.
        parameter: alloc::string::String,
        /// Rejected value.
        value: alloc::string::String,
    },
    /// A template marker was not closed.
    MalformedTemplate,
    /// A template referenced a variable that was neither declared nor safe built-in.
    UndeclaredTemplateVariable {
        /// Template variable name.
        variable: alloc::string::String,
    },
    /// A declared parameter was not referenced by the template.
    UnusedParameter {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A required parameter had no supplied value.
    MissingParameterValue {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A parameter value arrived through a source kind that the parameter rejects.
    InvalidParameterSource {
        /// Parameter name.
        parameter: alloc::string::String,
    },
    /// A retry policy declared no bounded attempts.
    ZeroRetryAttempts,
    /// A retry check requested an effect kind that reusable core must never execute.
    UnsafeRetryEffect {
        /// Rejected effect label.
        effect: alloc::string::String,
    },
    /// A BLAKE3 identity field failed lowercase hex validation.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
    },
    /// A recipe DAG collection exceeded its bounded limit.
    DagLimitExceeded {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum allowed items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A recipe DAG declared the same step identifier twice.
    DuplicateDagStep {
        /// Duplicated step identifier.
        step_id: alloc::string::String,
    },
    /// A recipe DAG declared the same dependency edge twice.
    DuplicateDagEdge {
        /// Step declaring the duplicate edge.
        step_id: alloc::string::String,
        /// Duplicated dependency step.
        dependency: alloc::string::String,
    },
    /// A recipe DAG dependency referenced an unknown step.
    UnknownDagDependency {
        /// Step declaring the dependency.
        step_id: alloc::string::String,
        /// Unknown dependency step.
        dependency: alloc::string::String,
    },
    /// A recipe DAG step depended on itself.
    SelfDagEdge {
        /// Step with the self-edge.
        step_id: alloc::string::String,
    },
    /// A recipe DAG contains a dependency cycle.
    DagCycle {
        /// Step participating in the cycle.
        step_id: alloc::string::String,
    },
    /// A recipe DAG label contains unsafe durable evidence material.
    UnsafeDagLabel {
        /// Boundary field name.
        field: &'static str,
        /// Rejected label.
        value: alloc::string::String,
    },
    /// A recipe DAG input reference was not declared as a parameter or upstream output.
    UndeclaredDagInput {
        /// Step declaring the input.
        step_id: alloc::string::String,
        /// Rejected input reference.
        reference: alloc::string::String,
    },
    /// A recipe DAG fan-in output reference was not declared by the upstream step.
    UndeclaredDagOutput {
        /// Step declaring the input.
        step_id: alloc::string::String,
        /// Rejected output reference.
        reference: alloc::string::String,
    },
    /// A recipe DAG step referenced a subrecipe outside admitted workflow metadata.
    UntrustedDagSubrecipe {
        /// Step declaring the subrecipe.
        step_id: alloc::string::String,
        /// Rejected subrecipe identity.
        subrecipe: alloc::string::String,
    },
    /// A recipe DAG step requested a capability outside the workflow ceiling.
    CapabilityWidening {
        /// Step declaring the capability.
        step_id: alloc::string::String,
        /// Rejected capability label.
        capability: alloc::string::String,
    },
    /// A recipe DAG step referenced a validation gate not declared by the workflow.
    MissingDagGate {
        /// Step declaring the gate.
        step_id: alloc::string::String,
        /// Rejected gate label.
        gate: alloc::string::String,
    },
    /// A recipe DAG step referenced a retry check outside the host-mediated retry policy.
    RetryBypass {
        /// Step declaring the retry reference.
        step_id: alloc::string::String,
        /// Rejected retry reference.
        retry_ref: alloc::string::String,
    },
}

impl From<crate::AdapterContractError> for RecipeWorkflowError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl core::fmt::Display for RecipeWorkflowError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => core::fmt::Display::fmt(error, formatter),
            Self::UnsupportedWorkflowValue { field, value } => {
                write!(formatter, "{field} contains unsupported workflow value {value}")
            }
            Self::EmptyCollection { field } => write!(formatter, "{field} must not be empty"),
            Self::DuplicateName { field, value } => {
                write!(formatter, "{field} contains duplicate value {value}")
            }
            Self::MissingOptionalDefault { parameter } => {
                write!(formatter, "optional parameter {parameter} must define a default")
            }
            Self::FileParameterDefault { parameter } => {
                write!(formatter, "file parameter {parameter} must not define a default")
            }
            Self::OptionalFileParameter { parameter } => {
                write!(formatter, "file parameter {parameter} must be required")
            }
            Self::EmptySelectOptions { parameter } => {
                write!(formatter, "select parameter {parameter} must define options")
            }
            Self::UnexpectedSelectOptions { parameter } => {
                write!(formatter, "non-select parameter {parameter} must not define options")
            }
            Self::InvalidSelectValue { parameter, value } => {
                write!(formatter, "parameter {parameter} rejects select value {value}")
            }
            Self::MalformedTemplate => formatter.write_str("template marker is not closed"),
            Self::UndeclaredTemplateVariable { variable } => {
                write!(formatter, "template variable {variable} is not declared")
            }
            Self::UnusedParameter { parameter } => {
                write!(formatter, "parameter {parameter} is not referenced by the template")
            }
            Self::MissingParameterValue { parameter } => {
                write!(formatter, "parameter {parameter} requires a value")
            }
            Self::InvalidParameterSource { parameter } => {
                write!(formatter, "parameter {parameter} arrived from an invalid source")
            }
            Self::ZeroRetryAttempts => formatter.write_str("retry attempts must be bounded"),
            Self::UnsafeRetryEffect { effect } => {
                write!(formatter, "retry effect {effect} is not host-mediated")
            }
            Self::InvalidHash { field } => write!(formatter, "{field} must be a BLAKE3 hex hash"),
            error => fmt_recipe_dag_error(error, formatter),
        }
    }
}

fn fmt_recipe_dag_error(error: &RecipeWorkflowError, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match error {
        RecipeWorkflowError::DagLimitExceeded {
            field,
            max_items,
            actual_items,
        } => write!(formatter, "{field} allows at most {max_items} DAG items; got {actual_items}"),
        RecipeWorkflowError::DuplicateDagStep { step_id } => {
            write!(formatter, "recipe DAG step {step_id} is duplicated")
        }
        RecipeWorkflowError::DuplicateDagEdge { step_id, dependency } => {
            write!(formatter, "recipe DAG step {step_id} repeats dependency {dependency}")
        }
        RecipeWorkflowError::UnknownDagDependency { step_id, dependency } => {
            write!(formatter, "recipe DAG step {step_id} depends on unknown step {dependency}")
        }
        RecipeWorkflowError::SelfDagEdge { step_id } => {
            write!(formatter, "recipe DAG step {step_id} depends on itself")
        }
        RecipeWorkflowError::DagCycle { step_id } => {
            write!(formatter, "recipe DAG cycle includes step {step_id}")
        }
        RecipeWorkflowError::UnsafeDagLabel { field, value } => {
            write!(formatter, "{field} contains unsafe recipe DAG label {value}")
        }
        RecipeWorkflowError::UndeclaredDagInput { step_id, reference } => {
            write!(formatter, "recipe DAG step {step_id} uses undeclared input {reference}")
        }
        RecipeWorkflowError::UndeclaredDagOutput { step_id, reference } => {
            write!(formatter, "recipe DAG step {step_id} uses undeclared output {reference}")
        }
        RecipeWorkflowError::UntrustedDagSubrecipe { step_id, subrecipe } => {
            write!(formatter, "recipe DAG step {step_id} references untrusted subrecipe {subrecipe}")
        }
        RecipeWorkflowError::CapabilityWidening { step_id, capability } => {
            write!(formatter, "recipe DAG step {step_id} widens capability {capability}")
        }
        RecipeWorkflowError::MissingDagGate { step_id, gate } => {
            write!(formatter, "recipe DAG step {step_id} references undeclared gate {gate}")
        }
        RecipeWorkflowError::RetryBypass { step_id, retry_ref } => {
            write!(formatter, "recipe DAG step {step_id} references undeclared retry check {retry_ref}")
        }
        _ => formatter.write_str("recipe workflow validation failed"),
    }
}

fn parse_workflow_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, RecipeWorkflowError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or(RecipeWorkflowError::UnsupportedWorkflowValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, RecipeWorkflowError> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(RecipeWorkflowError::from)
}

fn optional_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, RecipeWorkflowError> {
    value.map(|text| required_text(field_name, text)).transpose()
}

fn text_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<alloc::vec::Vec<alloc::string::String>, RecipeWorkflowError> {
    if required && values.is_empty() {
        return Err(RecipeWorkflowError::EmptyCollection {
            field: field_name.as_str(),
        });
    }

    let mut normalized = alloc::vec::Vec::with_capacity(values.len());
    for value in values {
        let text = required_text(field_name, value)?;
        if contains_text(normalized.as_slice(), text.as_str()) {
            return Err(RecipeWorkflowError::DuplicateName {
                field: field_name.as_str(),
                value: text,
            });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

fn contains_text(values: &[alloc::string::String], needle: &str) -> bool {
    values.iter().any(|value| value.as_str() == needle)
}
