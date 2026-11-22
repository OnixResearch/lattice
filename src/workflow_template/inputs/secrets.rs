//! The secret-like input rule. Pure.
//!
//! Parameter types already exclude free text, but a `relative_path` value is caller text over
//! letters, digits, `.`, `_`, `-` and `/`, so it can still carry a token. Run inputs are never a
//! secret channel; credential references are. The rule applies only to `relative_path`
//! parameters. A `bool` or bounded `integer` cannot carry a credential, and an `enum` value is
//! one of the reviewed manifest tokens, like a default. It refuses:
//!
//! - a `relative_path` parameter whose name, or whose binding key after `LATTICE_PARAM_`, has a
//!   secret word as an `_`-separated word (see [`SECRET_WORDS`]) or a qualifier followed by `key`
//!   (see [`KEY_QUALIFIERS`]), ignoring ASCII case;
//! - a supplied `relative_path` value with a `/`-separated component that is a known provider
//!   token: its prefix followed by at least that format's minimum suffix drawn only from that
//!   format's alphabet (see [`TOKEN_FORMATS`]);
//! - a supplied `relative_path` value with a run between separators (`/`, `-`, `_`, `.`) of at
//!   least [`TOKEN_RUN_MIN_BYTES`] ASCII letters and digits that mixes uppercase, lowercase and
//!   digits.
//!
//! A binding into a credential-bound key cannot occur: every target key starts with
//! `LATTICE_PARAM_`, and #27's `validate_targets` refuses any node credential key with that
//! prefix at load.

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

/// A random run of this many letters and digits with mixed classes is token-shaped.
///
/// Real tokens are long separator-free runs: base62 and base64 keys are 20 to 100 or more
/// characters. Words in refs and paths (`PROJ`, `1234`, `Safari15`) are short runs between
/// separators. A 20-character mixed-class run is uncommon in a path and covers the shortest
/// provider tokens.
pub const TOKEN_RUN_MIN_BYTES: usize = 20;

/// The alphabet of a provider token's suffix after its prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenAlphabet {
    /// ASCII letters and digits.
    Alphanumeric,
    /// Uppercase ASCII letters and digits.
    UpperAlphanumeric,
    /// Letters, digits and `_`.
    AlphanumericUnderscore,
    /// Letters, digits and `-`.
    AlphanumericDash,
    /// Letters, digits, `-` and `_`.
    Base64Url,
    /// Letters, digits, `-`, `_` and `.`.
    Base64UrlDot,
}

/// One provider token format: its prefix, the minimum suffix length, and the suffix alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenFormat {
    pub prefix: &'static str,
    pub min_suffix_bytes: usize,
    pub alphabet: TokenAlphabet,
}

const fn format(prefix: &'static str, min_suffix_bytes: usize, alphabet: TokenAlphabet) -> TokenFormat {
    TokenFormat {
        prefix,
        min_suffix_bytes,
        alphabet,
    }
}

pub const TOKEN_FORMATS: [TokenFormat; 26] = [
    format("ghp_", 30, TokenAlphabet::Alphanumeric),
    format("gho_", 30, TokenAlphabet::Alphanumeric),
    format("ghu_", 30, TokenAlphabet::Alphanumeric),
    format("ghs_", 30, TokenAlphabet::Alphanumeric),
    format("ghr_", 30, TokenAlphabet::Alphanumeric),
    format("github_pat_", 40, TokenAlphabet::AlphanumericUnderscore),
    format("glpat-", 20, TokenAlphabet::Base64Url),
    format("xoxa-", 20, TokenAlphabet::AlphanumericDash),
    format("xoxb-", 20, TokenAlphabet::AlphanumericDash),
    format("xoxp-", 20, TokenAlphabet::AlphanumericDash),
    format("xoxr-", 20, TokenAlphabet::AlphanumericDash),
    format("xoxs-", 20, TokenAlphabet::AlphanumericDash),
    format("sk-proj-", 40, TokenAlphabet::Base64Url),
    format("sk-svcacct-", 40, TokenAlphabet::Base64Url),
    format("sk-", 32, TokenAlphabet::Alphanumeric),
    format("sk_live_", 20, TokenAlphabet::Alphanumeric),
    format("sk_test_", 20, TokenAlphabet::Alphanumeric),
    format("rk_live_", 20, TokenAlphabet::Alphanumeric),
    format("AKIA", 16, TokenAlphabet::UpperAlphanumeric),
    format("ASIA", 16, TokenAlphabet::UpperAlphanumeric),
    format("AIza", 30, TokenAlphabet::Base64Url),
    format("ya29.", 20, TokenAlphabet::Base64UrlDot),
    format("npm_", 30, TokenAlphabet::Alphanumeric),
    format("pypi-", 50, TokenAlphabet::Base64Url),
    format("hf_", 30, TokenAlphabet::Alphanumeric),
    format("eyJ", 30, TokenAlphabet::Base64UrlDot),
];

