const JSON_PRETTY_PROFILE_ID: &str = "json_pretty";
const JSON_COMPACT_PROFILE_ID: &str = "json_compact";
const TOON_V3_PROFILE_ID: &str = "toon_v3";
const JSON_INPUT_HEADING: &str = "JSON input";
const COMPACT_JSON_INPUT_HEADING: &str = "Compact JSON input";
const TOON_V3_INPUT_HEADING: &str = "TOON v3 input";
const DISPLAY_INDEX_BASE: usize = 1;
const JSON_IMPLEMENTATION: &str = "serde_json";
pub const TOON_V3_IMPLEMENTATION: &str = "toon-format/0.5.0/spec-v3.0";

// r[impl lattice.agent_runtime.structured_input_projection]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum StructuredInputProjection {
    #[default]
    JsonPretty,
    JsonCompact,
    ToonV3,
}

impl StructuredInputProjection {
    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::JsonPretty => JSON_PRETTY_PROFILE_ID,
            Self::JsonCompact => JSON_COMPACT_PROFILE_ID,
            Self::ToonV3 => TOON_V3_PROFILE_ID,
        }
    }

    #[must_use]
    pub const fn implementation(self) -> &'static str {
        match self {
            Self::JsonPretty | Self::JsonCompact => JSON_IMPLEMENTATION,
            Self::ToonV3 => TOON_V3_IMPLEMENTATION,
        }
    }

    const fn heading(self) -> &'static str {
        match self {
            Self::JsonPretty => JSON_INPUT_HEADING,
            Self::JsonCompact => COMPACT_JSON_INPUT_HEADING,
            Self::ToonV3 => TOON_V3_INPUT_HEADING,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionErrorClass {
    EncodeRejected,
    EmptyProjection,
    StrictDecodeRejected,
    RoundTripMismatch,
}

impl ProjectionErrorClass {
    const fn diagnostic(self) -> &'static str {
        match self {
            Self::EncodeRejected => "codec rejected the typed value",
            Self::EmptyProjection => "codec returned an empty projection",
            Self::StrictDecodeRejected => "strict decoder rejected the projection",
            Self::RoundTripMismatch => "strict round trip changed the typed value",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionError {
    pub input_index: usize,
    pub profile: StructuredInputProjection,
    pub class: ProjectionErrorClass,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "structured input {} projection '{}' failed: {}",
            self.input_index,
            self.profile.profile_id(),
            self.class.diagnostic()
        )
    }
}

impl std::error::Error for ProjectionError {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RenderedStructuredInput {
    pub heading: String,
    pub body: String,
    pub profile: StructuredInputProjection,
}

// r[impl lattice.agent_runtime.structured_input_projection.scenario.default]
// r[impl lattice.agent_runtime.toon_v3_projection]
// r[impl lattice.agent_runtime.toon_v3_projection.scenario.supported]
// r[impl lattice.agent_runtime.toon_v3_projection.scenario.rejected]
pub fn render_structured_inputs(
    inputs: &[serde_json::Value],
    profile: StructuredInputProjection,
) -> Result<Vec<RenderedStructuredInput>, ProjectionError> {
    let mut rendered = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        let input_index = index.saturating_add(DISPLAY_INDEX_BASE);
        let body = render_body(input, profile, input_index)?;
        debug_assert!(!body.trim().is_empty());
        rendered.push(RenderedStructuredInput {
            heading: format!("{} {input_index}", profile.heading()),
            body,
            profile,
        });
    }
    debug_assert_eq!(rendered.len(), inputs.len());
    Ok(rendered)
}

pub fn decode_toon_v3_strict(input: &str) -> Result<serde_json::Value, ProjectionErrorClass> {
    crate::decode::decode_default(input).map_err(|_| ProjectionErrorClass::StrictDecodeRejected)
}

fn render_body(
    input: &serde_json::Value,
    profile: StructuredInputProjection,
    input_index: usize,
) -> Result<String, ProjectionError> {
    match profile {
        StructuredInputProjection::JsonPretty => serde_json::to_string_pretty(input)
            .map_err(|_| projection_error(input_index, profile, ProjectionErrorClass::EncodeRejected)),
        StructuredInputProjection::JsonCompact => serde_json::to_string(input)
            .map_err(|_| projection_error(input_index, profile, ProjectionErrorClass::EncodeRejected)),
        StructuredInputProjection::ToonV3 => render_toon_v3(input, input_index),
    }
}

fn render_toon_v3(input: &serde_json::Value, input_index: usize) -> Result<String, ProjectionError> {
    let profile = StructuredInputProjection::ToonV3;
    let encoded = crate::encode::encode_default(input)
        .map_err(|_| projection_error(input_index, profile, ProjectionErrorClass::EncodeRejected))?;
    if encoded.trim().is_empty() {
        return Err(projection_error(input_index, profile, ProjectionErrorClass::EmptyProjection));
    }
    let decoded = decode_toon_v3_strict(&encoded).map_err(|class| projection_error(input_index, profile, class))?;
    if decoded != *input {
        return Err(projection_error(input_index, profile, ProjectionErrorClass::RoundTripMismatch));
    }
    Ok(encoded)
}

const fn projection_error(
    input_index: usize,
    profile: StructuredInputProjection,
    class: ProjectionErrorClass,
) -> ProjectionError {
    ProjectionError {
        input_index,
        profile,
        class,
    }
}
