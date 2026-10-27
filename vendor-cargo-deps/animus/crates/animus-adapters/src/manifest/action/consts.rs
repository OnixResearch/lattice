pub(super) const ROLE_KIND_ACTION: &str = "action";
pub(super) const ROLE_KIND_EVENT_HOOK: &str = "event-hook";
pub(super) const ROLE_KIND_HOST_PANE_ENTRYPOINT: &str = "host-pane-entrypoint";
pub(super) const ROLE_KIND_LINK_HANDLER: &str = "link-handler";
pub(super) const EVENT_WORKSPACE_OPENED: &str = "workspace-opened";
pub(super) const EVENT_REQUEST_STARTED: &str = "request-started";
pub(super) const EVENT_REQUEST_FINISHED: &str = "request-finished";
pub(super) const EVENT_AGENT_STATUS_CHANGED: &str = "agent-status-changed";
pub(super) const UNSAFE_PATH_PARENT: &str = "..";
pub(super) const UNSAFE_SHELL_AND: &str = "&&";
pub(super) const HOST_PATH_HOME_MARKER: &str = "/home/";
pub(super) const HOST_PATH_TEMP_MARKER: &str = "/tmp/";
pub(super) const WINDOWS_USERS_MARKER: &str = "\\Users\\";
pub(super) const RAW_PROMPT_MARKER: &str = "RAW_PROMPT:";
pub(super) const TRANSCRIPT_MARKER: &str = "TRANSCRIPT:";
pub(super) const CREDENTIAL_MARKER: &str = "SECRET=";
pub(super) const UNBOUNDED_LOG_MARKER: &str = "UNBOUNDED_LOG:";
pub(super) const PROVIDER_PAYLOAD_MARKER: &str = "PROVIDER_PAYLOAD:";
pub(super) const VERSION_SEPARATOR: char = '.';
pub(super) const ROLE_PATH_SEPARATOR: char = '/';
pub(super) const ROLE_PATH_SEPARATOR_TEXT: &str = "/";
pub(super) const ROLE_WINDOWS_SEPARATOR_TEXT: &str = "\\";
pub(super) const ROLE_SHELL_VARIABLE_TEXT: &str = "$";
pub(super) const ROLE_SHELL_COMMAND_TEXT: &str = ";";
pub(super) const ROLE_PIPE_TEXT: &str = "|";
pub(super) const ROLE_NEWLINE_TEXT: &str = "\n";
pub(super) const ROLE_CARRIAGE_RETURN_TEXT: &str = "\r";
pub(super) const COMPATIBILITY_VERSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_minimum_host_version");
pub(super) const COMPATIBILITY_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_required_host_capability");
pub(super) const COMPATIBILITY_PLATFORM_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_supported_platform");
pub(super) const ROLE_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_kind");
pub(super) const ROLE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_id");
pub(super) const ROLE_TITLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_title");
pub(super) const ROLE_CONTEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_context");
pub(super) const ROLE_COMMAND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_command_label");
pub(super) const ROLE_ARTIFACT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_artifact_label");
pub(super) const ROLE_COMMAND_OR_ARTIFACT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_command_or_artifact_label");
pub(super) const ROLE_EVENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_event_name");
pub(super) const ROLE_LINK_SCHEME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("plugin_role_link_scheme");
pub(super) const HOST_VERSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_version");
pub(super) const HOST_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_capability");
pub(super) const HOST_PLATFORM_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_platform");
pub(super) const CONTEXT_WORKSPACE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_workspace_label");
pub(super) const CONTEXT_TAB_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_tab_label");
pub(super) const CONTEXT_PANE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_pane_label");
pub(super) const CONTEXT_AGENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_agent_status_label");
pub(super) const CONTEXT_WORKTREE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_worktree_provenance_label");
pub(super) const CONTEXT_SESSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_session_provenance_label");
pub(super) const CONTEXT_REQUEST_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_request_identity");
pub(super) const CONTEXT_PLUGIN_ROOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_plugin_root_label");
pub(super) const CONTEXT_PLUGIN_CONFIG_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_plugin_config_label");
pub(super) const CONTEXT_PLUGIN_STATE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_plugin_state_label");
pub(super) const CONTEXT_SOURCE_METADATA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("invocation_source_metadata_label");
pub(super) const KNOWN_EVENT_NAMES: &[&str] = &[
    EVENT_WORKSPACE_OPENED,
    EVENT_REQUEST_STARTED,
    EVENT_REQUEST_FINISHED,
    EVENT_AGENT_STATUS_CHANGED,
];
