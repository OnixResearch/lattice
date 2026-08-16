#![allow(
    unknown_lints,
    ambiguous_params,
    reason = "analyzer input fields remain adjacent to their typed manifest and references at each private call site"
)]
#![allow(
    assertion_density,
    reason = "untrusted analyzer inputs and outputs return typed errors instead of terminating the process"
)]

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataReference {
    pub version_identity: String,
    pub content_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub schema: String,
    pub pack_identity: String,
    pub analyzer_identity: String,
    pub run_id: Option<u64>,
    pub run_state: Option<String>,
    pub data_references: Vec<DataReference>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub schema: String,
    pub structured: serde_json::Value,
    pub markdown: Option<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Completed,
    Failed,
    Missing,
    TimedOut,
    Truncated,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub analyzer_id: String,
    pub analyzer_identity: String,
    pub required: bool,
    pub status: Status,
    pub published_version_identity: Option<String>,
    pub diagnostic: String,
    pub content_omitted: bool,
}

pub fn select_for_direct(analyzers: &[super::model::AnalyzerManifest]) -> Vec<&super::model::AnalyzerManifest> {
    analyzers
        .iter()
        .filter(|analyzer| {
            matches!(analyzer.scope, super::model::AnalyzerScope::Method | super::model::AnalyzerScope::Operation)
        })
        .collect()
}

pub fn plan_input(
    pack_identity: &str,
    analyzer: &super::model::AnalyzerManifest,
    run_id: u64,
    run_state: &str,
    references: &[DataReference],
) -> Result<Input, super::Error> {
    super::validation::validate_identity(pack_identity)?;
    let analyzer_identity = super::identity::analyzer_identity(analyzer)?;
    let should_include_run = analyzer.data_view.contains(&super::model::AnalyzerDataView::RunMetadata);
    let should_include_references = analyzer.data_view.contains(&super::model::AnalyzerDataView::ExactDataReferences);
    for reference in references {
        super::validation::validate_identity(&reference.version_identity)?;
        super::validation::validate_identity(&reference.content_identity)?;
    }
    Ok(Input {
        schema: super::model::ANALYZER_INPUT_SCHEMA.to_string(),
        pack_identity: pack_identity.to_string(),
        analyzer_identity,
        run_id: should_include_run.then_some(run_id),
        run_state: should_include_run.then(|| run_state.to_string()),
        data_references: if should_include_references {
            references.to_vec()
        } else {
            Vec::new()
        },
    })
}

pub fn classify_output(
    analyzer: &super::model::AnalyzerManifest,
    bytes: &[u8],
    prohibited_fragments: &[String],
) -> Result<Output, super::Error> {
    if bytes.len() > crate::operation::model::MAXIMUM_OPERATION_BYTES {
        return Err(super::Error::Invalid("analyzer output bound"));
    }
    if contains_fragment(bytes, prohibited_fragments) {
        return Err(super::Error::Invalid("analyzer secret output"));
    }
    let output: Output = serde_json::from_slice(bytes).map_err(|_| super::Error::Invalid("analyzer output schema"))?;
    if output.schema != super::model::ANALYZER_OUTPUT_SCHEMA {
        return Err(super::Error::Invalid("analyzer output schema"));
    }
    if value_has_prohibited_key(&output.structured) {
        return Err(super::Error::Invalid("analyzer output authority"));
    }
    match &output.markdown {
        Some(markdown) if !analyzer.markdown_allowed || markdown.len() > analyzer.maximum_markdown_bytes => {
            return Err(super::Error::Invalid("analyzer Markdown policy"));
        }
        Some(markdown) if contains_private_path(markdown) => {
            return Err(super::Error::Invalid("analyzer private path"));
        }
        Some(_) | None => {}
    }
    for required in &analyzer.required_non_claims {
        if !output.non_claims.contains(required) {
            return Err(super::Error::Invalid("analyzer non-claim"));
        }
    }
    Ok(output)
}

pub fn parent_succeeded(operation_completed: bool, outcomes: &[Outcome]) -> bool {
    operation_completed && outcomes.iter().all(|outcome| !outcome.required || outcome.status == Status::Completed)
}

pub fn failed_outcome(
    analyzer: &super::model::AnalyzerManifest,
    status: Status,
    diagnostic: &'static str,
) -> Result<Outcome, super::Error> {
    Ok(Outcome {
        analyzer_id: analyzer.id.clone(),
        analyzer_identity: super::identity::analyzer_identity(analyzer)?,
        required: analyzer.required,
        status,
        published_version_identity: None,
        diagnostic: diagnostic.to_string(),
        content_omitted: true,
    })
}

fn contains_fragment(bytes: &[u8], fragments: &[String]) -> bool {
    fragments.iter().any(|fragment| {
        let fragment = fragment.as_bytes();
        !fragment.is_empty() && bytes.windows(fragment.len()).any(|window| window == fragment)
    })
}

fn value_has_prohibited_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, nested)| {
            crate::operation::schema::key_is_secret_bearing(key)
                || matches!(key.as_str(), "authority" | "release_ready" | "production_ready" | "property_proved")
                || value_has_prohibited_key(nested)
        }),
        serde_json::Value::Array(values) => values.iter().any(value_has_prohibited_key),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => false,
    }
}

fn contains_private_path(value: &str) -> bool {
    value.contains("/home/") || value.contains("/Users/") || value.contains("C:\\Users\\")
}
