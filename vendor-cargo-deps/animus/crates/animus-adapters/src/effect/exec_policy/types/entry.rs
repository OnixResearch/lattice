/// Borrowed exec-policy rule before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleDraft<'a> {
    /// Stable rule identity.
    pub id: Option<&'a str>,
    /// Decision label: allow, prompt, or forbid.
    pub decision: Option<&'a str>,
    /// Safe rationale label or short sentence.
    pub rationale: Option<&'a str>,
    /// Command prefix matched word-for-word against argv.
    pub command_prefix: &'a [&'a str],
    /// Examples that must match the command prefix.
    pub positive_examples: &'a [super::CommandIntentDraft<'a>],
    /// Examples that must not match the command prefix.
    pub negative_examples: &'a [super::CommandIntentDraft<'a>],
}

/// Validated exec-policy rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    id: super::super::Text,
    decision: super::Decision,
    rationale: super::super::Text,
    command_prefix: super::super::Collection<super::super::Text>,
    hash: super::super::Text,
}

impl Rule {
    /// Validates an exec-policy rule and its positive/negative examples.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::Error`] when the rule is malformed or an example
    /// does not prove the intended prefix boundary.
    pub fn validate(draft: &RuleDraft<'_>) -> Result<Self, super::super::Error> {
        let id = super::super::required_text(super::super::RULE_ID_FIELD, draft.id)?;
        let decision = parse_decision(super::super::required_text(
            super::super::DECISION_FIELD,
            draft.decision,
        )?)?;
        let rationale =
            super::super::required_text(super::super::RATIONALE_FIELD, draft.rationale)?;
        let command_prefix =
            super::argv::validate_words(super::super::PREFIX_FIELD, draft.command_prefix)?;
        validate_examples_len(
            super::super::POSITIVE_EXAMPLES_FIELD,
            draft.positive_examples.len(),
        )?;
        validate_examples_len(
            super::super::NEGATIVE_EXAMPLES_FIELD,
            draft.negative_examples.len(),
        )?;
        let rule_hash = identity(&id, decision, &rationale, command_prefix.as_slice());
        let rule = Self {
            id,
            decision,
            rationale,
            command_prefix,
            hash: rule_hash,
        };
        rule.validate_positive_examples(draft.positive_examples)?;
        rule.validate_negative_examples(draft.negative_examples)?;
        Ok(rule)
    }

    /// Returns the stable rule identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the rule decision.
    #[must_use]
    pub const fn decision(&self) -> super::Decision {
        self.decision
    }

    /// Returns the safe rule rationale.
    #[must_use]
    pub fn rationale(&self) -> &str {
        self.rationale.as_str()
    }

    /// Returns the command prefix.
    #[must_use]
    pub fn command_prefix(&self) -> &[super::super::Text] {
        self.command_prefix.as_slice()
    }

    /// Returns the BLAKE3 hash of validated rule fields.
    #[must_use]
    pub fn rule_hash(&self) -> &str {
        self.hash.as_str()
    }

    pub(in crate::effect::exec_policy) fn matches_intent(
        &self,
        intent: &super::CommandIntent,
    ) -> bool {
        prefix_matches(self.command_prefix.as_slice(), intent.argv())
    }

    fn validate_positive_examples(
        &self,
        examples: &[super::CommandIntentDraft<'_>],
    ) -> Result<(), super::super::Error> {
        for example in examples {
            let intent = super::CommandIntent::new(example)?;
            if !self.matches_intent(&intent) {
                return Err(super::super::Error::PositiveExampleMismatch {
                    rule_id: self.id.clone(),
                    command_fingerprint: super::super::Text::from(intent.fingerprint()),
                });
            }
        }
        Ok(())
    }

    fn validate_negative_examples(
        &self,
        examples: &[super::CommandIntentDraft<'_>],
    ) -> Result<(), super::super::Error> {
        for example in examples {
            let intent = super::CommandIntent::new(example)?;
            if self.matches_intent(&intent) {
                return Err(super::super::Error::NegativeExampleMatched {
                    rule_id: self.id.clone(),
                    command_fingerprint: super::super::Text::from(intent.fingerprint()),
                });
            }
        }
        Ok(())
    }
}

fn parse_decision(value: super::super::Text) -> Result<super::Decision, super::super::Error> {
    match value.as_str() {
        super::super::ALLOW_LABEL => Ok(super::Decision::Allow),
        super::super::PROMPT_LABEL => Ok(super::Decision::Prompt),
        super::super::FORBID_LABEL => Ok(super::Decision::Forbid),
        _ => Err(super::super::Error::UnsupportedDecision { value }),
    }
}

fn prefix_matches(prefix: &[super::super::Text], argv: &[super::super::Text]) -> bool {
    if prefix.len() > argv.len() {
        return false;
    }
    prefix
        .iter()
        .zip(argv.iter())
        .all(|(prefix_word, argv_word)| prefix_word == argv_word)
}

fn validate_examples_len(
    field: crate::validation::FieldName,
    len: usize,
) -> Result<(), super::super::Error> {
    if len == 0 {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field.as_str(),
        }
        .into());
    }
    if len > super::super::MAX_RULE_EXAMPLES {
        return Err(super::super::Error::TooManyItems {
            field: field.as_str(),
            max_items: super::super::MAX_RULE_EXAMPLES,
            actual_items: len,
        });
    }
    Ok(())
}

fn identity(
    id: &str,
    decision: super::Decision,
    rationale: &str,
    command_prefix: &[super::super::Text],
) -> super::super::Text {
    let fixed_parts: usize = 3;
    let total_parts = fixed_parts.saturating_add(command_prefix.len());
    let mut parts = alloc::vec::Vec::with_capacity(total_parts);
    parts.push(id);
    parts.push(decision.as_str());
    parts.push(rationale);
    for word in command_prefix {
        parts.push(word.as_str());
    }
    super::super::hash_parts(super::super::RULE_HASH_KIND, parts.as_slice())
}
