const RECIPE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_recipe_id");
const FAMILY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_route_family");
const ROLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_model_role");
const OUTPUT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_output_contract");
const FAMILY_SINGLE: &str = "single";
const FAMILY_CONFIDENCE: &str = "confidence";
const FAMILY_RATINGS: &str = "ratings";
const FAMILY_REMOM: &str = "remom";
const FAMILY_FUSION: &str = "fusion";
const FAMILY_WORKFLOW: &str = "workflow";
const ROLE_PRIMARY: &str = "primary";
const ROLE_ESCALATION: &str = "escalation";
const ROLE_RATER: &str = "rater";
const ROLE_SYNTHESIZER: &str = "synthesizer";
const ROLE_JUDGE: &str = "judge";
const ROLE_FINALIZER: &str = "finalizer";
const ROLE_WORKER: &str = "worker";
const ROLE_REPAIR: &str = "repair";
const ROLE_FALLBACK: &str = "fallback";
const CONTRACT_TEXT: &str = "text";
const CONTRACT_JSON: &str = "json";
const CONTRACT_TOOL_CALL: &str = "tool_call";
const CONTRACT_ARTIFACT: &str = "artifact";
/// Model-router recipe family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteFamily {
    /// One selected model handles the request.
    Single,
    /// A primary model escalates when confidence signals require it.
    Confidence,
    /// Multiple raters produce bounded ratings before selection.
    Ratings,
    /// ReMoM-style workers produce quorum evidence for synthesis.
    Remom,
    /// Judge and finalizer roles resolve disagreement.
    Fusion,
    /// A bounded role workflow runs behind one model-service surface.
    Workflow,
}
impl RouteFamily {
    /// Parses a stable route-family label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(FAMILY_FIELD, value, Self::from_normalized)
    }
    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            FAMILY_SINGLE => Some(Self::Single),
            FAMILY_CONFIDENCE => Some(Self::Confidence),
            FAMILY_RATINGS => Some(Self::Ratings),
            FAMILY_REMOM => Some(Self::Remom),
            FAMILY_FUSION => Some(Self::Fusion),
            FAMILY_WORKFLOW => Some(Self::Workflow),
            _ => None,
        }
    }
    /// Returns the stable route-family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Single => FAMILY_SINGLE,
            Self::Confidence => FAMILY_CONFIDENCE,
            Self::Ratings => FAMILY_RATINGS,
            Self::Remom => FAMILY_REMOM,
            Self::Fusion => FAMILY_FUSION,
            Self::Workflow => FAMILY_WORKFLOW,
        }
    }
}
/// Role used inside a router recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// Primary model role.
    Primary,
    /// Escalation model role.
    Escalation,
    /// Rating worker role.
    Rater,
    /// Quorum synthesizer role.
    Synthesizer,
    /// Judge role.
    Judge,
    /// Finalizer role.
    Finalizer,
    /// Generic bounded workflow worker role.
    Worker,
    /// Bounded repair role.
    Repair,
    /// Safe fallback role.
    Fallback,
}
impl Role {
    /// Parses a stable role label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(ROLE_FIELD, value, Self::from_normalized)
    }
    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            ROLE_PRIMARY => Some(Self::Primary),
            ROLE_ESCALATION => Some(Self::Escalation),
            ROLE_RATER => Some(Self::Rater),
            ROLE_SYNTHESIZER => Some(Self::Synthesizer),
            ROLE_JUDGE => Some(Self::Judge),
            ROLE_FINALIZER => Some(Self::Finalizer),
            ROLE_WORKER => Some(Self::Worker),
            ROLE_REPAIR => Some(Self::Repair),
            ROLE_FALLBACK => Some(Self::Fallback),
            _ => None,
        }
    }
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => ROLE_PRIMARY,
            Self::Escalation => ROLE_ESCALATION,
            Self::Rater => ROLE_RATER,
            Self::Synthesizer => ROLE_SYNTHESIZER,
            Self::Judge => ROLE_JUDGE,
            Self::Finalizer => ROLE_FINALIZER,
            Self::Worker => ROLE_WORKER,
            Self::Repair => ROLE_REPAIR,
            Self::Fallback => ROLE_FALLBACK,
        }
    }
}
/// Final output contract required from a router recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputContract {
    /// Plain text response.
    Text,
    /// JSON response.
    Json,
    /// Tool-call response.
    ToolCall,
    /// Structured artifact response.
    Artifact,
}
impl OutputContract {
    /// Parses a stable output-contract label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(OUTPUT_FIELD, value, Self::from_normalized)
    }
    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            CONTRACT_TEXT => Some(Self::Text),
            CONTRACT_JSON => Some(Self::Json),
            CONTRACT_TOOL_CALL => Some(Self::ToolCall),
            CONTRACT_ARTIFACT => Some(Self::Artifact),
            _ => None,
        }
    }
    /// Returns the stable output-contract label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => CONTRACT_TEXT,
            Self::Json => CONTRACT_JSON,
            Self::ToolCall => CONTRACT_TOOL_CALL,
            Self::Artifact => CONTRACT_ARTIFACT,
        }
    }
}
/// Recipe contract checked before provider dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recipe {
    id: alloc::string::String,
    family: RouteFamily,
    roles: alloc::vec::Vec<Role>,
    output_contract: OutputContract,
    bounds: super::Bounds,
}
impl Recipe {
    /// Builds a validated router recipe.
    ///
    /// # Errors
    ///
    /// Returns [`super::ValidationError`] when the recipe is malformed, has
    /// duplicate roles, unknown labels, missing output contract, or invalid bounds.
    pub fn new(draft: RecipeDraft<'_>) -> Result<Self, super::ValidationError> {
        let id = crate::validation::normalized_adapter_field(RECIPE_ID_FIELD, draft.id)?;
        let family = RouteFamily::parse(draft.family)?;
        let roles = collect_roles(draft.roles)?;
        let output_contract = OutputContract::parse(draft.output_contract)?;
        let bounds = super::Bounds::new(super::BoundsDraft {
            max_candidates: draft.max_candidates,
            max_parallelism: draft.max_parallelism,
            min_success_quorum: draft.min_success_quorum,
            timeout_class: draft.timeout_class,
            max_synthesis_attempts: draft.max_synthesis_attempts,
            max_repair_attempts: draft.max_repair_attempts,
            fallback_policy: draft.fallback_policy,
        })?;
        Ok(Self {
            id,
            family,
            roles,
            output_contract,
            bounds,
        })
    }
    /// Returns the stable recipe identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
    /// Returns the route family.
    #[must_use]
    pub const fn family(&self) -> RouteFamily {
        self.family
    }
    /// Returns required roles.
    #[must_use]
    pub fn roles(&self) -> &[Role] {
        self.roles.as_slice()
    }
    /// Returns the final output contract.
    #[must_use]
    pub const fn output_contract(&self) -> OutputContract {
        self.output_contract
    }
    /// Returns hard execution bounds.
    #[must_use]
    pub const fn bounds(&self) -> &super::Bounds {
        &self.bounds
    }
}
/// Borrowed fields used to validate a router recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeDraft<'a> {
    /// Stable recipe identifier.
    pub id: &'a str,
    /// Route-family label.
    pub family: &'a str,
    /// Required role labels.
    pub roles: &'a [&'a str],
    /// Required final output contract label.
    pub output_contract: &'a str,
    /// Hard candidate count cap.
    pub max_candidates: u32,
    /// Hard parallelism cap.
    pub max_parallelism: u32,
    /// Minimum successful candidates required.
    pub min_success_quorum: u32,
    /// Timeout class label.
    pub timeout_class: &'a str,
    /// Hard synthesis attempt cap.
    pub max_synthesis_attempts: u32,
    /// Bounded repair attempt cap.
    pub max_repair_attempts: Option<u32>,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
}
fn collect_roles(values: &[&str]) -> Result<alloc::vec::Vec<Role>, super::ValidationError> {
    if values.is_empty() {
        return Err(super::ValidationError::EmptyRoles);
    }
    let mut roles = alloc::vec::Vec::with_capacity(values.len());
    for value in values {
        let role = Role::parse(value)?;
        if roles.contains(&role) {
            return Err(super::ValidationError::DuplicateRole { role });
        }
        roles.push(role);
    }
    Ok(roles)
}
