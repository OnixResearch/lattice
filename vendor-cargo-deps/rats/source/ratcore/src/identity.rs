//! Internal helpers for declaring public semantic identifier types.

/// Declares a role-only, string-backed semantic identifier.
///
/// Role-only identifiers intentionally accept empty and otherwise malformed text so
/// raw application records remain diagnosable by their owning domain validators.
/// The wrapper provides no implicit conversion to another identity namespace.
macro_rules! semantic_string_id {
    ($(#[$meta:meta])* $visibility:vis struct $name:ident;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $visibility struct $name(String);

        impl $name {
            /// Creates an identifier from its stable textual representation.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the stable textual representation.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Returns true when the raw textual representation is empty.
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Consumes the identifier and returns its textual representation.
            #[must_use]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                Self(value.clone())
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}
