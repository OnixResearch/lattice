/// Borrowed argv-like command intent before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandIntentDraft<'a> {
    /// Command words exactly as selected by the caller.
    pub argv: &'a [&'a str],
}

/// Validated argv-like command intent with redacted fingerprint evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIntent {
    argv: super::super::Collection<super::super::Text>,
    fingerprint: super::super::Text,
}

impl CommandIntent {
    /// Validates a command intent.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::Error`] when argv is empty, too large, or
    /// contains an invalid adapter-boundary field.
    pub fn new(draft: &CommandIntentDraft<'_>) -> Result<Self, super::super::Error> {
        let argv = validate_words(super::super::COMMAND_ARG_FIELD, draft.argv)?;
        Ok(Self::from_valid_words(argv))
    }

    pub(in crate::effect::exec_policy) fn from_valid_words(
        argv: super::super::Collection<super::super::Text>,
    ) -> Self {
        assert!(!argv.is_empty(), "validated command argv must not be empty");
        assert!(
            argv.len() <= super::super::MAX_COMMAND_WORDS,
            "validated command argv must fit policy bound"
        );
        let mut refs = alloc::vec::Vec::with_capacity(argv.len());
        for word in &argv {
            refs.push(word.as_str());
        }
        let fingerprint =
            super::super::hash_parts(super::super::COMMAND_HASH_KIND, refs.as_slice());
        Self { argv, fingerprint }
    }

    /// Returns validated argv words.
    #[must_use]
    pub fn argv(&self) -> &[super::super::Text] {
        self.argv.as_slice()
    }

    /// Returns the BLAKE3 command fingerprint used for evidence.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        self.fingerprint.as_str()
    }

    /// Returns the executable or wrapper word.
    #[must_use]
    pub fn program(&self) -> &str {
        self.argv.first().map_or(
            super::super::NO_MATCHING_RULE_ID,
            super::super::Text::as_str,
        )
    }
}

pub(super) fn validate_words(
    field: crate::validation::FieldName,
    words: &[&str],
) -> Result<super::super::Collection<super::super::Text>, super::super::Error> {
    assert!(!field.as_str().is_empty(), "field name must not be empty");
    if words.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field.as_str(),
        }
        .into());
    }
    if words.len() > super::super::MAX_COMMAND_WORDS {
        return Err(super::super::Error::TooManyItems {
            field: field.as_str(),
            max_items: super::super::MAX_COMMAND_WORDS,
            actual_items: words.len(),
        });
    }
    assert!(
        words.len() <= super::super::MAX_COMMAND_WORDS,
        "command word bound is enforced"
    );
    let mut normalized = super::super::Collection::with_capacity(words.len());
    for word in words {
        normalized.push(super::super::normalized_text(field, word)?);
    }
    Ok(normalized)
}
