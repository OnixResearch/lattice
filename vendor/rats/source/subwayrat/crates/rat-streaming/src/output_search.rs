//! Search within message output — Ctrl+F / f incremental search
//!
//! Provides an overlay search bar with match highlighting and
//! navigation between matches. Supports both exact substring
//! (case-insensitive, smart-case) and fuzzy (subsequence) matching.

/// A single match location in the output
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub row: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Search mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Case-insensitive substring match (smart-case: case-sensitive when query has uppercase)
    Substring,
    /// Fuzzy subsequence match (characters in order, not necessarily contiguous)
    Fuzzy,
}

/// Interactive search widget state
pub struct OutputSearch {
    /// Whether the search overlay is visible and capturing input
    pub active: bool,
    /// Current search query
    pub query: String,
    /// All match locations (sorted by row, then byte_start)
    pub matches: Vec<SearchMatch>,
    /// Index of the currently highlighted match
    pub current: usize,
    /// Search mode
    pub mode: SearchMode,
    /// Whether the render should scroll to the current match
    pub scroll_to_current: bool,
}

impl Default for OutputSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputSearch {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new(),
            matches: Vec::new(),
            current: 0,
            mode: SearchMode::Substring,
            scroll_to_current: false,
        }
    }

    /// Activate the search overlay
    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.matches.clear();
        self.current = 0;
        self.scroll_to_current = false;
    }

    /// Close the search overlay but keep matches for n/N navigation
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Close and clear everything
    pub fn cancel(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
        self.current = 0;
        self.scroll_to_current = false;
    }

    /// Type a character into the search query
    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Delete last character from the query
    pub fn backspace(&mut self) {
        self.query.pop();
    }

    /// Toggle between substring and fuzzy mode
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::Substring => SearchMode::Fuzzy,
            SearchMode::Fuzzy => SearchMode::Substring,
        };
    }

    /// Navigate to next match
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    /// Navigate to previous match
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = if self.current == 0 {
                self.matches.len() - 1
            } else {
                self.current - 1
            };
        }
    }

    /// Get the row of the current match (for scrolling)
    pub fn current_match_row(&self) -> Option<usize> {
        self.matches.get(self.current).map(|m| m.row)
    }

    /// Returns true if there's an active query with results
    pub fn has_matches(&self) -> bool {
        !self.query.is_empty() && !self.matches.is_empty()
    }

    /// Returns true if there's a non-empty query (even without matches)
    pub fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    /// Recompute matches against the given plain text lines.
    /// Tries to keep the current match near its previous position.
    pub fn update_matches(&mut self, plain_lines: &[String]) {
        let prev_row = self.matches.get(self.current).map(|m| m.row);

        self.matches.clear();

        if self.query.is_empty() {
            self.current = 0;
            return;
        }

        match self.mode {
            SearchMode::Substring => self.find_substring_matches(plain_lines),
            SearchMode::Fuzzy => self.find_fuzzy_matches(plain_lines),
        }

        if self.matches.is_empty() {
            self.current = 0;
        } else if let Some(prev) = prev_row {
            // Keep current match near the previous position
            self.current = self
                .matches
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| (m.row as isize - prev as isize).unsigned_abs())
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else {
            self.current = 0;
        }
    }

    fn find_substring_matches(&mut self, plain_lines: &[String]) {
        // Smart-case: case-insensitive unless query contains uppercase
        let case_sensitive = self.query.chars().any(|c| c.is_uppercase());
        let query = if case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        for (row, line) in plain_lines.iter().enumerate() {
            let haystack = if case_sensitive {
                line.clone()
            } else {
                line.to_lowercase()
            };

            let mut search_start = 0;
            while let Some(pos) = haystack[search_start..].find(&query) {
                let byte_start = search_start + pos;
                let byte_end = byte_start + query.len();
                self.matches.push(SearchMatch {
                    row,
                    byte_start,
                    byte_end,
                });
                search_start = byte_start + 1;
                if search_start >= haystack.len() {
                    break;
                }
            }
        }
    }

    fn find_fuzzy_matches(&mut self, plain_lines: &[String]) {
        let query_chars: Vec<char> = self.query.to_lowercase().chars().collect();
        if query_chars.is_empty() {
            return;
        }

        for (row, line) in plain_lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            if let Some((byte_start, byte_end)) = fuzzy_match_range(&line_lower, &query_chars) {
                self.matches.push(SearchMatch {
                    row,
                    byte_start,
                    byte_end,
                });
            }
        }
    }
}

