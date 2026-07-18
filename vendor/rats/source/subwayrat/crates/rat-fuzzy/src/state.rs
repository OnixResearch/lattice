//! Fuzzy finder state.

use std::cmp::Reverse;

use crate::score::{FuzzyScoreInput, ScoredMatch, fuzzy_score};
use crate::types::{FuzzyCandidate, FuzzySource};

pub struct FuzzyState {
    pub query: String,
    pub results: Vec<ScoredMatch>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub open: bool,
    result: Option<FuzzyCandidate>,
}

impl FuzzyState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            open: false,
            result: None,
        }
    }

    /// Re-score all candidates against the current query.
    pub fn update_results(&mut self, source: &dyn FuzzySource) {
        let candidates = source.candidates();
        if self.query.is_empty() {
            self.results = candidates
                .iter()
                .enumerate()
                .map(|(i, _)| ScoredMatch {
                    index: i,
                    score: 0,
                    positions: vec![],
                })
                .collect();
        } else {
            let mut scored: Vec<ScoredMatch> = candidates
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    fuzzy_score(FuzzyScoreInput {
                        text: &c.text,
                        query: &self.query,
                    })
                    .map(|mut m| {
                        m.index = i;
                        m
                    })
                })
                .collect();
            scored.sort_by_key(|scored_match| Reverse(scored_match.score));
            self.results = scored;
        }
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Take the selection result (resets to None after taking).
    pub fn take_result(&mut self) -> Option<FuzzyCandidate> {
        self.result.take()
    }

    /// Confirm: store the selected candidate as the result and close.
    pub fn confirm(&mut self, source: &dyn FuzzySource) {
        if let Some(scored) = self.results.get(self.selected) {
            let candidates = source.candidates();
            if scored.index < candidates.len() {
                self.result = Some(candidates[scored.index].clone());
            }
        }
        self.open = false;
    }
}

impl Default for FuzzyState {
    fn default() -> Self {
        Self::new()
    }
}
