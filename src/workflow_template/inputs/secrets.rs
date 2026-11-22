//! The secret-like input rule. Pure.
//!
//! Parameter types already exclude free text, but a relative path or enum token can still
//! carry a token-shaped string. Run inputs are never a secret channel; credential references
//! are. The rule refuses:
//!
//! - a declared parameter whose name, or whose binding key after `LATTICE_PARAM_`, has a secret
//!   word as an `_`-separated word (see [`SECRET_WORDS`]) or a qualifier followed by `key` (see
//!   [`KEY_QUALIFIERS`]);
//! - a declared binding whose target node binds the same environment key to a credential reference;
//! - a supplied text value with a `/`-separated component that starts with a known credential
//!   prefix followed by at least [`MIN_PREFIXED_SUFFIX_BYTES`] bytes, or that is at least
//!   [`TOKEN_SHAPE_MIN_BYTES`] bytes of only ASCII letters, digits, `_` and `-` mixing uppercase,
//!   lowercase and digits.
//!
//! Every declared parameter is checked by name and target, because binding writes every
//! declared target, defaults included. Only supplied values are checked by value; defaults
//! are reviewed manifest bytes.

pub const SECRET_WORDS: [&str; 17] = [
    "token",
    "tokens",
    "secret",
    "secrets",
    "password",
    "passwords",
    "passwd",
    "passphrase",
    "credential",
    "credentials",
    "apikey",
    "privatekey",
    "auth",
    "bearer",
    "cookie",
    "pat",
    "otp",
];
pub const KEY_QUALIFIERS: [&str; 8] = [
    "api",
    "private",
    "access",
    "secret",
    "session",
    "signing",
    "client",
    "encryption",
];
pub const CREDENTIAL_PREFIXES: [&str; 24] = [
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "xoxa-",
    "xoxb-",
    "xoxp-",
    "xoxr-",
    "xoxs-",
    "sk-",
    "sk_live_",
    "sk_test_",
    "rk_live_",
    "AKIA",
    "ASIA",
    "AIza",
    "ya29.",
    "npm_",
    "pypi-",
    "hf_",
    "eyJ",
];
pub const MIN_PREFIXED_SUFFIX_BYTES: usize = 8;
pub const TOKEN_SHAPE_MIN_BYTES: usize = 32;

// r[impl lattice.workflow_templates.run_inputs.secrets]
/// Refuse secret-like parameter names, binding keys, credential-bearing targets, and supplied
/// values. The diagnostic names only the declared parameter.
pub fn screen(
    declared: &[super::super::params::ParamDeclaration],
    workflow: &crate::engine::Workflow,
    supplied: &std::collections::BTreeMap<String, super::super::params::ParamValue>,
) -> Result<(), super::InputError> {
    for declaration in declared {
        let refuse = |reason| super::InputError::SecretLike {
            param: declaration.name.clone(),
            reason,
        };
        let keys = declaration.bindings.iter().map(|target| {
            target.shell_env.strip_prefix(super::super::params::PARAM_ENV_PREFIX).unwrap_or(&target.shell_env)
        });
        if is_secret_identifier(&declaration.name) || keys.clone().any(is_secret_identifier) {
            return Err(refuse(super::SecretReason::Name));
        }
        if declaration.bindings.iter().any(|target| binds_credential(workflow, target)) {
            return Err(refuse(super::SecretReason::CredentialTarget));
        }
        if let Some(super::super::params::ParamValue::Text(value)) = supplied.get(&declaration.name)
            && is_secret_value(value)
        {
            return Err(refuse(super::SecretReason::Value));
        }
    }
    debug_assert!(supplied.keys().all(|name| declared.iter().any(|declaration| &declaration.name == name)));
    Ok(())
}

/// Whether an identifier has a secret word, or a qualifier followed by `key`, as `_`-separated
/// words, ignoring ASCII case.
pub fn is_secret_identifier(identifier: &str) -> bool {
    let lowered = identifier.to_ascii_lowercase();
    let words = lowered.split('_').collect::<Vec<_>>();
    let has_secret_word = words.iter().any(|word| SECRET_WORDS.contains(word));
    let has_qualified_key = words.windows(2).any(|pair| KEY_QUALIFIERS.contains(&pair[0]) && pair[1] == "key");
    has_secret_word || has_qualified_key
}

/// Whether a text value has a credential-prefixed or token-shaped `/`-separated component.
pub fn is_secret_value(value: &str) -> bool {
    value.split('/').any(|component| {
        let is_prefixed = CREDENTIAL_PREFIXES.iter().any(|prefix| {
            component.starts_with(prefix) && component.len() >= prefix.len().saturating_add(MIN_PREFIXED_SUFFIX_BYTES)
        });
        is_prefixed || is_token_shaped(component)
    })
}

fn is_token_shaped(component: &str) -> bool {
    let bytes = component.as_bytes();
    let has_token_alphabet = bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    let has_mixed_classes = bytes.iter().any(u8::is_ascii_uppercase)
        && bytes.iter().any(u8::is_ascii_lowercase)
        && bytes.iter().any(u8::is_ascii_digit);
    bytes.len() >= TOKEN_SHAPE_MIN_BYTES && has_token_alphabet && has_mixed_classes
}

/// Whether any node with the target's label binds its environment key to a credential reference.
fn binds_credential(workflow: &crate::engine::Workflow, target: &super::super::params::ParamTarget) -> bool {
    workflow.graph.nodes().filter(|node| node.label == target.node).any(|node| {
        workflow
            .node_credentials(node.id)
            .is_some_and(|credentials| credentials.env.contains_key(&target.shell_env))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn secret_identifiers_are_word_matched() {
        for name in [
            "api_token",
            "github_pat",
            "db_password",
            "api_key",
            "PRIVATE_KEY",
            "auth",
            "client_secret",
        ] {
            assert!(super::is_secret_identifier(name), "{name}");
        }
        for name in [
            "catalog_root",
            "author",
            "sort_key",
            "tokenizer",
            "path",
            "count",
            "format",
        ] {
            assert!(!super::is_secret_identifier(name), "{name}");
        }
    }

    #[test]
    fn credential_prefixes_and_token_shapes_are_secret_values() {
        let shaped = "Ab1".repeat(11);
        for value in [
            "ghp_0123456789abcdef",
            "dir/github_pat_0123456789",
            "AKIAIOSFODNN7EXAMPLE",
            "xoxb-12345678",
            "sk-proj-abcdefgh",
            shaped.as_str(),
        ] {
            assert!(super::is_secret_value(value), "{value}");
        }
        let lower_hex = "0123456789abcdef0123456789abcdef01234567";
        for value in [
            "builtins/agent-evaluations",
            "src/ghp_",
            "sk-a",
            lower_hex,
            "Readme.md",
            "a/B/c1",
        ] {
            assert!(!super::is_secret_value(value), "{value}");
        }
    }
}