/// Find the byte range of a fuzzy (subsequence) match.
/// Returns (start_byte, end_byte) covering from the first to last matched character.
fn fuzzy_match_range(haystack: &str, needle_chars: &[char]) -> Option<(usize, usize)> {
    if needle_chars.is_empty() {
        return None;
    }

    let mut ni = 0; // needle index
    let mut first_byte = None;
    let mut last_byte_end;

    for (byte_idx, ch) in haystack.char_indices() {
        if ch == needle_chars[ni] {
            if first_byte.is_none() {
                first_byte = Some(byte_idx);
            }
            last_byte_end = byte_idx + ch.len_utf8();
            ni += 1;
            if ni == needle_chars.len() {
                // Safe: first_byte is set when ni first matches (before incrementing)
                return Some((first_byte.expect("set on first match"), last_byte_end));
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring_search() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "hello".to_string();

        let lines = vec![
            "  hello world".to_string(),
            "  foo bar".to_string(),
            "  say hello again".to_string(),
        ];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 2);
        assert_eq!(search.matches[0].row, 0);
        assert_eq!(search.matches[1].row, 2);
    }

    #[test]
    fn test_case_insensitive() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "hello".to_string();

        let lines = vec!["HELLO World".to_string()];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 1);
    }

    #[test]
    fn test_smart_case() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "Hello".to_string();

        let lines = vec!["hello world".to_string(), "Hello world".to_string()];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.matches[0].row, 1);
    }

    #[test]
    fn test_fuzzy_match() {
        let result = fuzzy_match_range("hello world", &['h', 'w']);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert_eq!(&"hello world"[start..end], "hello w");
    }

    #[test]
    fn test_fuzzy_no_match() {
        let result = fuzzy_match_range("hello", &['z']);
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_search_mode() {
        let mut search = OutputSearch::new();
        search.activate();
        search.mode = SearchMode::Fuzzy;
        search.query = "hw".to_string();

        let lines = vec!["hello world".to_string(), "goodbye".to_string()];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.matches[0].row, 0);
    }

    #[test]
    fn test_navigation() {
        let mut search = OutputSearch::new();
        search.matches = vec![
            SearchMatch {
                row: 0,
                byte_start: 0,
                byte_end: 5,
            },
            SearchMatch {
                row: 2,
                byte_start: 0,
                byte_end: 5,
            },
            SearchMatch {
                row: 4,
                byte_start: 0,
                byte_end: 5,
            },
        ];
        search.current = 0;

        search.next_match();
        assert_eq!(search.current, 1);
        search.next_match();
        assert_eq!(search.current, 2);
        search.next_match(); // wraps
        assert_eq!(search.current, 0);

        search.prev_match(); // wraps back
        assert_eq!(search.current, 2);
        search.prev_match();
        assert_eq!(search.current, 1);
    }

    #[test]
    fn test_toggle_mode() {
        let mut search = OutputSearch::new();
        assert_eq!(search.mode, SearchMode::Substring);
        search.toggle_mode();
        assert_eq!(search.mode, SearchMode::Fuzzy);
        search.toggle_mode();
        assert_eq!(search.mode, SearchMode::Substring);
    }

    #[test]
    fn test_multiple_matches_same_line() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "ab".to_string();

        let lines = vec!["ab cd ab ef ab".to_string()];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 3);
    }

    #[test]
    fn test_keeps_position_on_update() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "x".to_string();

        let lines = vec![
            "a x b".to_string(),
            "c d e".to_string(),
            "f x g".to_string(),
            "h i j".to_string(),
            "k x l".to_string(),
        ];
        search.update_matches(&lines);
        assert_eq!(search.matches.len(), 3);

        // Navigate to the last match (row 4)
        search.current = 2;
        assert_eq!(search.matches[search.current].row, 4);

        // Update with same content — should stay near row 4
        search.update_matches(&lines);
        assert_eq!(search.matches[search.current].row, 4);
    }

    #[test]
    fn test_cancel_clears_all() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "test".to_string();
        search.matches.push(SearchMatch {
            row: 0,
            byte_start: 0,
            byte_end: 4,
        });
        search.current = 0;

        search.cancel();
        assert!(!search.active);
        assert!(search.query.is_empty());
        assert!(search.matches.is_empty());
    }

    #[test]
    fn test_deactivate_keeps_matches() {
        let mut search = OutputSearch::new();
        search.activate();
        search.query = "test".to_string();
        search.matches.push(SearchMatch {
            row: 0,
            byte_start: 0,
            byte_end: 4,
        });

        search.deactivate();
        assert!(!search.active);
        assert!(search.has_matches());
    }
}
