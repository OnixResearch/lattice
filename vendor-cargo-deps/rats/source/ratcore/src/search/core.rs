//! Fuzzy scoring algorithm with match position tracking.

const EMPTY_SCORE: i32 = 0;
const MINIMUM_MATCH_SCORE: i32 = 1;
const MATCHED_CHARACTER_SCORE: i32 = 10;
const CONSECUTIVE_MATCH_BONUS: i32 = 16;
const WORD_BOUNDARY_BONUS: i32 = 8;
const EXACT_CASE_BONUS: i32 = 2;
const PREFIX_BONUS: i32 = 15;
const GAP_PENALTY_PER_CHARACTER: i32 = 3;
const TIGHT_MATCH_DIVISOR: i32 = 4;
const FIRST_POSITION: usize = 0;
const NEXT_POSITION_OFFSET: usize = 1;

fn i32_from_usize(value: usize) -> Option<i32> {
    i32::try_from(value).ok()
}

/// Input for [`fuzzy_score`].
#[derive(Clone, Copy)]
pub struct FuzzyScoreInput<'a> {
    /// Candidate text to score.
    pub text: &'a str,
    /// Query text to match against the candidate.
    pub query: &'a str,
}

/// Position of one candidate within the caller's candidate list.
///
/// A candidate-list position is a distinct role from a match offset inside a
/// single candidate's text (see [`MatchOffset`]); this wrapper keeps the two
/// from being silently exchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CandidateIndex(usize);

