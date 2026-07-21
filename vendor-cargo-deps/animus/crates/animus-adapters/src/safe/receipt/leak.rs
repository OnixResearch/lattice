const TOKEN_PREFIX: &str = "sk-";
const GITHUB_TOKEN_PREFIX: &str = "ghp_";
const TOKEN_ASSIGNMENT_MARKER: &str = "token=";
const API_KEY_MARKER: &str = "api_key";
const API_KEY_DASH_MARKER: &str = "api-key";
const BEARER_MARKER: &str = "bearer ";
const RAW_PROMPT_MARKER: &str = "prompt:";
const SYSTEM_PROMPT_MARKER: &str = "system prompt";
const USER_PROMPT_MARKER: &str = "user:";
const PROVIDER_CHOICES_MARKER: &str = "\"choices\"";
const PROVIDER_MESSAGES_MARKER: &str = "\"messages\"";
const PROVIDER_MODEL_MARKER: &str = "\"model\"";
const TOOL_INPUT_MARKER: &str = "tool_input:";
const TOOL_INPUT_TEXT_MARKER: &str = "tool input:";
const TOOL_OUTPUT_MARKER: &str = "tool_output:";
const TOOL_OUTPUT_TEXT_MARKER: &str = "tool output:";
const MODEL_OUTPUT_MARKER: &str = "model_output:";
const MODEL_OUTPUT_TEXT_MARKER: &str = "model output:";
const ASSISTANT_OUTPUT_MARKER: &str = "assistant:";
const ROOT_PATH_MARKER: &str = "/";
const HOME_PATH_MARKER: &str = "/home/";
const TMP_PATH_MARKER: &str = "/tmp/";
const ETC_PATH_MARKER: &str = "/etc/";
const USER_PATH_MARKER: &str = "\\users\\";
const JSON_OBJECT_PREFIX: &str = "{";

const PROMPT_MARKERS: &[&str] = &[RAW_PROMPT_MARKER, SYSTEM_PROMPT_MARKER, USER_PROMPT_MARKER];
const PROVIDER_MARKERS: &[&str] = &[
    PROVIDER_CHOICES_MARKER,
    PROVIDER_MESSAGES_MARKER,
    PROVIDER_MODEL_MARKER,
];
const CREDENTIAL_MARKERS: &[&str] = &[
    TOKEN_PREFIX,
    GITHUB_TOKEN_PREFIX,
    TOKEN_ASSIGNMENT_MARKER,
    API_KEY_MARKER,
    API_KEY_DASH_MARKER,
    BEARER_MARKER,
];
const PATH_MARKERS: &[&str] = &[
    HOME_PATH_MARKER,
    TMP_PATH_MARKER,
    ETC_PATH_MARKER,
    USER_PATH_MARKER,
];
const TOOL_INPUT_MARKERS: &[&str] = &[TOOL_INPUT_MARKER, TOOL_INPUT_TEXT_MARKER];
const TOOL_OUTPUT_MARKERS: &[&str] = &[TOOL_OUTPUT_MARKER, TOOL_OUTPUT_TEXT_MARKER];
const MODEL_OUTPUT_MARKERS: &[&str] = &[
    MODEL_OUTPUT_MARKER,
    MODEL_OUTPUT_TEXT_MARKER,
    ASSISTANT_OUTPUT_MARKER,
];

pub(super) fn class(value: &str) -> Option<super::LeakClass> {
    assert!(!value.is_empty(), "safe metadata value must not be empty");
    assert!(
        value.len() >= value.chars().count(),
        "byte length covers chars"
    );
    let lower = value.to_ascii_lowercase();
    let trimmed = lower.trim();
    if contains_any(trimmed, PROMPT_MARKERS) {
        return Some(super::LeakClass::RawPrompt);
    }
    if is_provider_json(trimmed) {
        return Some(super::LeakClass::ProviderJson);
    }
    class_after_provider(trimmed)
}

fn class_after_provider(value: &str) -> Option<super::LeakClass> {
    if contains_any(value, CREDENTIAL_MARKERS) {
        return Some(super::LeakClass::Credential);
    }
    if is_path_like(value) {
        return Some(super::LeakClass::AbsolutePath);
    }
    class_after_path(value)
}

fn class_after_path(value: &str) -> Option<super::LeakClass> {
    if contains_any(value, TOOL_INPUT_MARKERS) {
        return Some(super::LeakClass::RawToolInput);
    }
    if contains_any(value, TOOL_OUTPUT_MARKERS) {
        return Some(super::LeakClass::RawToolOutput);
    }
    if contains_any(value, MODEL_OUTPUT_MARKERS) {
        return Some(super::LeakClass::RawModelOutput);
    }
    None
}

fn is_provider_json(value: &str) -> bool {
    if !value.starts_with(JSON_OBJECT_PREFIX) {
        return false;
    }
    contains_any(value, PROVIDER_MARKERS)
}

fn is_path_like(value: &str) -> bool {
    if value.starts_with(ROOT_PATH_MARKER) {
        return true;
    }
    contains_any(value, PATH_MARKERS)
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    assert!(!markers.is_empty(), "marker list must not be empty");
    assert!(
        value.len() >= value.chars().count(),
        "byte length covers chars"
    );
    markers.iter().any(|marker| value.contains(marker))
}
