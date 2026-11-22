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
//! - a supplied `relative_path` value with a `/`-separated component, or its stem before the first
//!   `.`, that is a known provider token: its prefix followed by at least that format's minimum
//!   suffix drawn only from that format's alphabet (see [`formats::TOKEN_FORMATS`]);
//! - a supplied `relative_path` value with a `.`-separated piece that, without its `-` and `_`, has
//!   at least [`TOKEN_RUN_MIN_BYTES`] letters and digits, mixes uppercase, lowercase and digits,
//!   and has lowercase runs averaging under [`WORD_RUN_MEAN_BYTES`] bytes.
//!
//! A binding into a credential-bound key cannot occur: every target key starts with
//! `LATTICE_PARAM_`, and #27's `validate_targets` refuses any node credential key with that
//! prefix at load.

mod formats;

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

/// A random-looking piece has at least this many letters and digits.
///
/// Base62 and base64url keys are 20 to 100 or more characters, so 20 covers the shortest
/// provider tokens.
pub const TOKEN_RUN_MIN_BYTES: usize = 20;

/// Lowercase runs averaging at least this many bytes are words, not random characters.
///
/// Identifier words make lowercase runs of three or more bytes (`OAuth2AuthorizationServerConfig`
/// averages 6.25, `SHA256DigestComputation` 7.5). In a uniform base62 or base64url string a
/// lowercase byte is followed by another with probability 26/62, so lowercase runs average
/// about 1.7 bytes. Measured on 40,000 uniform base64url strings each, the rule refuses about
/// 98.0% at 32 characters, 99.2% at 43 and 99.8% at 64. A CamelCase name made mostly of
/// one- and two-letter words with digits can still match.
pub const WORD_RUN_MEAN_BYTES: usize = 3;

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
    value
        .split('/')
        .any(|component| formats::is_provider_token(component) || has_token_shaped_run(component))
}

/// Whether a `.`-separated piece, read without its `-` and `_`, is a random-looking run.
///
/// Base64url secrets put `-` and `_` anywhere, so those separators are dropped rather than split
/// on; `.` still separates an extension. A piece is token-shaped when it has at least
/// [`TOKEN_RUN_MIN_BYTES`] letters and digits, mixes uppercase, lowercase and digits, and its
/// lowercase runs average under [`WORD_RUN_MEAN_BYTES`] bytes.
fn has_token_shaped_run(component: &str) -> bool {
    component.split('.').any(|piece| {
        let mut shape = RunShape::default();
        let is_alphanumeric = piece.bytes().filter(|byte| !matches!(byte, b'-' | b'_')).all(|byte| shape.push(byte));
        is_alphanumeric && shape.is_token_shaped()
    })
}

/// Byte-class counts of one piece, read in order.
#[derive(Default)]
struct RunShape {
    length_bytes: usize,
    uppercase_bytes: usize,
    lowercase_bytes: usize,
    digit_bytes: usize,
    lowercase_runs: usize,
    is_after_lowercase: bool,
}

impl RunShape {
    /// Count one byte; `false` for a byte outside letters and digits.
    fn push(&mut self, byte: u8) -> bool {
        let is_lowercase = byte.is_ascii_lowercase();
        if is_lowercase && !self.is_after_lowercase {
            self.lowercase_runs = self.lowercase_runs.saturating_add(1);
        }
        self.is_after_lowercase = is_lowercase;
        self.length_bytes = self.length_bytes.saturating_add(1);
        let class = if byte.is_ascii_uppercase() {
            &mut self.uppercase_bytes
        } else if is_lowercase {
            &mut self.lowercase_bytes
        } else if byte.is_ascii_digit() {
            &mut self.digit_bytes
        } else {
            return false;
        };
        *class = class.saturating_add(1);
        true
    }

    /// Words in identifiers make lowercase runs of three or more bytes on average (`Builder`,
    /// `Config`, `Sha`); a random base62 or base64url run averages about 1.7.
    fn is_token_shaped(&self) -> bool {
        let has_mixed_classes = self.uppercase_bytes > 0 && self.lowercase_bytes > 0 && self.digit_bytes > 0;
        let is_word_free = self.lowercase_bytes < self.lowercase_runs.saturating_mul(WORD_RUN_MEAN_BYTES);
        debug_assert!(self.lowercase_runs <= self.lowercase_bytes);
        self.length_bytes >= TOKEN_RUN_MIN_BYTES && has_mixed_classes && is_word_free
    }
}

#[cfg(test)]
mod tests;
