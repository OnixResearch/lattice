use super::*;

enum PredicateEvalFrame<'a> {
    Evaluate(&'a Predicate),
    ApplyNot,
    ApplyAnd(usize),
    ApplyOr(usize),
}

struct PredicateEvalState<'a> {
    frames: Vec<PredicateEvalFrame<'a>>,
    results: Vec<bool>,
}

impl<'a> PredicateEvalState<'a> {
    fn new(predicate: &'a Predicate) -> Self {
        let mut frames = Vec::with_capacity(1);
        frames.push(PredicateEvalFrame::Evaluate(predicate));
        let results = Vec::with_capacity(1);
        debug_assert_eq!(frames.len(), 1);
        debug_assert!(results.capacity() >= results.len());
        Self { frames, results }
    }

    fn evaluate<R: QueryRecord>(&mut self, record: &R) {
        while let Some(frame) = self.frames.pop() {
            match frame {
                PredicateEvalFrame::Evaluate(current) => self.push_eval_frame(current, record),
                PredicateEvalFrame::ApplyNot => self.apply_not(),
                PredicateEvalFrame::ApplyAnd(child_count) => self.fold_results(child_count, true),
                PredicateEvalFrame::ApplyOr(child_count) => self.fold_results(child_count, false),
            }
        }
    }

    fn finish(&self) -> bool {
        matches!(self.results.as_slice(), [true])
    }

    fn push_eval_frame<R: QueryRecord>(&mut self, predicate: &'a Predicate, record: &R) {
        debug_assert!(self.frames.len() <= self.frames.capacity());
        debug_assert!(self.results.len() <= self.results.capacity());
        match predicate {
            Predicate::Always => self.results.push(true),
            Predicate::Field { field, operator, value } => self
                .results
                .push(record.get_value(field).is_some_and(|actual| operator_matches(actual, *operator, value))),
            Predicate::And(children) => {
                self.frames.push(PredicateEvalFrame::ApplyAnd(children.len()));
                self.frames.extend(children.iter().rev().map(PredicateEvalFrame::Evaluate));
            }
            Predicate::Or(children) => {
                self.frames.push(PredicateEvalFrame::ApplyOr(children.len()));
                self.frames.extend(children.iter().rev().map(PredicateEvalFrame::Evaluate));
            }
            Predicate::Not(child) => {
                self.frames.push(PredicateEvalFrame::ApplyNot);
                self.frames.push(PredicateEvalFrame::Evaluate(child));
            }
        }
    }

    fn apply_not(&mut self) {
        let Some(value) = self.results.pop() else {
            return;
        };
        self.results.push(!value);
    }

    fn fold_results(&mut self, child_count: usize, identity: bool) {
        let mut has_folded_match = identity;
        for _ in 0..child_count {
            let Some(value) = self.results.pop() else {
                return;
            };
            has_folded_match = if identity {
                has_folded_match && value
            } else {
                has_folded_match || value
            };
        }
        self.results.push(has_folded_match);
    }
}

pub(super) fn predicate_matches<R: QueryRecord>(record: &R, predicate: &Predicate) -> bool {
    let mut state = PredicateEvalState::new(predicate);
    state.evaluate(record);
    state.finish()
}

fn operator_matches(actual: &QueryValue, operator: QueryOperator, expected: &QueryValue) -> bool {
    match operator {
        QueryOperator::Equals => actual == expected,
        QueryOperator::NotEquals => actual != expected,
        QueryOperator::Contains => match expected {
            QueryValue::Text(text) => actual.text_contains(text),
            QueryValue::Tags(tags) => tags.iter().all(|tag| actual.text_contains(tag)),
            QueryValue::Number(_) | QueryValue::Bool(_) | QueryValue::DateOrdinal(_) => false,
        },
        QueryOperator::StartsWith => match expected {
            QueryValue::Text(text) => actual.text_starts_with(text),
            QueryValue::Tags(tags) => tags.iter().all(|tag| actual.text_starts_with(tag)),
            QueryValue::Number(_) | QueryValue::Bool(_) | QueryValue::DateOrdinal(_) => false,
        },
        QueryOperator::GreaterThan => actual > expected,
        QueryOperator::LessThan => actual < expected,
    }
}

pub(super) fn date_ranges_match<R: QueryRecord>(record: &R, clauses: &[DateRangeClause]) -> bool {
    clauses.iter().all(|clause| {
        record.get_value(&clause.field).is_some_and(|value| match value {
            QueryValue::DateOrdinal(date) => clause.range.contains(*date),
            QueryValue::Text(_) | QueryValue::Number(_) | QueryValue::Bool(_) | QueryValue::Tags(_) => false,
        })
    })
}

pub(super) fn search_matches<R: QueryRecord>(record: &R, searches: &[SearchTerm]) -> (bool, u16) {
    if searches.is_empty() {
        return (true, EMPTY_SEARCH_SCORE);
    }
    let mut score = EMPTY_SEARCH_SCORE;
    debug_assert_eq!(score, EMPTY_SEARCH_SCORE);
    debug_assert!(searches.len() <= searches.len().saturating_add(1));
    for search in searches {
        if search.term.is_empty() {
            continue;
        }
        let mut has_term_match = false;
        for field in &search.fields {
            if record.get_value(field).is_some_and(|value| value.text_contains(&search.term)) {
                has_term_match = true;
                score = score.saturating_add(FIELD_MATCH_SCORE);
            }
        }
        if !has_term_match {
            return (false, score);
        }
    }
    (true, score)
}

pub(super) fn matched_fields<R: QueryRecord>(record: &R, searches: &[SearchTerm]) -> Vec<String> {
    let mut fields = std::collections::BTreeSet::new();
    for search in searches {
        if search.term.is_empty() {
            continue;
        }
        for field in &search.fields {
            if record.get_value(field).is_some_and(|value| value.text_contains(&search.term)) {
                fields.insert(field.clone());
            }
        }
    }
    fields.into_iter().collect()
}