// r[impl lattice.workflow_templates.run_inputs.secrets]
/// Refuse secret-like `relative_path` parameter names, binding keys, and supplied values. The
/// diagnostic names only the declared parameter.
pub fn screen(
    declared: &[super::super::params::ParamDeclaration],
    supplied: &std::collections::BTreeMap<String, super::super::params::ParamValue>,
) -> Result<(), super::InputError> {
    let caller_text = declared
        .iter()
        .filter(|declaration| matches!(declaration.value_type, super::super::params::ParamType::RelativePath { .. }));
    for declaration in caller_text {
        // #27's `validate_targets` refuses a credential key with this prefix, so no target can
        // receive a credential.
        debug_assert!(
            declaration
                .bindings
                .iter()
                .all(|target| target.shell_env.starts_with(super::super::params::PARAM_ENV_PREFIX))
        );
        let refuse = |reason| super::InputError::SecretLike {
            param: declaration.name.clone(),
            reason,
        };
        let mut keys = declaration.bindings.iter().map(|target| {
            target.shell_env.strip_prefix(super::super::params::PARAM_ENV_PREFIX).unwrap_or(&target.shell_env)
        });
        if is_secret_identifier(&declaration.name) || keys.any(is_secret_identifier) {
            return Err(refuse(super::SecretReason::Name));
        }
        if let Some(super::super::params::ParamValue::Text(value)) = supplied.get(&declaration.name)
            && is_secret_value(value)
        {
            return Err(refuse(super::SecretReason::Value));
        }
    }
    Ok(())
}

/// Whether an identifier has a secret word, or a qualifier followed by `key`, as `_`-separated
/// words, ignoring ASCII case.
pub fn is_secret_identifier(identifier: &str) -> bool {
    let mut previous: Option<&str> = None;
    for word in identifier.split('_') {
        let is_secret_word = SECRET_WORDS.iter().any(|secret| word.eq_ignore_ascii_case(secret));
        let is_qualified_key = word.eq_ignore_ascii_case("key")
            && previous
                .is_some_and(|qualifier| KEY_QUALIFIERS.iter().any(|known| qualifier.eq_ignore_ascii_case(known)));
        if is_secret_word || is_qualified_key {
            return true;
        }
        previous = Some(word);
    }
    false
}

/// Whether a value has a provider-token component or a token-shaped run.
pub fn is_secret_value(value: &str) -> bool {
    value.split('/').any(|component| is_provider_token(component) || has_token_shaped_run(component))
}

fn is_provider_token(component: &str) -> bool {
    TOKEN_FORMATS.iter().any(|token| {
        component.strip_prefix(token.prefix).is_some_and(|suffix| {
            suffix.len() >= token.min_suffix_bytes && suffix.bytes().all(|byte| in_alphabet(token.alphabet, byte))
        })
    })
}

const fn in_alphabet(alphabet: TokenAlphabet, byte: u8) -> bool {
    match alphabet {
        TokenAlphabet::Alphanumeric => byte.is_ascii_alphanumeric(),
        TokenAlphabet::UpperAlphanumeric => byte.is_ascii_uppercase() || byte.is_ascii_digit(),
        TokenAlphabet::AlphanumericUnderscore => byte.is_ascii_alphanumeric() || byte == b'_',
        TokenAlphabet::AlphanumericDash => byte.is_ascii_alphanumeric() || byte == b'-',
        TokenAlphabet::Base64Url => byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'),
        TokenAlphabet::Base64UrlDot => byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'),
    }
}

fn has_token_shaped_run(component: &str) -> bool {
    component.split(['-', '_', '.']).any(|run| {
        let bytes = run.as_bytes();
        let has_mixed_classes = bytes.iter().any(u8::is_ascii_uppercase)
            && bytes.iter().any(u8::is_ascii_lowercase)
            && bytes.iter().any(u8::is_ascii_digit);
        bytes.len() >= TOKEN_RUN_MIN_BYTES && bytes.iter().all(u8::is_ascii_alphanumeric) && has_mixed_classes
    })
}

#[cfg(test)]
mod tests;
