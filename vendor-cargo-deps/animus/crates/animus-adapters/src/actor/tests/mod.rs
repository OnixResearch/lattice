use super::*;

mod negative;
mod positive;

const NAME: &str = "reviewer";
const PROJECT_ID: &str = "agent:project:reviewer";
const USER_ID: &str = "agent:user:reviewer";
const HOST_ID: &str = "agent:host:reviewer";
const SECOND_PROJECT_ID: &str = "agent:project:reviewer:two";
const DESCRIPTION: &str = "Reviews changes without extra authority";
const MODEL_ROLE: &str = "review-model";
const PROMPT_FRAGMENT: &str = "Review for safety and correctness.";
const SECOND_PROMPT_FRAGMENT: &str = "Report blockers before suggesting execution.";
const CAPABILITY: &str = "code-review";
const EXTRA_CAPABILITY: &str = "network-admin";
const TOOLSET: &str = "read-only-tools";
const EXTRA_TOOLSET: &str = "shell-tools";
const MEMORY_SCOPE: &str = "session-summary";
const EXTRA_MEMORY_SCOPE: &str = "hidden-memory";
const CONTEXT_SCOPE: &str = "repo-docs";
const NETWORK_CONTEXT_SCOPE: &str = "network:prod";
const WORKFLOW_ABILITY: &str = "review-only";
const EXTRA_WORKFLOW_ABILITY: &str = "background-worker";
const REDACTION_MARKER: &str = "raw-prompt-redacted";
const SECOND_REDACTION_MARKER: &str = "provider-payload-redacted";
const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UNSUPPORTED_SCOPE: &str = "workspace";
const SAFE_RECEIPT_TEXT: &str =
    "agent=reviewer source=user model=review-model manifest_hash=abc redaction=raw-prompt-redacted";
const RAW_PROMPT_RECEIPT: &str = "agent=reviewer raw_prompt=secret instructions";
const FULL_PATH_RECEIPT: &str = "agent=reviewer path=/home/example/project";
const CREDENTIAL_RECEIPT: &str = "agent=reviewer secret=token";
const PROVIDER_RECEIPT: &str = "agent=reviewer provider_json={}";
const MEMORY_RECEIPT: &str = "agent=reviewer hidden_memory=private";
const DUPLICATE_DIAGNOSTIC_COUNT: usize = 1;
const EMPTY_SELECTION_COUNT: usize = 0;

fn user_draft() -> Draft<'static> {
    Draft {
        name: Some(NAME),
        description: Some(DESCRIPTION),
        model_preference: Some(MODEL_ROLE),
        prompt_fragments: &[PROMPT_FRAGMENT, SECOND_PROMPT_FRAGMENT],
        capability_ceilings: &[CAPABILITY],
        toolset_limits: &[TOOLSET],
        memory_scopes: &[MEMORY_SCOPE],
        context_reference_scopes: &[CONTEXT_SCOPE],
        workflow_abilities: &[WORKFLOW_ABILITY],
        source_scope: Some(USER_SOURCE_LABEL),
        trust_status: Some(TRUSTED_LABEL),
        manifest_identity: Some(USER_ID),
    }
}

fn project_draft() -> Draft<'static> {
    Draft {
        source_scope: Some(PROJECT_SOURCE_LABEL),
        trust_status: Some(UNTRUSTED_LABEL),
        manifest_identity: Some(PROJECT_ID),
        ..user_draft()
    }
}

fn host_draft() -> Draft<'static> {
    Draft {
        source_scope: Some(HOST_SOURCE_LABEL),
        manifest_identity: Some(HOST_ID),
        ..user_draft()
    }
}

fn user_manifest() -> Manifest {
    Manifest::validate(&user_draft()).expect("user manifest should validate")
}

fn project_manifest() -> Manifest {
    Manifest::validate(&project_draft()).expect("project manifest should validate")
}

fn full_policy() -> ActivationPolicy<'static> {
    ActivationPolicy {
        model_roles: &[MODEL_ROLE],
        capabilities: &[CAPABILITY],
        tools: &[TOOLSET],
        memory_scopes: &[MEMORY_SCOPE],
        context_reference_scopes: &[CONTEXT_SCOPE],
        workflow_abilities: &[WORKFLOW_ABILITY],
    }
}

fn admitted_project() -> Admission<'static> {
    Admission {
        admitted_manifest_identities: &[PROJECT_ID],
    }
}

fn no_admission() -> Admission<'static> {
    Admission {
        admitted_manifest_identities: &[],
    }
}

fn safe_hash() -> crate::SafeHash {
    crate::SafeHash::new(VALID_HASH).expect("test hash should be valid BLAKE3 hex")
}
