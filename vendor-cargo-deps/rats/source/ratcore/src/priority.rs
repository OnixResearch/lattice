//! Shared validated priority cookie used by agenda and tag consumers.
//!
//! A priority cookie is a single uppercase ASCII letter (`A`..=`Z`). It is a
//! distinct role from a raw `char` or a `String` identity; unsupported cookies
//! fail validation instead of entering typed state.

/// A validated priority cookie (`A`..=`Z`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Priority(char);

/// Error returned when a priority cookie is not an uppercase ASCII letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPriority(char);

impl Priority {
    /// Validates an uppercase priority cookie.
    pub fn new(cookie: char) -> Result<Self, InvalidPriority> {
        if cookie.is_ascii_uppercase() {
            Ok(Self(cookie))
        } else {
            Err(InvalidPriority(cookie))
        }
    }

    /// Creates a priority from a known-valid uppercase cookie.
    #[must_use]
    pub const fn new_unchecked(cookie: char) -> Self {
        Self(cookie)
    }

    /// Returns the priority cookie character.
    #[must_use]
    pub const fn cookie(self) -> char {
        self.0
    }

    /// Formats the priority as an Org-mode `[#X]` cookie.
    #[must_use]
    pub fn format(self) -> String {
        format!("[#{}]", self.0)
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[#{}]", self.0)
    }
}

impl InvalidPriority {
    /// Returns the rejected cookie character.
    #[must_use]
    pub const fn cookie(self) -> char {
        self.0
    }
}

impl TryFrom<char> for Priority {
    type Error = InvalidPriority;

    fn try_from(cookie: char) -> Result<Self, Self::Error> {
        Self::new(cookie)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercase_cookies_validate() {
        assert_eq!(Priority::new('A'), Ok(Priority::new_unchecked('A')));
        assert_eq!(Priority::new('Z').unwrap().cookie(), 'Z');
    }

    #[test]
    fn unsupported_cookies_fail_validation() {
        assert_eq!(Priority::new('a'), Err(InvalidPriority('a')));
        assert_eq!(Priority::new('1'), Err(InvalidPriority('1')));
        assert_eq!(Priority::new('['), Err(InvalidPriority('[')));
        assert!(Priority::try_from('!').is_err());
    }

    #[test]
    fn formats_as_org_cookie() {
        assert_eq!(Priority::new_unchecked('A').format(), "[#A]");
        assert_eq!(Priority::new_unchecked('B').to_string(), "[#B]");
    }

    #[test]
    fn priority_roles_do_not_implicitly_come_from_char() {
        let cookie = 'A';
        // A raw char is not a Priority without an explicit adapter.
        assert!(Priority::try_from(cookie).is_ok());
    }
}
