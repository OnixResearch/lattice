use super::super::super::*;
use super::super::support::*;
use super::fixtures::*;

#[test]
fn validates_safe_context_and_rejects_leaky_contexts() {
    let context = safe_context();
    assert_eq!(context.workspace_label(), WORKSPACE_LABEL);
    assert_eq!(context.request_identity(), REQUEST_IDENTITY);

    let raw_prompt = unsafe_context(CONTEXT_WORKSPACE, RAW_PROMPT_CONTEXT);
    let transcript = unsafe_context(CONTEXT_WORKSPACE, TRANSCRIPT_CONTEXT);
    let credential = unsafe_context(CONTEXT_WORKSPACE, CREDENTIAL_CONTEXT);
    let private_path = unsafe_context(CONTEXT_WORKSPACE, PRIVATE_PATH_CONTEXT);
    let unbounded_log = unsafe_context(CONTEXT_WORKSPACE, UNBOUNDED_LOG_CONTEXT);
    let provider_payload = unsafe_context(CONTEXT_WORKSPACE, PROVIDER_PAYLOAD_CONTEXT);

    assert_context_leak(raw_prompt, PluginInvocationContextLeak::RawPrompt);
    assert_context_leak(transcript, PluginInvocationContextLeak::Transcript);
    assert_context_leak(credential, PluginInvocationContextLeak::Credential);
    assert_context_leak(private_path, PluginInvocationContextLeak::FullPrivatePath);
    assert_context_leak(unbounded_log, PluginInvocationContextLeak::UnboundedLog);
    assert_context_leak(provider_payload, PluginInvocationContextLeak::ProviderPayload);
}

const CONTEXT_WORKSPACE: ContextField = ContextField::Workspace;

#[derive(Clone, Copy)]
enum ContextField {
    Workspace,
}

fn unsafe_context(field: ContextField, value: &'static str) -> PluginInvocationContextDraft<'static> {
    let mut draft = safe_context_draft();
    match field {
        ContextField::Workspace => draft.workspace_label = Some(value),
    }
    draft
}

fn assert_context_leak(draft: PluginInvocationContextDraft<'_>, leak: PluginInvocationContextLeak) {
    let error = PluginInvocationContext::validate(&draft).expect_err("unsafe context should fail");
    assert_eq!(error, PluginDescriptorError::UnsafeInvocationContext {
        field: "invocation_workspace_label",
        leak,
    });
}