impl CandidateIndex {
    /// A text offset is never a candidate-list position.
    ///
    /// ```compile_fail
    /// use ratcore::fuzzy::{CandidateIndex, MatchOffset};
    /// let offset = MatchOffset::from_zero_based(3);
    /// let position: CandidateIndex = offset;
    /// ```
    /// Creates a candidate-list position from a zero-based backing value.
    #[must_use]
    pub const fn from_zero_based(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based backing value for checked candidate-list access.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A byte/character offset inside one matched candidate's text.
///
/// Offsets within the matched text are a distinct role from a candidate-list
/// position ([`CandidateIndex`]); highlighting a matched span must consume
/// only [`MatchOffset`] values.
///
/// Indexing the candidate list with a text offset must not compile.
///
/// ```compile_fail
/// use ratcore::fuzzy::{CandidateIndex, MatchOffset};
/// let offset = MatchOffset::from_zero_based(3);
/// let candidates = ["a", "b"];
/// let _ = candidates.get(offset);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatchOffset(usize);

impl MatchOffset {
    /// Creates a text offset from a zero-based backing value.
    #[must_use]
    pub const fn from_zero_based(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based backing value for checked text-position access.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Result of scoring a candidate against a query.
#[derive(Debug, Clone)]
pub struct ScoredMatch {
    /// Position of this candidate within the caller's candidate list, set by
    /// the caller after scoring.
    pub index: CandidateIndex,
    /// Score where higher values represent better matches.
    pub score: i32,
    /// Character indices within this candidate's matched text.
    pub positions: Vec<MatchOffset>,
}

/// Scores a candidate `text` against `query`.
///
/// Rewards consecutive matches, word-boundary matches, and prefix matches.
/// Penalizes gaps between matches. Returns `None` when the query does not
/// match at all.
#[must_use]
pub fn fuzzy_score(input: FuzzyScoreInput<'_>) -> Option<ScoredMatch> {
    let FuzzyScoreInput { text, query } = input;
    if query.is_empty() {
        return Some(ScoredMatch {
            index: CandidateIndex::from_zero_based(0),
            score: EMPTY_SCORE,
            positions: vec![],
        });
    }

    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();
    let text_lower: Vec<char> = text.to_lowercase().chars().collect();
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();

    let mut positions = Vec::with_capacity(query_lower.len());
    let mut text_index = FIRST_POSITION;
    for &query_character in &query_lower {
        let mut is_match_found = false;
        while text_index < text_lower.len() {
            if text_lower[text_index] == query_character {
                positions.push(MatchOffset::from_zero_based(text_index));
                text_index = text_index.saturating_add(NEXT_POSITION_OFFSET);
                is_match_found = true;
                break;
            }
            text_index = text_index.saturating_add(NEXT_POSITION_OFFSET);
        }
        if !is_match_found {
            return None;
        }
    }

    debug_assert_eq!(positions.len(), query_lower.len());
    debug_assert!(positions.windows(2).all(|window| window[0] < window[1]));

    let mut score = EMPTY_SCORE;
    for (query_index, &position) in positions.iter().enumerate() {
        let position = position.get();
        score += MATCHED_CHARACTER_SCORE;

        if query_index > FIRST_POSITION {
            let previous_query_index = query_index.saturating_sub(NEXT_POSITION_OFFSET);
            let previous_position = positions[previous_query_index].get();
            if previous_position.saturating_add(NEXT_POSITION_OFFSET) == position {
                score += CONSECUTIVE_MATCH_BONUS;
            }
        }

        if position == FIRST_POSITION || !text_chars[position.saturating_sub(NEXT_POSITION_OFFSET)].is_alphanumeric() {
            score += WORD_BOUNDARY_BONUS;
        }

        if text_chars[position] == query_chars[query_index] {
            score += EXACT_CASE_BONUS;
        }

        if position == FIRST_POSITION && query_index == FIRST_POSITION {
            score += PREFIX_BONUS;
        }

        if query_index > FIRST_POSITION {
            let previous_query_index = query_index.saturating_sub(NEXT_POSITION_OFFSET);
            let previous_position = positions[previous_query_index].get();
            let gap = position.saturating_sub(previous_position).saturating_sub(NEXT_POSITION_OFFSET);
            score -= i32_from_usize(gap)? * GAP_PENALTY_PER_CHARACTER;
        }
    }

    let unmatched_characters = text_chars.len().saturating_sub(positions.len());
    score -= i32_from_usize(unmatched_characters)? / TIGHT_MATCH_DIVISOR;

    Some(ScoredMatch {
        index: CandidateIndex::from_zero_based(0),
        score: score.max(MINIMUM_MATCH_SCORE),
        positions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(input: (&str, &str)) -> Option<ScoredMatch> {
        let (text, query) = input;
        fuzzy_score(FuzzyScoreInput { text, query })
    }

    #[test]
    fn exact_prefix_highest() {
        let prefix = score(("Ship docs", "ship")).unwrap();
        let mid_word = score(("Worship plan", "ship")).unwrap();
        let suffix = score(("Flagship", "ship")).unwrap();
        assert!(prefix.score > mid_word.score, "prefix {} > mid-word {}", prefix.score, mid_word.score);
        assert!(prefix.score > suffix.score, "prefix {} > mid-word {}", prefix.score, suffix.score);
    }

    #[test]
    fn no_match_returns_none() {
        assert!(score(("hello", "xyz")).is_none());
    }

    #[test]
    fn empty_query_matches_all() {
        let scored_match = score(("anything", "")).unwrap();
        assert_eq!(scored_match.score, 0);
        assert!(scored_match.positions.is_empty());
    }

    #[test]
    fn consecutive_beats_scattered() {
        let consecutive = score(("abcdef", "abc")).unwrap();
        let scattered = score(("a_b_c_def", "abc")).unwrap();
        assert!(consecutive.score > scattered.score);
    }

    #[test]
    fn case_insensitive() {
        assert!(score(("Ship Docs", "sd")).is_some());
    }

    #[test]
    fn positions_tracked() {
        let scored_match = score(("Ship docs", "sd")).unwrap();
        assert_eq!(scored_match.positions.len(), 2);
        assert_eq!(scored_match.positions[0].get(), 0);
        assert_eq!(scored_match.positions[1].get(), 5);
    }

    #[test]
    fn candidate_index_is_set_by_caller_after_scoring() {
        let mut scored = score(("Ship docs", "ship")).unwrap();
        assert_eq!(scored.index.get(), 0);
        scored.index = CandidateIndex::from_zero_based(4);
        assert_eq!(scored.index.get(), 4);
    }

    #[test]
    fn ordering_consumes_score_not_position_role() {
        // Ordering must compare scores; a candidate-list position and a text
        // offset stay in different roles and cannot be swapped into ordering.
        let mut matches = vec![
            score(("Ship docs", "ship")).unwrap(),
            score(("Flagship", "ship")).unwrap(),
            score(("Worship plan", "ship")).unwrap(),
        ];
        let mut backing = matches.iter().map(|m| (m.score, m.index.get(), m.positions.clone())).collect::<Vec<_>>();
        backing.sort_by(|a, b| b.0.cmp(&a.0));
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        assert!(matches.iter().all(|m| m.positions.windows(2).all(|w| w[0] < w[1])));
        assert_eq!(
            matches.iter().map(|m| m.score).collect::<Vec<_>>(),
            backing.iter().map(|m| m.0).collect::<Vec<_>>(),
            "score order unchanged by role typing"
        );
    }

    #[test]
    fn offsets_route_highlighting_not_candidate_access() {
        let scored = score(("Ship docs", "sd")).unwrap();
        let label_chars: Vec<char> = "Ship docs".chars().collect();
        for MatchOffset_value in &scored.positions {
            assert!(MatchOffset_value.get() < label_chars.len());
        }
        // Rendering highlight consumes MatchOffset values.
        let highlighted: Vec<usize> = scored.positions.iter().map(|offset| offset.get()).collect();
        assert_eq!(highlighted, [0, 5]);
    }
}
