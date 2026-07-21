#[derive(Clone, Copy)]
struct PatternMatchInput<'a> {
    kind: PatternMatchKind,
    pattern: &'a str,
    resource_label: &'a str,
}

#[derive(Clone, Copy)]
struct PatternText<'a> {
    pattern: &'a str,
    resource_label: &'a str,
}

struct RuleFields {
    match_kind: PatternMatchKind,
    action: super::super::GuardedAction,
    resource_pattern: super::Text,
    outcome: super::Outcome,
    source_identity: super::Text,
    trust_scope: super::TrustScope,
    trust_status: super::TrustStatus,
    precedence: u16,
}

fn rule_from_fields(fields: RuleFields) -> PatternRule {
    assert!(
        !fields.resource_pattern.is_empty(),
        "rule fields keep a resource pattern"
    );
    assert!(
        (super::MIN_PRECEDENCE..=super::MAX_PRECEDENCE).contains(&fields.precedence),
        "rule fields keep validated precedence"
    );
    let precedence_text = fields.precedence.to_string();
    let rule_hash = super::hash_parts(&[
        super::HASH_SCHEMA,
        fields.match_kind.as_str(),
        fields.action.as_str(),
        fields.resource_pattern.as_str(),
        fields.outcome.as_str(),
        fields.source_identity.as_str(),
        fields.trust_scope.as_str(),
        precedence_text.as_str(),
    ]);
    PatternRule {
        match_kind: fields.match_kind,
        action: fields.action,
        resource_pattern: fields.resource_pattern,
        outcome: fields.outcome,
        source_identity: fields.source_identity,
        trust_scope: fields.trust_scope,
        trust_status: fields.trust_status,
        precedence: fields.precedence,
        rule_hash,
    }
}

fn pattern_matches(input: PatternMatchInput<'_>) -> bool {
    assert!(!input.pattern.is_empty(), "pattern input is validated");
    assert!(
        input.resource_label.len() >= input.resource_label.chars().count(),
        "resource label byte length bounds chars"
    );
    let text = PatternText {
        pattern: input.pattern,
        resource_label: input.resource_label,
    };
    match input.kind {
        PatternMatchKind::Glob => glob_matches(text),
        PatternMatchKind::Regex => regex_like_matches(text),
    }
}

fn glob_matches(input: PatternText<'_>) -> bool {
    assert!(!input.pattern.is_empty(), "glob pattern is validated");
    if let Some(prefix) = input.pattern.strip_suffix(GLOB_WILDCARD) {
        return input.resource_label.starts_with(prefix);
    }
    input.pattern == input.resource_label
}

fn regex_like_matches(input: PatternText<'_>) -> bool {
    assert!(!input.pattern.is_empty(), "regex-like pattern is validated");
    if let Some(literal) = input
        .pattern
        .strip_prefix('^')
        .and_then(|value| value.strip_suffix('$'))
    {
        return input.resource_label == literal;
    }
    input.resource_label.contains(input.pattern)
}

fn validate_pattern(kind: PatternMatchKind, pattern: &str) -> Result<(), super::Error> {
    if matches!(kind, PatternMatchKind::Regex) && regex_has_unbalanced_class(pattern) {
        return Err(super::Error::InvalidPattern {
            field: super::PATTERN_RESOURCE_FIELD.as_str(),
        });
    }
    Ok(())
}

fn regex_has_unbalanced_class(pattern: &str) -> bool {
    assert!(!pattern.is_empty(), "regex-like pattern is validated");
    assert!(
        pattern.len() >= pattern.chars().count(),
        "pattern byte length bounds chars"
    );
    let mut is_escaped = false;
    let mut is_class_open = false;
    for character in pattern.chars() {
        if is_escaped {
            is_escaped = false;
            continue;
        }
        if character == REGEX_ESCAPE {
            is_escaped = true;
            continue;
        }
        if character == REGEX_CLASS_START {
            if is_class_open {
                return true;
            }
            is_class_open = true;
            continue;
        }
        if character == REGEX_CLASS_END {
            if !is_class_open {
                return true;
            }
            is_class_open = false;
        }
    }
    is_class_open || is_escaped
}

fn match_kind_from_label(value: &str) -> Option<PatternMatchKind> {
    match value {
        GLOB_MATCH_LABEL => Some(PatternMatchKind::Glob),
        REGEX_MATCH_LABEL => Some(PatternMatchKind::Regex),
        _ => None,
    }
}
