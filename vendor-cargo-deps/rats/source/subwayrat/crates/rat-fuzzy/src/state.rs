//! Fuzzy finder state projected from the shared prompt reducer.

use crate::types::FuzzyCandidate;
use crate::types::FuzzySource;

/// Terminal fuzzy state. Text cursor, scroll, drawing, and physical events remain local.
pub struct FuzzyState {
    /// Committed query projected from the shared prompt session.
    pub query: String,
    /// Filtered rows in shared deterministic order with terminal highlight metadata.
    pub results: Vec<crate::score::ScoredMatch>,
    /// Active projected row.
    pub selected: usize,
    /// Renderer-owned visible scroll offset.
    pub scroll_offset: usize,
    /// Whether the adapter is mounted and interactive.
    pub open: bool,
    result: Option<FuzzyCandidate>,
    prompt: ratcore::prompt_completion::PromptSession,
    generation: u64,
}

impl FuzzyState {
    /// Creates a closed terminal adapter with one valid shared prompt session.
    #[must_use]
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            open: false,
            result: None,
            prompt: crate::prompt::new_session(),
            generation: 0,
        }
    }

    /// Replaces caller-supplied candidates and synchronizes renderer projection.
    pub fn update_results(&mut self, source: &dyn FuzzySource) {
        self.synchronize_public_query();
        self.generation = self.generation.saturating_add(1);
        let Some(snapshot) = crate::prompt::snapshot(source, self.generation) else {
            return;
        };
        let transition = ratcore::prompt_completion::reduce_prompt(
            &self.prompt,
            ratcore::prompt_completion::PromptMessage::ReplaceCandidates(snapshot),
        );
        if transition.accepted {
            self.prompt = transition.session;
        }
        self.project(source);
    }

    /// Opens and resets against one immutable supplied candidate snapshot.
    pub(crate) fn open_with(&mut self, source: &dyn FuzzySource) {
        self.generation = self.generation.saturating_add(1);
        let Some(completion) = crate::prompt::snapshot(source, self.generation) else {
            return;
        };
        let transition =
            ratcore::prompt_completion::reduce_prompt(&self.prompt, ratcore::prompt_completion::PromptMessage::Reset {
                input: String::new(),
                completion: Some(completion),
            });
        if transition.accepted {
            self.prompt = transition.session;
            self.open = true;
            self.project(source);
        }
    }

    pub(crate) fn replace_query(&mut self, query: String, source: &dyn FuzzySource) {
        let transition = ratcore::prompt_completion::reduce_prompt(
            &self.prompt,
            ratcore::prompt_completion::PromptMessage::ReplaceInput {
                input: query,
                expected_revision: self.prompt.input_revision(),
            },
        );
        if transition.accepted {
            self.prompt = transition.session;
        }
        self.project(source);
    }

    pub(crate) fn move_active(
        &mut self,
        direction: ratcore::prompt_completion::PromptDirection,
        source: &dyn FuzzySource,
    ) {
        let transition = ratcore::prompt_completion::reduce_prompt(
            &self.prompt,
            ratcore::prompt_completion::PromptMessage::Move(direction),
        );
        if transition.accepted {
            self.prompt = transition.session;
        }
        self.project(source);
    }

    /// Returns the number of filtered projected candidates.
    #[must_use]
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Takes the accepted domain candidate after reducer acceptance.
    pub fn take_result(&mut self) -> Option<FuzzyCandidate> {
        self.result.take()
    }

    /// Submits the active stable identity and closes only after reducer acceptance.
    pub fn confirm(&mut self, source: &dyn FuzzySource) {
        let generation = self.prompt.completion().map(|snapshot| snapshot.generation);
        let transition = ratcore::prompt_completion::reduce_prompt(
            &self.prompt,
            ratcore::prompt_completion::PromptMessage::Submit {
                input_revision: self.prompt.input_revision(),
                candidate_generation: generation,
            },
        );
        if let Some(ratcore::prompt_completion::PromptOutput::Candidate { id, .. }) = &transition.output {
            self.result = source.candidates().iter().find(|candidate| candidate.id == id.as_str()).cloned();
        }
        if transition.accepted {
            self.prompt = transition.session;
            self.open = false;
        }
    }

    pub(crate) fn cancel(&mut self) {
        let transition =
            ratcore::prompt_completion::reduce_prompt(&self.prompt, ratcore::prompt_completion::PromptMessage::Cancel);
        if transition.accepted {
            self.prompt = transition.session;
            self.open = false;
        }
    }

    fn synchronize_public_query(&mut self) {
        if self.query == self.prompt.input() {
            return;
        }
        let transition = ratcore::prompt_completion::reduce_prompt(
            &self.prompt,
            ratcore::prompt_completion::PromptMessage::ReplaceInput {
                input: self.query.clone(),
                expected_revision: self.prompt.input_revision(),
            },
        );
        if transition.accepted {
            self.prompt = transition.session;
        }
    }

    fn project(&mut self, source: &dyn FuzzySource) {
        let projection = ratcore::prompt_completion::project_prompt(&self.prompt);
        self.query = projection.input;
        self.results = projection
            .candidates
            .iter()
            .filter_map(|projected| {
                let index = source.candidates().iter().position(|candidate| candidate.id == projected.id.as_str())?;
                if self.query.is_empty() {
                    return Some(crate::score::ScoredMatch {
                        index: crate::score::CandidateIndex::from_zero_based(index),
                        score: 0,
                        positions: Vec::new(),
                    });
                }
                crate::score::fuzzy_score(crate::score::FuzzyScoreInput {
                    text: projected.label.as_str(),
                    query: self.query.as_str(),
                })
                .map(|mut matched| {
                    matched.index = crate::score::CandidateIndex::from_zero_based(index);
                    matched
                })
            })
            .collect();
        self.selected = projection.candidates.iter().position(|candidate| candidate.active).unwrap_or_default();
        self.scroll_offset = self.scroll_offset.min(self.selected);
    }
}

impl Default for FuzzyState {
    fn default() -> Self {
        Self::new()
    }
}
