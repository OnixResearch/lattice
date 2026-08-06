const PREFIX: &str = "#!animus ";
const MODE_KEY: &str = "mode";
const CHAIN_KEY: &str = "chain";
const KEY_VALUE_SEPARATOR: char = '=';
const LINE_SEPARATOR: char = '\n';

/// Directive parser issue code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// Directive token did not use `key=value` syntax.
    MalformedToken,
    /// Directive key is not supported.
    UnsupportedKey,
    /// Mode value is not supported.
    UnknownMode,
    /// Chain target is syntactically valid but not supported by policy.
    UnsupportedChainTarget,
    /// An exclusive directive key appeared more than once.
    DuplicateDirective,
}

/// Diagnostic from directive parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    issue: Issue,
    token: super::support::Text,
}

impl Diagnostic {
    /// Returns the directive issue code.
    #[must_use]
    pub const fn issue(&self) -> Issue {
        self.issue
    }

    /// Returns the bounded directive token or value that failed.
    #[must_use]
    pub fn token(&self) -> &str {
        self.token.as_str()
    }
}

/// Result of parsing profile body directives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    visible_body: super::support::Text,
    mode: Option<super::types::Mode>,
    chain_target: Option<super::types::Mode>,
    diagnostics: super::support::Collection<Diagnostic>,
}

impl Report {
    /// Returns profile body text with directive syntax removed.
    #[must_use]
    pub fn visible_body(&self) -> &str {
        self.visible_body.as_str()
    }

    /// Returns the requested mode when parsing succeeded.
    #[must_use]
    pub const fn mode(&self) -> Option<super::types::Mode> {
        self.mode
    }

    /// Returns the requested chain target when parsing succeeded.
    #[must_use]
    pub const fn chain_target(&self) -> Option<super::types::Mode> {
        self.chain_target
    }

    /// Returns directive diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns whether directive parsing failed closed.
    #[must_use]
    pub fn has_blocking_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Parses first-line profile directives and strips them from body text.
#[must_use]
pub fn parse(body: &str, supported_chain_targets: &[super::types::Mode]) -> Report {
    assert!(supported_chain_targets.len() <= super::support::MAX_CHAIN_TARGETS);
    assert!(!PREFIX.is_empty());
    let (directive_text, visible_body) = split_directive(body);
    let Some(directive) = directive_text else {
        return Report {
            visible_body,
            mode: None,
            chain_target: None,
            diagnostics: super::support::Collection::new(),
        };
    };

    parse_tokens(directive, visible_body, supported_chain_targets)
}

fn parse_tokens(
    directive: &str,
    visible_body: super::support::Text,
    supported_chain_targets: &[super::types::Mode],
) -> Report {
    let mut mode = None;
    let mut chain_target = None;
    let mut diagnostics = super::support::Collection::with_capacity(super::support::MAX_DIRECTIVE_TOKENS);

    for token in directive.split_whitespace().take(super::support::MAX_DIRECTIVE_TOKENS) {
        if let Some(diagnostic) = parse_token(token, supported_chain_targets, &mut mode, &mut chain_target) {
            diagnostics.push(diagnostic);
        }
    }

    finalize_report(visible_body, mode, chain_target, diagnostics)
}

fn finalize_report(
    visible_body: super::support::Text,
    mode: Option<super::types::Mode>,
    chain_target: Option<super::types::Mode>,
    diagnostics: super::support::Collection<Diagnostic>,
) -> Report {
    if diagnostics.is_empty() {
        Report {
            visible_body,
            mode,
            chain_target,
            diagnostics,
        }
    } else {
        Report {
            visible_body,
            mode: None,
            chain_target: None,
            diagnostics,
        }
    }
}

fn split_directive(body: &str) -> (Option<&str>, super::support::Text) {
    let Some(first_line_end) = body.find(LINE_SEPARATOR) else {
        return match body.strip_prefix(PREFIX) {
            Some(directive) => (Some(directive), super::support::Text::new()),
            None => (None, super::support::Text::from(body.trim())),
        };
    };

    let (first_line, rest) = body.split_at(first_line_end);
    match first_line.strip_prefix(PREFIX) {
        Some(directive) => {
            (Some(directive), super::support::Text::from(rest.trim_start_matches(LINE_SEPARATOR).trim()))
        }
        None => (None, super::support::Text::from(body.trim())),
    }
}

fn parse_token(
    token: &str,
    supported_chain_targets: &[super::types::Mode],
    mode: &mut Option<super::types::Mode>,
    chain_target: &mut Option<super::types::Mode>,
) -> Option<Diagnostic> {
    let Some((key, value)) = token.split_once(KEY_VALUE_SEPARATOR) else {
        return Some(diagnostic(Issue::MalformedToken, token));
    };

    match key {
        MODE_KEY => parse_mode(value, mode),
        CHAIN_KEY => parse_chain(value, supported_chain_targets, chain_target),
        _ => Some(diagnostic(Issue::UnsupportedKey, key)),
    }
}

fn parse_mode(value: &str, mode: &mut Option<super::types::Mode>) -> Option<Diagnostic> {
    if mode.is_some() {
        return Some(diagnostic(Issue::DuplicateDirective, MODE_KEY));
    }

    match super::support::parse_mode(value) {
        Some(parsed) => {
            *mode = Some(parsed);
            None
        }
        None => Some(diagnostic(Issue::UnknownMode, value)),
    }
}

fn parse_chain(
    value: &str,
    supported_chain_targets: &[super::types::Mode],
    chain_target: &mut Option<super::types::Mode>,
) -> Option<Diagnostic> {
    assert!(supported_chain_targets.len() <= super::support::MAX_CHAIN_TARGETS);
    assert!(!CHAIN_KEY.is_empty());
    if chain_target.is_some() {
        return Some(diagnostic(Issue::DuplicateDirective, CHAIN_KEY));
    }

    match super::support::parse_mode(value) {
        Some(parsed) if supported_chain_targets.contains(&parsed) => {
            *chain_target = Some(parsed);
            None
        }
        Some(_) => Some(diagnostic(Issue::UnsupportedChainTarget, value)),
        None => Some(diagnostic(Issue::UnknownMode, value)),
    }
}

fn diagnostic(issue: Issue, token: &str) -> Diagnostic {
    Diagnostic {
        issue,
        token: super::support::Text::from(token.trim()),
    }
}
