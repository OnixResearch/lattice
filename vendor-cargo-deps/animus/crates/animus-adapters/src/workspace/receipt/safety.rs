const RAW_PROMPT_MARKERS: &[&str] = &["RAW_PROMPT:", "<raw-prompt>", "raw prompt"];
const CREDENTIAL_MARKERS: &[&str] = &[
    "BEGIN OPENSSH PRIVATE KEY",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
];
const ABSOLUTE_PATH_MARKERS: &[&str] = &["/home/", "C:\\Users\\", "\\Users\\"];
const RAW_FILE_MARKERS: &[&str] = &["RAW_FILE:", "raw file contents"];
const RAW_PROCESS_MARKERS: &[&str] = &["RAW_PROCESS_OUTPUT:", "process stdout"];
const RAW_MANIFEST_MARKERS: &[&str] = &["RAW_BRANCH_MANIFEST:", "branch manifest body"];
const RAW_NETWORK_MARKERS: &[&str] = &["RAW_NETWORK_PAYLOAD:", "provider payload"];
const UNBOUNDED_EVENT_MARKERS: &[&str] = &["UNBOUNDED_EVENTS:", "unbounded event stream"];

/// Workspace receipt leak class detected in candidate evidence text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Leak {
    /// Raw prompt text appeared in receipt evidence.
    RawPrompt,
    /// Credential-looking material appeared in receipt evidence.
    Credential,
    /// Absolute or private host path appeared in receipt evidence.
    AbsolutePath,
    /// Raw file body appeared in receipt evidence.
    RawFileContents,
    /// Raw process output appeared in receipt evidence.
    RawProcessOutput,
    /// Raw branch manifest appeared in receipt evidence.
    RawBranchManifest,
    /// Raw network/provider payload appeared in receipt evidence.
    RawNetworkPayload,
    /// Unbounded event stream appeared in receipt evidence.
    UnboundedEventStream,
}

const LEAK_MARKERS: &[(Leak, &[&str])] = &[
    (Leak::RawPrompt, RAW_PROMPT_MARKERS),
    (Leak::Credential, CREDENTIAL_MARKERS),
    (Leak::AbsolutePath, ABSOLUTE_PATH_MARKERS),
    (Leak::RawFileContents, RAW_FILE_MARKERS),
    (Leak::RawProcessOutput, RAW_PROCESS_MARKERS),
    (Leak::RawBranchManifest, RAW_MANIFEST_MARKERS),
    (Leak::RawNetworkPayload, RAW_NETWORK_MARKERS),
    (Leak::UnboundedEventStream, UNBOUNDED_EVENT_MARKERS),
];

impl Leak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::Credential => "credential",
            Self::AbsolutePath => "absolute_path",
            Self::RawFileContents => "raw_file_contents",
            Self::RawProcessOutput => "raw_process_output",
            Self::RawBranchManifest => "raw_branch_manifest",
            Self::RawNetworkPayload => "raw_network_payload",
            Self::UnboundedEventStream => "unbounded_event_stream",
        }
    }
}

/// Validates candidate workspace receipt evidence text for obvious sensitive material.
///
/// # Errors
///
/// Returns [`Leak`] when the text contains a known unsafe marker for raw prompts,
/// credentials, full paths, file bodies, process output, branch manifests, network
/// payloads, or unbounded event streams.
pub fn validate_text(text: &str) -> Result<(), Leak> {
    first_leak(text).map_or(Ok(()), Err)
}

fn first_leak(text: &str) -> Option<Leak> {
    LEAK_MARKERS.iter().find_map(|(leak, markers)| contains_marker(text, markers).then_some(*leak))
}

fn contains_marker(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}
