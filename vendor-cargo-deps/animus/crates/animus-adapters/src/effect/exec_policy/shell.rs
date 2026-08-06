const PROGRAM_SH: &str = "sh";
const PROGRAM_BASH: &str = "bash";
const FLAG_COMMAND: &str = "-c";
const FLAG_LOGIN_COMMAND: &str = "-lc";
const OPERATOR_AND: &str = "&&";
const OPERATOR_OR: &str = "||";
const OPERATOR_SEQUENCE: &str = ";";
const WRAPPER_ARGC: usize = 3;
const PROGRAM_INDEX: usize = 0;
const FLAG_INDEX: usize = 1;
const SOURCE_INDEX: usize = 2;
const MIN_CHAIN_COMMANDS: usize = 2;
const WORD_EXTRA_CHARS: [char; 10] = ['/', '.', '-', '_', ':', '=', '+', ',', '@', '%'];

const _: () = assert!(WRAPPER_ARGC > SOURCE_INDEX);
const _: () = assert!(MIN_CHAIN_COMMANDS > 0);

/// Classification returned by conservative wrapper analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisKind {
    /// The command was already an argv-like direct command.
    Direct,
    /// A simple wrapper was split into separate command intents.
    SimpleChain,
    /// A wrapper used unsupported syntax and was left opaque.
    OpaqueUnsupported,
}

/// Result of conservative wrapper analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Analysis {
    kind: AnalysisKind,
    intents: super::Collection<super::CommandIntent>,
}

impl Analysis {
    fn direct(intent: &super::CommandIntent) -> Self {
        Self {
            kind: AnalysisKind::Direct,
            intents: alloc::vec![intent.clone()],
        }
    }

    fn opaque(intent: &super::CommandIntent) -> Self {
        Self {
            kind: AnalysisKind::OpaqueUnsupported,
            intents: alloc::vec![intent.clone()],
        }
    }

    fn simple_chain(intents: super::Collection<super::CommandIntent>) -> Self {
        assert!(intents.len() >= MIN_CHAIN_COMMANDS, "simple chain must contain multiple intents");
        Self {
            kind: AnalysisKind::SimpleChain,
            intents,
        }
    }

    /// Returns the analysis classification.
    #[must_use]
    pub const fn kind(&self) -> AnalysisKind {
        self.kind
    }

    /// Returns the intents that must be evaluated by policy.
    #[must_use]
    pub fn intents(&self) -> &[super::CommandIntent] {
        self.intents.as_slice()
    }
}

/// Splits only narrow `sh -c` / `bash -lc` plain-word command chains.
#[must_use]
pub fn analyze_wrapper(intent: &super::CommandIntent) -> Analysis {
    if !is_supported_wrapper(intent.argv()) {
        return Analysis::direct(intent);
    }
    match parse_simple_chain(intent.argv()[SOURCE_INDEX].as_str()) {
        Some(intents) if intents.len() >= MIN_CHAIN_COMMANDS => Analysis::simple_chain(intents),
        Some(_intents) => Analysis::direct(intent),
        None => Analysis::opaque(intent),
    }
}

fn is_supported_wrapper(argv: &[super::Text]) -> bool {
    if argv.len() != WRAPPER_ARGC {
        return false;
    }
    let program = argv[PROGRAM_INDEX].as_str();
    let flag = argv[FLAG_INDEX].as_str();
    matches!(program, PROGRAM_SH | PROGRAM_BASH) && matches!(flag, FLAG_COMMAND | FLAG_LOGIN_COMMAND)
}

fn parse_simple_chain(source: &str) -> Option<super::Collection<super::CommandIntent>> {
    let token_count = source.split_whitespace().count();
    if token_count == 0 || token_count > super::MAX_COMMAND_WORDS {
        return None;
    }
    assert!(token_count > 0, "simple chain must contain at least one token");
    assert!(token_count <= super::MAX_COMMAND_WORDS, "simple chain token count must fit command bound");
    let mut parsed = super::Collection::with_capacity(token_count);
    let mut current = super::Collection::with_capacity(token_count);
    for token in source.split_whitespace() {
        if is_chain_operator(token) {
            if current.is_empty() {
                return None;
            }
            parsed.push(command_from_words(core::mem::take(&mut current)));
        } else if is_plain_word(token) {
            current.push(super::Text::from(token));
        } else {
            return None;
        }
    }
    if current.is_empty() {
        return None;
    }
    parsed.push(command_from_words(current));
    Some(parsed)
}

fn command_from_words(words: super::Collection<super::Text>) -> super::CommandIntent {
    assert!(!words.is_empty(), "parsed command must not be empty");
    super::CommandIntent::from_valid_words(words)
}

fn is_chain_operator(token: &str) -> bool {
    matches!(token, OPERATOR_AND | OPERATOR_OR | OPERATOR_SEQUENCE)
}

fn is_plain_word(token: &str) -> bool {
    !token.is_empty()
        && token.chars().all(|character| character.is_ascii_alphanumeric() || is_extra_word_char(character))
}

fn is_extra_word_char(character: char) -> bool {
    WORD_EXTRA_CHARS.contains(&character)
}
