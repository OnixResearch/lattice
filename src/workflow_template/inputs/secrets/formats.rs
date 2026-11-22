//! Known provider token formats and their matching. Pure.

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

/// What a provider token's suffix must contain beyond its alphabet, so kebab-case names that
/// merely start with a provider prefix are not tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuffixRequirement {
    None,
    /// At least this many ASCII digits: Slack tokens carry numeric team, app, or user ids.
    MinDigits(u8),
    /// At least one uppercase letter or digit: GitLab tokens are random base64url.
    UppercaseOrDigit,
}

impl SuffixRequirement {
    fn is_met(self, suffix: &str) -> bool {
        match self {
            Self::None => true,
            Self::MinDigits(minimum) => suffix.bytes().filter(u8::is_ascii_digit).count() >= usize::from(minimum),
            Self::UppercaseOrDigit => suffix.bytes().any(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit()),
        }
    }
}

/// One provider token format: its prefix, the minimum suffix length, the suffix alphabet, and
/// what the suffix must contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenFormat {
    pub prefix: &'static str,
    pub min_suffix_bytes: usize,
    pub alphabet: TokenAlphabet,
    pub requirement: SuffixRequirement,
}

const fn format(prefix: &'static str, min_suffix_bytes: usize, alphabet: TokenAlphabet) -> TokenFormat {
    requiring(prefix, min_suffix_bytes, alphabet, SuffixRequirement::None)
}

const fn requiring(
    prefix: &'static str,
    min_suffix_bytes: usize,
    alphabet: TokenAlphabet,
    requirement: SuffixRequirement,
) -> TokenFormat {
    TokenFormat {
        prefix,
        min_suffix_bytes,
        alphabet,
        requirement,
    }
}

/// Slack ids in `xapp-` and `xox?-` tokens add up to well over ten digits.
const SLACK_MIN_DIGITS: SuffixRequirement = SuffixRequirement::MinDigits(10);

pub const TOKEN_FORMATS: [TokenFormat; 27] = [
    format("ghp_", 30, TokenAlphabet::Alphanumeric),
    format("gho_", 30, TokenAlphabet::Alphanumeric),
    format("ghu_", 30, TokenAlphabet::Alphanumeric),
    format("ghs_", 30, TokenAlphabet::Alphanumeric),
    format("ghr_", 30, TokenAlphabet::Alphanumeric),
    format("github_pat_", 40, TokenAlphabet::AlphanumericUnderscore),
    requiring("glpat-", 20, TokenAlphabet::Base64UrlDot, SuffixRequirement::UppercaseOrDigit),
    requiring("xapp-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
    requiring("xoxa-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
    requiring("xoxb-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
    requiring("xoxp-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
    requiring("xoxr-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
    requiring("xoxs-", 20, TokenAlphabet::AlphanumericDash, SLACK_MIN_DIGITS),
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

/// Whether a component is a provider token. A format whose alphabet has no `.` also matches the
/// component's stem before its first `.`, so `hf_<token>.txt` is refused; a format whose
/// alphabet has `.` (`glpat-`, `ya29.`, `eyJ`) matches the whole component.
pub(super) fn is_provider_token(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component);
    TOKEN_FORMATS.iter().any(|token| {
        let candidate = if in_alphabet(token.alphabet, b'.') {
            component
        } else {
            stem
        };
        candidate.strip_prefix(token.prefix).is_some_and(|suffix| {
            suffix.len() >= token.min_suffix_bytes
                && suffix.bytes().all(|byte| in_alphabet(token.alphabet, byte))
                && token.requirement.is_met(suffix)
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
