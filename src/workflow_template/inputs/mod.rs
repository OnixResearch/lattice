//! Typed per-run inputs for saved workflows retained from a workflow-template bundle. Pure.
//!
//! `lattice run NAME --input NAME=VALUE --input-file PATH` and `authority inspect` supply raw
//! values. They are typed by the declarations of the exact bundle the saved workflow was
//! activated from, screened for secret-like content, and bound through [`super::bind`], which
//! re-runs full bundle admission. Nothing here reads files, the store, or a clock.

mod admission;
mod parsing;
mod secrets;

pub use admission::admit;
pub use admission::has_approval_gate;
pub use admission::is_current;
pub use admission::reverify_admitted;
pub use parsing::collect;
pub use secrets::screen;

/// The largest `--input-file` accepted, in bytes.
pub const MAX_INPUT_FILE_BYTES: u64 = 65_536;

/// One supplied value before the retained declarations give it a type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawInput {
    /// A `--input NAME=VALUE` value, typed by its declaration.
    Text(String),
    /// A value of the `--input-file` JSON object, typed by its JSON type.
    Json(serde_json::Value),
}

/// The template a saved workflow was activated from, retained by identity at save.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedBinding {
    pub bundle: String,
    pub template: super::model::ContentIdentity,
    pub workflow: crate::orchestration::WorkflowRevisionRef,
}

/// The binding to retain for a bound bundle, or `None` for an unbound catalog template.
pub fn retained_binding(bundle: &super::model::ValidatedBundle) -> Option<RetainedBinding> {
    let bound = bundle.params()?;
    debug_assert_ne!(&bound.template, bundle.identity());
    Some(RetainedBinding {
        bundle: bundle.manifest().id.clone(),
        template: bound.template.clone(),
        workflow: bundle.workflow_revision().clone(),
    })
}

/// The declared type of one input, without its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Bool,
    Integer,
    Enum,
    RelativePath,
}

impl InputKind {
    pub const fn of(value_type: &super::params::ParamType) -> Self {
        match value_type {
            super::params::ParamType::Bool {} => Self::Bool,
            super::params::ParamType::Integer { .. } => Self::Integer,
            super::params::ParamType::Enum { .. } => Self::Enum,
            super::params::ParamType::RelativePath { .. } => Self::RelativePath,
        }
    }
}

/// One declared parameter of a bound run: its name, kind, and whether the caller supplied it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputName {
    pub name: String,
    pub kind: InputKind,
    pub supplied: bool,
}

/// The content-omitting identity of one admitted input set. It holds no value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputIdentity {
    pub bundle: String,
    pub template: super::model::ContentIdentity,
    /// BLAKE3 over the template identity, `values`, and the bound workflow revision.
    pub bound: super::model::ContentIdentity,
    /// The bound workflow revision that runs and that `authority inspect` reports.
    pub workflow: crate::orchestration::WorkflowRevisionRef,
    /// The `params::Resolved` identity over every name, kind, and value.
    pub values: super::model::ContentIdentity,
    /// Every declared parameter, sorted by name.
    pub params: Vec<InputName>,
}

/// The input identity persisted with one run, keyed by its run id.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInputRecord {
    pub run_id: crate::engine::RunId,
    pub inputs: InputIdentity,
}

/// What a caller loaded for one saved workflow, supplied to [`admit`].
pub struct SavedFacts<'a> {
    pub workflow_name: &'a str,
    pub saved: Option<&'a crate::engine::Workflow>,
    pub binding: Option<&'a RetainedBinding>,
    /// The bundle loaded from the directory the binding names, if any.
    pub template: Option<&'a super::model::ValidatedBundle>,
}

/// A bundle bound to admitted inputs, and their content-omitting identity.
///
/// Only [`admit`] constructs one, so holding one means every input check ran. A run still
/// re-verifies it with [`reverify_admitted`] before it reserves a run id.
#[derive(Debug, Clone)]
pub struct Admitted {
    bundle: super::model::ValidatedBundle,
    identity: InputIdentity,
}

impl Admitted {
    pub fn bundle(&self) -> &super::model::ValidatedBundle {
        &self.bundle
    }

    pub fn identity(&self) -> &InputIdentity {
        &self.identity
    }

    /// The bound workflow to run and the identity to record, without copying the workflow.
    pub(crate) fn into_run_parts(self) -> (crate::engine::Workflow, InputIdentity) {
        (self.bundle.workflow, self.identity)
    }

    /// Tamper with the bound bundle to test start re-verification.
    #[cfg(test)]
    pub(crate) fn bundle_mut(&mut self) -> &mut super::model::ValidatedBundle {
        &mut self.bundle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretReason {
    Name,
    Value,
}

/// A safe input diagnostic. It never echoes a supplied value or an undeclared name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    MalformedInput,
    MalformedFile,
    DuplicateInput,
    WorkflowNotFound,
    BindingMissing,
    NotParameterized,
    SavedWorkflowDrift,
    TemplateUnavailable,
    TemplateLoad(super::model::Diagnostic),
    ApprovalGate,
    Param(super::params::ParamError),
    SecretLike { param: String, reason: SecretReason },
    Bind(super::model::Diagnostic),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedInput => formatter.write_str("each --input must be NAME=VALUE with a nonempty NAME"),
            Self::MalformedFile => formatter.write_str("--input-file must hold one JSON object of input values"),
            Self::DuplicateInput => formatter.write_str("an input name is supplied more than once"),
            Self::WorkflowNotFound => formatter.write_str("saved workflow not found"),
            Self::BindingMissing => {
                formatter.write_str("no retained workflow-template binding; save it again from its bundle")
            }
            Self::NotParameterized => formatter.write_str("saved workflow declares no parameters; inputs are refused"),
            Self::SavedWorkflowDrift => {
                formatter.write_str("saved workflow no longer matches the template binding it was saved with")
            }
            Self::TemplateUnavailable => {
                formatter.write_str("the exact workflow-template bundle this workflow was saved from is unavailable")
            }
            Self::TemplateLoad(diagnostic) => write!(
                formatter,
                "the workflow-template bundle this workflow was saved from failed to load: {}: {}",
                diagnostic.code.as_str(),
                diagnostic.message
            ),
            Self::ApprovalGate => formatter.write_str("inputs are refused for a workflow with an approval gate"),
            Self::Param(error) => error.fmt(formatter),
            Self::SecretLike { param, reason } => {
                let what = match reason {
                    SecretReason::Name => "its name",
                    SecretReason::Value => "its value",
                };
                write!(
                    formatter,
                    "parameter {param} is secret-like by {what}; pass secrets only as credential references"
                )
            }
            Self::Bind(diagnostic) => write!(formatter, "{}: {}", diagnostic.code.as_str(), diagnostic.message),
        }
    }
}

impl std::error::Error for InputError {}
