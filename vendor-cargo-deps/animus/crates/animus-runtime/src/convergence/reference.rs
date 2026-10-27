const HASH_DOMAIN: &str = "animus.harness-convergence.handle.v1";
const HASH_SEPARATOR: &str = "|";
const HANDLE_MIN_CHARS: u32 = 1;
/// Maximum accepted handle token length.
pub const HANDLE_MAX_CHARS: u32 = 64;

/// Store namespace that owns one opaque handle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StoreKind {
    /// Tool-result store.
    Result,
    /// Session store.
    Session,
    /// Document store.
    Document,
}

impl StoreKind {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Result => "result",
            Self::Session => "session",
            Self::Document => "document",
        }
    }
}

/// Short opaque reference resolved only by its owning store.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Handle {
    kind: StoreKind,
    token: alloc::string::String,
    identity: alloc::string::String,
}

impl Handle {
    /// Validates a store-typed opaque token.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::InvalidHandle`] for blank, overlong, or unsafe tokens.
    pub fn new(kind: StoreKind, token: &str) -> Result<Self, super::Error> {
        let char_count =
            u32::try_from(token.chars().count()).map_err(|_| super::Error::InvalidHandle)?;
        if !(HANDLE_MIN_CHARS..=HANDLE_MAX_CHARS).contains(&char_count)
            || !token.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        {
            return Err(super::Error::InvalidHandle);
        }
        let identity = hash_parts(&[kind.label(), token]);
        Ok(Self {
            kind,
            token: alloc::string::String::from(token),
            identity,
        })
    }

    /// Returns the owning store namespace.
    #[must_use]
    pub const fn kind(&self) -> StoreKind {
        self.kind
    }

    /// Returns the exact opaque token.
    #[must_use]
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    /// Returns the stable BLAKE3 handle identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

fn hash_parts(parts: &[&str]) -> alloc::string::String {
    let mut framed = alloc::string::String::from(HASH_DOMAIN);
    for part in parts {
        framed.push_str(HASH_SEPARATOR);
        framed.push_str(part);
    }
    alloc::format!("{}", blake3::hash(framed.as_bytes()).to_hex())
}
