//! Pure typed parameters for workflow-template bundles.
//!
//! A bundle declares a closed list of typed parameters. Supplied values are checked by
//! [`resolve`] and bound into the workflow only as environment values of declared shell
//! nodes, never as command text.

mod binding;
mod grammar;
mod resolution;

pub use binding::apply;
pub use binding::validate_bindings;
pub use binding::validate_bound;
pub use resolution::render;
pub use resolution::resolve;
pub use resolution::validate_declarations;

pub const MAX_PARAMS: usize = 16;
pub const MAX_PARAM_NAME_BYTES: usize = 64;
pub const MAX_PARAM_TEXT_BYTES: usize = 256;
pub const MAX_ENUM_VALUES: usize = 32;
pub const MAX_PARAM_BINDINGS: usize = 8;
pub const MAX_ENV_KEY_BYTES: usize = 64;
pub const MAX_TARGET_LABEL_BYTES: usize = 128;
pub const PARAM_ENV_PREFIX: &str = "LATTICE_PARAM_";
const RESOLVED_IDENTITY_DOMAIN: &str = "lattice.workflow-template-params.v1";
const BOUND_IDENTITY_DOMAIN: &str = "lattice.workflow-template-binding.v1";

/// One declared bundle parameter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamDeclaration {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: ParamType,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<ParamValue>,
    pub bindings: Vec<ParamTarget>,
}

/// The closed set of parameter types. No type admits free-form text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParamType {
    Bool {},
    Integer { min: i64, max: i64 },
    Enum { values: Vec<String> },
    RelativePath { max_bytes: usize },
}

/// A typed parameter value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Bool(bool),
    Integer(i64),
    Text(String),
}

/// One structured input that receives a parameter value: an environment variable of a shell node.
///
/// `node` is the label of exactly one shell node. Labels survive Nickel import, which
/// renumbers node identities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamTarget {
    pub node: String,
    pub shell_env: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedParam {
    pub name: String,
    pub value: ParamValue,
}

/// Every declared parameter with its checked value, sorted by name, and their BLAKE3 identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    values: Vec<ResolvedParam>,
    identity: super::model::ContentIdentity,
}

impl Resolved {
    fn new(mut values: Vec<ResolvedParam>) -> Self {
        values.sort_by(|left, right| left.name.cmp(&right.name));
        let identity = resolved_identity(&values);
        Self { values, identity }
    }

    pub fn values(&self) -> &[ResolvedParam] {
        &self.values
    }

    pub fn get(&self, name: &str) -> Option<&ParamValue> {
        self.values.iter().find(|param| param.name == name).map(|param| &param.value)
    }

    /// The identity a bound bundle, saved workflow, or run should bind for these values.
    pub fn identity(&self) -> &super::model::ContentIdentity {
        &self.identity
    }
}

/// The resolved values bound into one bundle, and the template they were bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    pub template: super::model::ContentIdentity,
    pub resolved: Resolved,
    /// The workflow revision the values produced; the bound identity commits to it.
    pub workflow_revision: crate::orchestration::WorkflowRevisionRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamErrorCode {
    InvalidDeclaration,
    UnknownParam,
    WrongType,
    OutOfBounds,
    MissingRequired,
    TooManyValues,
}

/// A safe parameter diagnostic. It names only a declared parameter and never echoes a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamError {
    pub code: ParamErrorCode,
    pub param: Option<String>,
}

impl ParamError {
    fn new(code: ParamErrorCode, param: Option<&str>) -> Self {
        Self {
            code,
            param: param.filter(|name| resolution::valid_name(name)).map(str::to_owned),
        }
    }

    pub const fn message(&self) -> &'static str {
        match self.code {
            ParamErrorCode::InvalidDeclaration => "bundle parameter declaration is invalid",
            ParamErrorCode::UnknownParam => "an undeclared parameter was supplied",
            ParamErrorCode::WrongType => "parameter value has the wrong type",
            ParamErrorCode::OutOfBounds => "parameter value is outside its declared bounds",
            ParamErrorCode::MissingRequired => "a required parameter is missing",
            ParamErrorCode::TooManyValues => "too many parameter values were supplied",
        }
    }
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.param {
            Some(name) => write!(formatter, "{}: {name}", self.message()),
            None => formatter.write_str(self.message()),
        }
    }
}

impl std::error::Error for ParamError {}

/// BLAKE3 over each name, value kind, and rendered value. Checked names and values never
/// contain NUL, so NUL-terminated fields frame them unambiguously.
fn resolved_identity(values: &[ResolvedParam]) -> super::model::ContentIdentity {
    let mut hasher = blake3::Hasher::new();
    update_terminated(&mut hasher, RESOLVED_IDENTITY_DOMAIN.as_bytes());
    for param in values {
        let kind: &[u8] = match &param.value {
            ParamValue::Bool(_) => b"bool",
            ParamValue::Integer(_) => b"integer",
            ParamValue::Text(_) => b"text",
        };
        update_terminated(&mut hasher, param.name.as_bytes());
        update_terminated(&mut hasher, kind);
        update_terminated(&mut hasher, render(&param.value).as_bytes());
    }
    super::model::ContentIdentity::from_bytes(hasher.finalize().as_bytes())
}

/// The identity of a template bundle bound to exact resolved values and the workflow revision
/// they produced.
pub fn bound_identity(
    template: &super::model::ContentIdentity,
    resolved: &Resolved,
    workflow_revision: &crate::orchestration::WorkflowRevisionRef,
) -> super::model::ContentIdentity {
    let mut hasher = blake3::Hasher::new();
    update_terminated(&mut hasher, BOUND_IDENTITY_DOMAIN.as_bytes());
    update_terminated(&mut hasher, template.as_str().as_bytes());
    update_terminated(&mut hasher, resolved.identity().as_str().as_bytes());
    update_terminated(&mut hasher, workflow_revision.content_hash.as_str().as_bytes());
    super::model::ContentIdentity::from_bytes(hasher.finalize().as_bytes())
}

fn update_terminated(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    debug_assert!(!bytes.contains(&0));
    hasher.update(bytes);
    hasher.update(&[0]);
}
